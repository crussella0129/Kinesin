# Sprint 9 Meta

- **Sprint number:** 9
- **Book schema version:** 2
- **Start timestamp:** 2026-09-12T00:13:25Z
- **End timestamp:** 2026-09-12T06:05:53Z
- **Model:** claude-opus-4-8
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Integrate operator-approved local (stdio) MCP tool servers on the official rmcp SDK — generalize the closed ToolName enum to a ToolRef (compiled ∪ MCP), declare+validate servers in config, discover tool schemas and freeze them into the run authority, emit them to the model, and dispatch tools/call through the existing authority/allow-list/byte/time gates as untrusted evidence-free observations, reproducible under deterministic replay without reconnecting (INT-0005). Remote/HTTP+OAuth split to INT-0020.
- **Intents:** [INT-0005](../../intents/INT-0005-mcp-tool-servers.md) — realized
- **Completion evidence:** INT-0005 realized: local stdio MCP tool-server integration on rmcp v3.3.0 — ToolRef generalization, operator [[mcp.servers]] gate, discovery frozen into the run authority, schema emitted to the model, tools/call dispatched through the existing gates as untrusted evidence-free observations, replay reproduces without reconnecting; tests/mcp.rs (approved/denied/poison-as-data/missing/timeout + E2E run-and-replay) + unit tests green on both CI OSes (a2b3bce); cargo deny/audit green; remote+OAuth split to INT-0020
