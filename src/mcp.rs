//! Operator-approved MCP tool servers reached over stdio. A server is a trusted
//! local binary the operator declared in config (like the model endpoint); what
//! stays untrusted is its tool descriptions and outputs, which are data, never
//! authority. Discovery runs under admitted active ownership and its result is
//! frozen in the startup journal before dispatch, so replay reproduces requests
//! re-validates each call without ever reconnecting.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock, PaginatedRequestParams};
use rmcp::service::{Peer, RoleClient, RunningService};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio_util::sync::CancellationToken;

use crate::config::{McpServer, ToolRef};
use crate::tools::{ToolError, ToolResult, ToolStatus};

mod transport;

pub const MAX_MCP_FRAME_BYTES: usize = 256 * 1024;
pub const MAX_MCP_SESSION_BYTES: usize = 16 * 1024 * 1024;
/// All inbound lines, including notifications and empty/unterminated frames.
/// This protocol ceiling is independent of the run's tool-call budget.
pub const MAX_MCP_SESSION_FRAMES: usize = 4096;
pub const MAX_MCP_METADATA_BYTES: usize = 64 * 1024;
pub const MAX_MCP_SERVERS: usize = 8;
pub const MAX_MCP_TOOLS: usize = 64;
pub const MAX_MCP_PAGES: usize = 16;
pub const MAX_MCP_ADVERTISED_TOOLS: usize = 1024;

/// Bounds connect+initialize and each `tools/list`, so a server that never
/// answers fails run start with a defined error instead of hanging it.
pub const MCP_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// An overall deadline for discovering all of a run's MCP servers. The runner
/// also applies its admission deadline and cancellation before creating children.
pub const MCP_DISCOVERY_DEADLINE: Duration = Duration::from_secs(20);

/// Caps on the untrusted, server-supplied tool metadata that is frozen into the
/// startup journal and embedded verbatim in every model request. A
/// server cannot bloat the journal or push requests past the model's request
/// budget with an oversized schema or description.
pub const MAX_MCP_SCHEMA_BYTES: usize = 16 * 1024;
pub const MAX_MCP_DESCRIPTION_BYTES: usize = 4 * 1024;

/// One discovered MCP tool, frozen in the startup journal. `input_schema` is the
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

/// A frozen description is data, never a second tool grant. Validate the whole
/// discovered set independently on live startup and replay before using it.
pub fn frozen_tools_digest(allow: &[ToolRef], defs: &[McpToolDef]) -> Result<String, &'static str> {
    let expected = allow.iter().filter(|tool| tool.mcp().is_some()).count();
    if defs.len() > MAX_MCP_TOOLS || defs.len() != expected {
        return Err("mcp_frozen_tool_count");
    }
    let mut seen = HashSet::new();
    for def in defs {
        let identity = ToolRef::Mcp {
            server: def.server.clone(),
            tool: def.tool.clone(),
        };
        if !allow.contains(&identity) || !seen.insert(identity) {
            return Err("mcp_frozen_tool_denied");
        }
        if def.description.len() > MAX_MCP_DESCRIPTION_BYTES
            || !def.input_schema.is_object()
            || serde_json::to_vec(&def.input_schema)
                .map_err(|_| "mcp_schema_invalid")?
                .len()
                > MAX_MCP_SCHEMA_BYTES
        {
            return Err("mcp_frozen_schema_invalid");
        }
    }
    let encoded = serde_json::to_vec(defs).map_err(|_| "mcp_schema_invalid")?;
    if encoded.len() > MAX_MCP_METADATA_BYTES {
        return Err("mcp_metadata_limit");
    }
    Ok(crate::model::fingerprint(&encoded))
}

/// Stable, content-free startup outcomes accepted in a replay capture. Provider
/// text is never copied into a terminal reason or made a control instruction.
pub fn is_preparation_error(code: &str) -> bool {
    matches!(
        code,
        "mcp_server_limit"
            | "mcp_server_undeclared"
            | "mcp_server_duplicate"
            | "mcp_command_empty"
            | "mcp_spawn_failed"
            | "mcp_pipe_failed"
            | "mcp_initialize_timeout"
            | "mcp_initialize_failed"
            | "mcp_tool_limit"
            | "mcp_server_unconnected"
            | "mcp_list_timeout"
            | "mcp_list_failed"
            | "mcp_advertised_tool_limit_or_duplicate"
            | "mcp_description_too_large"
            | "mcp_schema_invalid"
            | "mcp_schema_too_large"
            | "mcp_metadata_limit"
            | "mcp_pagination_limit"
            | "mcp_tool_absent"
            | "mcp_frozen_tool_count"
            | "mcp_frozen_tool_denied"
            | "mcp_frozen_schema_invalid"
            | "mcp_authority_already_prepared"
            | "mcp_prepare_cancelled"
            | "mcp_prepare_timeout"
    )
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

/// Run-scoped sessions plus explicit process ownership. Normal release calls
/// shutdown and awaits cleanup; dropping remains a process-kill fallback.
#[derive(Default)]
pub struct McpClientPool {
    clients: BTreeMap<String, Peer<RoleClient>>,
    services: tokio::sync::Mutex<Vec<RunningService<RoleClient, ()>>>,
    processes: tokio::sync::Mutex<BTreeMap<String, crate::process::OwnedProcess>>,
}

impl McpClientPool {
    pub async fn connect(
        servers: &[McpServer],
        needed: &HashSet<String>,
        timeout: Duration,
    ) -> Result<Self, McpError> {
        let mut pool = Self::default();
        if let Err(error) = pool.connect_into(servers, needed, timeout).await {
            pool.shutdown().await?;
            return Err(error);
        }
        Ok(pool)
    }

    /// Retain every spawned process in this pool before awaiting initialization.
    /// A caller may cancel this future and still await the pool's cleanup.
    pub async fn connect_into(
        &mut self,
        servers: &[McpServer],
        needed: &HashSet<String>,
        timeout: Duration,
    ) -> Result<(), McpError> {
        if needed.len() > MAX_MCP_SERVERS {
            return Err(McpError::new("mcp_server_limit", ""));
        }
        let mut ids: Vec<_> = needed.iter().collect();
        ids.sort();
        for id in ids {
            let server = servers
                .iter()
                .find(|server| &server.id == id)
                .ok_or_else(|| McpError::new("mcp_server_undeclared", id))?;
            if self.processes.get_mut().contains_key(id) {
                return Err(McpError::new("mcp_server_duplicate", id));
            }
            let (exe, args) = server
                .command
                .split_first()
                .ok_or_else(|| McpError::new("mcp_command_empty", id))?;
            let mut command = tokio::process::Command::new(exe);
            crate::process::scrub_environment(&mut command);
            command
                .args(args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null());
            let mut process = crate::process::OwnedProcess::spawn(&mut command)
                .map_err(|_| McpError::new("mcp_spawn_failed", id))?;
            let stdout = process
                .take_stdout()
                .ok_or_else(|| McpError::new("mcp_pipe_failed", id))?;
            let stdin = process
                .take_stdin()
                .ok_or_else(|| McpError::new("mcp_pipe_failed", id))?;
            self.processes.get_mut().insert(id.clone(), process);
            let reader = transport::BoundedReader::new(
                stdout,
                MAX_MCP_FRAME_BYTES,
                MAX_MCP_SESSION_BYTES,
                MAX_MCP_SESSION_FRAMES,
            );
            let transport = rmcp::transport::async_rw::AsyncRwTransport::new_client(reader, stdin);
            let client = tokio::time::timeout(timeout, ().serve(transport))
                .await
                .map_err(|_| McpError::new("mcp_initialize_timeout", id))?
                .map_err(|_| McpError::new("mcp_initialize_failed", id))?;
            self.clients.insert(id.clone(), client.peer().clone());
            self.services.get_mut().push(client);
        }
        Ok(())
    }

    /// Close protocol tasks and terminate the owned groups/jobs before releasing
    /// normal run ownership. Drop remains a synchronous process-kill fallback.
    pub async fn shutdown(&self) -> Result<(), McpError> {
        let mut services = self.services.lock().await;
        for service in services.iter() {
            service.cancellation_token().cancel();
        }
        let mut processes = self.processes.lock().await;
        let mut failure = None;
        for (server, process) in processes.iter_mut() {
            if process.terminate_and_wait().await.is_err() && failure.is_none() {
                failure = Some(McpError::new("mcp_cleanup_failed", server));
            }
        }
        processes.clear();
        for service in services.iter_mut() {
            if service.close().await.is_err() && failure.is_none() {
                failure = Some(McpError::new("mcp_cleanup_failed", ""));
            }
        }
        services.clear();
        failure.map_or(Ok(()), Err)
    }

    /// Page explicitly instead of using the SDK's accumulating list_all_tools.
    pub async fn discover(
        &self,
        allow: &[ToolRef],
        timeout: Duration,
    ) -> Result<Vec<McpToolDef>, McpError> {
        let mut wanted: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for tool_ref in allow {
            if let Some((server, tool)) = tool_ref.mcp() {
                wanted.entry(server).or_default().push(tool);
            }
        }
        if allow.iter().filter(|tool| tool.mcp().is_some()).count() > MAX_MCP_TOOLS {
            return Err(McpError::new("mcp_tool_limit", ""));
        }
        let mut defs = Vec::new();
        // Include the array delimiters and separators in the captured envelope.
        let mut frozen_bytes = 2_usize;
        for (server, tools) in wanted {
            let client = self
                .clients
                .get(server)
                .ok_or_else(|| McpError::new("mcp_server_unconnected", server))?;
            let mut cursor = None;
            let mut cursors = HashSet::new();
            let mut names = HashSet::new();
            let mut found = BTreeMap::new();
            for page_index in 0..MAX_MCP_PAGES {
                let params = PaginatedRequestParams::default().with_cursor(cursor.take());
                let page = tokio::time::timeout(timeout, client.list_tools(Some(params)))
                    .await
                    .map_err(|_| McpError::new("mcp_list_timeout", server))?
                    .map_err(|_| McpError::new("mcp_list_failed", server))?;
                for tool in page.tools {
                    let name = tool.name.to_string();
                    if names.len() >= MAX_MCP_ADVERTISED_TOOLS || !names.insert(name.clone()) {
                        return Err(McpError::new(
                            "mcp_advertised_tool_limit_or_duplicate",
                            server,
                        ));
                    }
                    if !tools.contains(&name.as_str()) {
                        continue;
                    }
                    let description = tool.description.as_deref().unwrap_or_default().to_owned();
                    if description.len() > MAX_MCP_DESCRIPTION_BYTES {
                        return Err(McpError::new("mcp_description_too_large", server));
                    }
                    let input_schema = Value::Object((*tool.input_schema).clone());
                    let schema_bytes = serde_json::to_vec(&input_schema)
                        .map_err(|_| McpError::new("mcp_schema_invalid", server))?
                        .len();
                    if schema_bytes > MAX_MCP_SCHEMA_BYTES {
                        return Err(McpError::new("mcp_schema_too_large", server));
                    }
                    let def = McpToolDef {
                        server: server.to_owned(),
                        tool: name.clone(),
                        description,
                        input_schema,
                    };
                    frozen_bytes = frozen_bytes
                        .checked_add(
                            serde_json::to_vec(&def)
                                .map_err(|_| McpError::new("mcp_schema_invalid", server))?
                                .len()
                                + 1,
                        )
                        .ok_or_else(|| McpError::new("mcp_metadata_limit", server))?;
                    if frozen_bytes > MAX_MCP_METADATA_BYTES {
                        return Err(McpError::new("mcp_metadata_limit", server));
                    }
                    found.insert(name, def);
                }
                let Some(next) = page.next_cursor else { break };
                if next.len() > 1024
                    || !cursors.insert(next.clone())
                    || page_index + 1 == MAX_MCP_PAGES
                {
                    return Err(McpError::new("mcp_pagination_limit", server));
                }
                cursor = Some(next);
            }
            for tool in tools {
                defs.push(
                    found
                        .remove(tool)
                        .ok_or_else(|| McpError::new("mcp_tool_absent", server))?,
                );
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
                if self.shutdown().await.is_err() {
                    return ToolResult::failure(ToolStatus::Error, "mcp_cleanup_failed", "MCP cleanup failed");
                }
                return ToolResult::failure(ToolStatus::Error, "mcp_cancelled", "MCP call cancelled");
            }
            outcome = tokio::time::timeout(timeout, client.call_tool(params)) => outcome,
        };
        match outcome {
            Err(_) => {
                if self.shutdown().await.is_err() {
                    return ToolResult::failure(
                        ToolStatus::Error,
                        "mcp_cleanup_failed",
                        "MCP cleanup failed",
                    );
                }
                ToolResult::failure(ToolStatus::Error, "mcp_call_timeout", "MCP call timed out")
            }
            Ok(Err(_)) => {
                if self.shutdown().await.is_err() {
                    return ToolResult::failure(
                        ToolStatus::Error,
                        "mcp_cleanup_failed",
                        "MCP cleanup failed",
                    );
                }
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
