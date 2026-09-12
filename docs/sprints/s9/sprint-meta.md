# Sprint 9 Meta

- **Sprint number:** 9
- **Book schema version:** 2
- **Start timestamp:** 2026-09-12T00:13:25Z
- **End timestamp:** (filled at Loop Phase)
- **Model:** claude-opus-4-8
- **Bundle version:** 0.22.0
- **Exit status:** in-progress
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Integrate operator-approved local (stdio) MCP tool servers on the official rmcp SDK — generalize the closed ToolName enum to a ToolRef (compiled ∪ MCP), declare+validate servers in config, discover tool schemas and freeze them into the run authority, emit them to the model, and dispatch tools/call through the existing authority/allow-list/byte/time gates as untrusted evidence-free observations, reproducible under deterministic replay without reconnecting (INT-0005). Remote/HTTP+OAuth split to INT-0020.
- **Intents:** [INT-0005](../../intents/INT-0005-mcp-tool-servers.md) — planned
- **Completion evidence:** (filled at Loop Phase)
