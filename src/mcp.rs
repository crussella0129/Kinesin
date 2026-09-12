//! Operator-approved MCP tool servers reached over stdio. A server is a trusted
//! local binary the operator declared in config (like the model endpoint); what
//! stays untrusted is its tool descriptions and outputs, which are data, never
//! authority. Discovery runs once at run start and its result is frozen into the
//! run authority, so deterministic replay reproduces the model request and
//! re-validates each call without ever reconnecting.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::TokioChildProcess;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio_util::sync::CancellationToken;

use crate::config::{McpServer, ToolRef};
use crate::tools::{ToolError, ToolResult, ToolStatus};

/// Bounds connect+initialize and each `tools/list`, so a server that never
/// answers fails run start with a defined error instead of hanging it.
pub const MCP_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// An overall deadline for discovering all of a run's MCP servers, so run start
/// cannot block for `server count × MCP_STARTUP_TIMEOUT` when several servers are
/// slow — connects run concurrently under this single bound.
pub const MCP_DISCOVERY_DEADLINE: Duration = Duration::from_secs(20);

/// Caps on the untrusted, server-supplied tool metadata that is frozen into the
/// run authority (and journaled) and embedded verbatim in every model request. A
/// server cannot bloat the journal or push requests past the model's request
/// budget with an oversized schema or description.
pub const MAX_MCP_SCHEMA_BYTES: usize = 16 * 1024;
pub const MAX_MCP_DESCRIPTION_BYTES: usize = 4 * 1024;

/// One discovered MCP tool, frozen into the run authority. `input_schema` is the
/// server-advertised JSON Schema, emitted verbatim to the model and used to
/// validate a call's arguments; it is untrusted data, not authority.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct McpToolDef {
    pub server: String,
    pub tool: String,
    #[serde(default)]
    pub description: String,
    pub input_schema: Value,
}

impl McpToolDef {
    /// The namespaced wire name the model sees and a tool call carries.
    pub fn wire_name(&self) -> String {
        ToolRef::Mcp {
            server: self.server.clone(),
            tool: self.tool.clone(),
        }
        .wire_name()
    }
}

/// A run-scoped error reaching or discovering an MCP server. It carries a stable
/// code and never embeds server-returned text, so an untrusted server cannot
/// shape an operator-facing message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpError {
    pub code: &'static str,
    pub server: String,
}

impl McpError {
    fn new(code: &'static str, server: &str) -> Self {
        Self {
            code,
            server: server.to_owned(),
        }
    }
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (mcp server '{}')", self.code, self.server)
    }
}

impl std::error::Error for McpError {}

/// The live stdio client sessions for one run's declared-and-used MCP servers,
/// opened once at run start and held for the run. Dropping the pool drops each
/// session, tearing down its child process.
pub struct McpClientPool {
    clients: BTreeMap<String, RunningService<RoleClient, ()>>,
}

impl McpClientPool {
    /// Spawn and initialize exactly the servers named in `needed` (the run's
    /// allow-listed MCP servers), skipping any declared server the run does not
    /// use. A missing declaration is a caller error (config validation already
    /// proved every referenced server is declared), so it is reported, not
    /// silently skipped.
    pub async fn connect(
        servers: &[McpServer],
        needed: &HashSet<String>,
        timeout: Duration,
    ) -> Result<Self, McpError> {
        // Connect every needed server concurrently, so run-start latency is the
        // slowest server's handshake, not the sum across servers.
        let mut connects = Vec::with_capacity(needed.len());
        for id in needed {
            let server = servers
                .iter()
                .find(|s| &s.id == id)
                .ok_or_else(|| McpError::new("mcp_server_undeclared", id))?;
            connects.push(Self::connect_one(server, timeout));
        }
        let clients = futures_util::future::try_join_all(connects)
            .await?
            .into_iter()
            .collect();
        Ok(Self { clients })
    }

    async fn connect_one(
        server: &McpServer,
        timeout: Duration,
    ) -> Result<(String, RunningService<RoleClient, ()>), McpError> {
        let (exe, args) = server
            .command
            .split_first()
            .ok_or_else(|| McpError::new("mcp_command_empty", &server.id))?;
        let mut command = tokio::process::Command::new(exe);
        command.args(args);
        let transport = TokioChildProcess::new(command)
            .map_err(|_| McpError::new("mcp_spawn_failed", &server.id))?;
        let client = tokio::time::timeout(timeout, ().serve(transport))
            .await
            .map_err(|_| McpError::new("mcp_initialize_timeout", &server.id))?
            .map_err(|_| McpError::new("mcp_initialize_failed", &server.id))?;
        Ok((server.id.clone(), client))
    }

    /// Discover the schema of each allow-listed MCP tool, listing each server's
    /// tools once. Errors if an allow-listed tool is absent from its server, so a
    /// run never starts offering a tool the server does not actually provide.
    pub async fn discover(
        &self,
        allow: &[ToolRef],
        timeout: Duration,
    ) -> Result<Vec<McpToolDef>, McpError> {
        // Which tools each server must yield, in allow-list order per server.
        let mut wanted: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for tool_ref in allow {
            if let Some((server, tool)) = tool_ref.mcp() {
                wanted.entry(server).or_default().push(tool);
            }
        }
        let mut defs = Vec::new();
        for (server, tools) in wanted {
            let client = self
                .clients
                .get(server)
                .ok_or_else(|| McpError::new("mcp_server_unconnected", server))?;
            let advertised = tokio::time::timeout(timeout, client.list_all_tools())
                .await
                .map_err(|_| McpError::new("mcp_list_timeout", server))?
                .map_err(|_| McpError::new("mcp_list_failed", server))?;
            for tool in tools {
                let found = advertised
                    .iter()
                    .find(|candidate| candidate.name.as_ref() == tool)
                    .ok_or_else(|| McpError::new("mcp_tool_absent", server))?;
                // Bound the untrusted schema/description before freezing it into
                // the run and every request, so a server cannot bloat the journal
                // or overflow the model's request budget.
                let description = found
                    .description
                    .as_ref()
                    .map(|d| d.as_ref().to_owned())
                    .unwrap_or_default();
                if description.len() > MAX_MCP_DESCRIPTION_BYTES {
                    return Err(McpError::new("mcp_description_too_large", server));
                }
                let input_schema = Value::Object((*found.input_schema).clone());
                if serde_json::to_vec(&input_schema).map_or(usize::MAX, |bytes| bytes.len())
                    > MAX_MCP_SCHEMA_BYTES
                {
                    return Err(McpError::new("mcp_schema_too_large", server));
                }
                defs.push(McpToolDef {
                    server: server.to_owned(),
                    tool: tool.to_owned(),
                    description,
                    input_schema,
                });
            }
        }
        Ok(defs)
    }
}

impl McpClientPool {
    /// Invoke one allow-listed MCP tool, bounded by `timeout` and cancellation and
    /// with its textual result bounded to `max_bytes`. The result is an untrusted
    /// observation: it mints no evidence and cannot change policy. A transport,
    /// timeout, or server error becomes a defined failure `ToolResult`, never a
    /// panic and never authority.
    pub async fn call(
        &self,
        server: &str,
        tool: &str,
        arguments: Map<String, Value>,
        timeout: std::time::Duration,
        cancel: &CancellationToken,
        max_bytes: usize,
    ) -> ToolResult {
        let Some(client) = self.clients.get(server) else {
            return ToolResult::failure(
                ToolStatus::Error,
                "mcp_unavailable",
                "MCP server session is unavailable",
            );
        };
        // `CallToolRequestParams` is `#[non_exhaustive]`, so build it from Default
        // and set the two fields we own rather than a struct literal.
        let mut params = CallToolRequestParams::default();
        params.name = tool.to_owned().into();
        params.arguments = Some(arguments);
        let outcome = tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                return ToolResult::failure(ToolStatus::Error, "mcp_cancelled", "MCP call cancelled");
            }
            outcome = tokio::time::timeout(timeout, client.call_tool(params)) => outcome,
        };
        match outcome {
            Err(_) => {
                ToolResult::failure(ToolStatus::Error, "mcp_call_timeout", "MCP call timed out")
            }
            Ok(Err(_)) => {
                ToolResult::failure(ToolStatus::Error, "mcp_call_failed", "MCP call failed")
            }
            Ok(Ok(result)) => map_result(result, max_bytes),
        }
    }
}

/// Map an MCP `CallToolResult` into a bounded, untrusted `ToolResult`. Text
/// content is concatenated; a non-text block is noted by kind, not inlined. The
/// body is the server's own text — data, never trusted — and mints no evidence.
///
/// The body is bounded exactly like a compiled tool's result: `max_bytes` is
/// first clamped to `MAX_TOOL_BYTES`, and the JSON **envelope** overhead is
/// reserved so the *encoded* `ToolResult` — not just the raw body — stays within
/// the cap the replay validator enforces (`encoded().len() <= max.min(8192)`).
/// Bounding only the raw body would let a large server result exceed that cap and
/// break deterministic replay.
fn map_result(result: CallToolResult, max_bytes: usize) -> ToolResult {
    let mut text = String::new();
    for block in &result.content {
        if !text.is_empty() {
            text.push('\n');
        }
        match block {
            ContentBlock::Text(content) => text.push_str(&content.text),
            ContentBlock::Image(_) => text.push_str("[non-text content: image]"),
            ContentBlock::Audio(_) => text.push_str("[non-text content: audio]"),
            ContentBlock::Resource(_) => text.push_str("[non-text content: resource]"),
            ContentBlock::ResourceLink(_) => text.push_str("[non-text content: resource link]"),
            _ => text.push_str("[non-text content]"),
        }
    }
    let error = result.is_error.unwrap_or(false).then(|| ToolError {
        code: "mcp_tool_error".into(),
        message: "The MCP tool reported an error".into(),
    });
    let mut mapped = ToolResult {
        status: if error.is_some() {
            ToolStatus::Error
        } else {
            ToolStatus::Ok
        },
        body: String::new(),
        truncated: false,
        error,
        evidence_id: None,
    };
    // Reserve the envelope (the encoded result with an empty body) so the body's
    // *escaped* length fits the remaining budget, mirroring the compiled tools.
    let limit = max_bytes.min(crate::tools::MAX_TOOL_BYTES);
    let overhead = mapped.encoded().map(|e| e.len()).unwrap_or(limit);
    let kept = crate::tools::escaped_prefix(&text, limit.saturating_sub(overhead));
    mapped.truncated = kept.len() < text.len();
    mapped.body = kept.to_owned();
    debug_assert!(mapped.encoded().map(|e| e.len()).unwrap_or(usize::MAX) <= limit);
    mapped
}

/// Validate a call's raw JSON arguments against an MCP tool's discovered input
/// schema, returning the argument object on success. This is a lightweight check
/// — the argument value must be a JSON object and every `required` property named
/// by the schema must be present. Full JSON-Schema validation is a documented
/// follow-up; the server remains the authority on its own schema. It performs no
/// I/O, so an invalid call is refused before any server is contacted.
pub fn validate_args(raw: &str, input_schema: &Value) -> Option<Map<String, Value>> {
    let Value::Object(object) = serde_json::from_str::<Value>(raw).ok()? else {
        return None;
    };
    if let Some(Value::Array(required)) = input_schema.get("required") {
        for key in required {
            match key.as_str() {
                Some(name) if object.contains_key(name) => {}
                _ => return None,
            }
        }
    }
    Some(object)
}

/// The set of MCP server ids an allow-list references, for `connect`.
pub fn needed_servers(allow: &[ToolRef]) -> HashSet<String> {
    allow
        .iter()
        .filter_map(|tool| tool.mcp().map(|(server, _)| server.to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_args_requires_object_and_required_keys() {
        let schema = json!({"type":"object","required":["path"]});
        // A well-formed object with the required key passes and returns the map.
        let ok = validate_args(r#"{"path":"a.txt","extra":1}"#, &schema).unwrap();
        assert_eq!(ok.get("path").unwrap(), "a.txt");
        // Missing a required key, a non-object, and malformed JSON are all refused
        // — before any server is contacted.
        assert!(validate_args(r#"{"other":1}"#, &schema).is_none());
        assert!(validate_args(r#"["path"]"#, &schema).is_none());
        assert!(validate_args("not json", &schema).is_none());
        // With no `required` list, any object is accepted.
        let open = json!({"type":"object"});
        assert!(validate_args(r#"{}"#, &open).is_some());
    }

    #[test]
    fn map_result_bounds_encoded_result_within_cap() {
        use rmcp::model::TextContent;
        // A large server result is bounded so the *encoded* ToolResult (body +
        // JSON envelope) fits the cap the replay validator enforces — not just the
        // raw body, which would overflow the envelope and break replay.
        let big = "x".repeat(10_000);
        let mapped = map_result(
            CallToolResult::success(vec![ContentBlock::Text(TextContent::new(big))]),
            256,
        );
        assert!(mapped.truncated);
        let encoded = mapped.encoded().unwrap();
        assert!(
            encoded.len() <= 256,
            "encoded result {} exceeds the cap",
            encoded.len()
        );
        // A small result is kept whole and not marked truncated.
        let small = map_result(
            CallToolResult::success(vec![ContentBlock::Text(TextContent::new("hi"))]),
            8192,
        );
        assert!(!small.truncated);
        assert_eq!(small.body, "hi");
    }

    #[test]
    fn tooldef_wire_name_matches_toolref() {
        let def = McpToolDef {
            server: "docs".into(),
            tool: "grep".into(),
            description: String::new(),
            input_schema: json!({"type":"object"}),
        };
        assert_eq!(def.wire_name(), "mcp__docs__grep");
    }
}
