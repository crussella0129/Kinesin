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
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::TokioChildProcess;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{McpServer, ToolRef};

/// Bounds connect+initialize and each `tools/list`, so a server that never
/// answers fails run start with a defined error instead of hanging it.
pub const MCP_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

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
        let mut clients = BTreeMap::new();
        for id in needed {
            let server = servers
                .iter()
                .find(|s| &s.id == id)
                .ok_or_else(|| McpError::new("mcp_server_undeclared", id))?;
            let (exe, args) = server
                .command
                .split_first()
                .ok_or_else(|| McpError::new("mcp_command_empty", id))?;
            let mut command = tokio::process::Command::new(exe);
            command.args(args);
            let transport = TokioChildProcess::new(command)
                .map_err(|_| McpError::new("mcp_spawn_failed", id))?;
            let client = tokio::time::timeout(timeout, ().serve(transport))
                .await
                .map_err(|_| McpError::new("mcp_initialize_timeout", id))?
                .map_err(|_| McpError::new("mcp_initialize_failed", id))?;
            clients.insert(server.id.clone(), client);
        }
        Ok(Self { clients })
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
                defs.push(McpToolDef {
                    server: server.to_owned(),
                    tool: tool.to_owned(),
                    description: found
                        .description
                        .as_ref()
                        .map(|d| d.as_ref().to_owned())
                        .unwrap_or_default(),
                    input_schema: Value::Object((*found.input_schema).clone()),
                });
            }
        }
        Ok(defs)
    }
}

/// The set of MCP server ids an allow-list references, for `connect`.
pub fn needed_servers(allow: &[ToolRef]) -> HashSet<String> {
    allow
        .iter()
        .filter_map(|tool| tool.mcp().map(|(server, _)| server.to_owned()))
        .collect()
}
