# Sprint 9 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Allow-listed MCP tools run under the same per-run gates and mint no evidence | T-005 / WHEN an allow-listed MCP tool is called with valid args THEN it SHALL run within byte/time/concurrency bounds and mint no evidence | `mcp_approved_call_bounded_no_evidence`, `mcp_result_truncated_at_byte_cap` |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Two-gate approval: undeclared server denied; un-allow-listed tool denied | T-002 / WHEN a config allow-lists an undeclared server THEN validation SHALL fail; T-005 / WHEN a call names a tool not in the allow-list THEN it SHALL be Denied without contacting a server | `mcp_config_rejects_undeclared_server`, `mcp_unapproved_call_denied` |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Server text (descriptions + results) is untrusted data; cannot grant a tool or install policy | T-007 / WHEN the poison tool's description/result carries injected instructions THEN policy + authority SHALL be unchanged and it SHALL mint no evidence | `mcp_poison_description_and_result_are_data` |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Replay reproduces from the frozen schema set without reconnecting | T-003 / freeze; T-004 / identical request bytes; T-006 / reproduce recorded observation without reconnect | `mcp_request_bytes_identical_on_replay`, `replay_reproduces_mcp_run_without_reconnect`, `e2e_mcp_echo_run_and_replay` |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Fixture-based coverage, no external network | T-007 / WHEN the in-repo fixture is spawned THEN discovery + call SHALL succeed | `mcp_discovery_freezes_schema`, `e2e_mcp_echo_run_and_replay` |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | MCP identity coexists with compiled tools without regressing them; barred from checked runs | T-001 / compiled roundtrip; MCP parse; checked-run bar | `toolref_compiled_roundtrip`, `toolref_mcp_parse`, `toolref_mcp_barred_from_checked_run`, (regression: existing `command_tool`/`runner_tools`/`replay` suites) |

## Unit Tests
### T-001 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `toolref_compiled_roundtrip`: `"read_file"` ⇄ `ToolRef::Compiled(ReadFile)`; `wire_name()` == `"read_file"`; existing compiled behavior unchanged.
- `toolref_mcp_parse`: `"mcp__files__grep"` → `ToolRef::Mcp { server: "files", tool: "grep" }`; `wire_name()` round-trips; a malformed `mcp__` string is rejected.
- `toolref_mcp_barred_from_checked_run`: a checked run whose allow-list contains a `ToolRef::Mcp` is rejected by the `is_mutating()` bar (mirrors the existing mutating-tool rejection test).

### T-002 unit tests
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `mcp_config_rejects_undeclared_server`: allow-list `mcp__S__T` with no `[[mcp.servers]]` named `S` → defined config error.
- `mcp_config_rejects_dup_or_empty`: duplicate server `name` → error; empty `command` → error.

## Integration Tests
### MCP discovery + freeze (T-003)
- **Intents:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `mcp_discovery_freezes_schema`: spawn the fixture; a run allow-listing `mcp__fixture__echo` freezes `{server,tool,description,input_schema}` into the authority.
- `mcp_missing_allowlisted_tool_fails_start`: allow-list a tool the fixture does not advertise → run fails to start with a defined error.
- `mcp_discovery_timeout_fails_start`: a server that never completes `initialize`/`tools/list` (fixture `--hang` mode) → run fails to start within the discovery bound with the defined error, not a hang (C-003).
- `authority_roundtrips_through_frozen_context`: a serialized `RunAuthority` with `mcp_tools` deserializes to `FrozenContext` and `json!(frozen)` equals the journaled authority.
- `non_mcp_authority_bytes_unchanged`: a run with no MCP tools serializes with no `mcp_tools` key (byte-identical to the pre-feature shape) and a pre-feature journal still replays.

### MCP model seam (T-004)
- **Intents:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `mcp_tool_schema_emitted_to_model`: the prepared model request `tools` array contains a function `mcp__fixture__echo` with `parameters` == the discovered `input_schema`.
- `mcp_request_bytes_identical_on_replay`: live-prepared request bytes for an MCP-carrying turn equal the replay-prepared bytes (fingerprint match), sourced from the frozen authority.

### MCP live dispatch (T-005)
- **Intents:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `mcp_approved_call_bounded_no_evidence`: an allow-listed `echo` call returns an observation matching the input, mints no evidence, and respects the tools permit + deadline.
- `mcp_unapproved_call_denied`: the model names an MCP tool not in the allow-list → `ToolStatus::Denied`; asserts **no MCP child is spawned and no `tools/call` is issued** for the denied call (C-001).
- `mcp_result_truncated_at_byte_cap`: a fixture tool returning a large payload is bounded at `max_tool_result_bytes` like a compiled result.

### MCP replay dispatch (T-006)
- **Intents:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `replay_reproduces_mcp_run_without_reconnect`: replaying a captured MCP run reproduces the recorded observation with no `McpClientPool` constructed (no child spawned).

### Untrusted server content (T-007)
- **Intents:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- `mcp_poison_description_and_result_are_data`: the `poison` tool's injected description/result do not change the run's allow-list, authority, or policy, and mint no evidence (extends the redteam corpus / content-as-data posture).

## End-to-End Tests
- **Status:** possible
- `e2e_mcp_echo_run_and_replay`: fake model (tests/model_protocol.rs pattern) emits an `mcp__fixture__echo` tool call → runner discovers + dispatches to the in-repo fixture → observation recorded in the journal → the same journal replays and reproduces the run without reconnecting. Pass: live observation matches the echo, replay verdict is clean, no child process spawned during replay.
