# Sprint 9 Integration Tests

Executed by `cargo test --test mcp` (Windows). All pass. Each drives a real run
through the runner against the in-repo `mcp-fixture` stdio server via the
scripted-model harness — the actual rmcp handshake, discovery, and `tools/call`
over a spawned child process, no external server or network.

## `tests/mcp.rs`
- `mcp_approved_call_bounded_no_evidence` (T-003, T-005) — a run granting `mcp__fixture__echo` discovers and **freezes** the tool into the authority (`mcp_tools().len()==1`, tool `echo`), then the model's call **executes** over the session, the observation equals the echoed input, and it **mints no evidence** (`evidence_id == null`). Proves the approved-call gate + discovery-freeze + no-evidence.
- `mcp_tool_schema_emitted_to_model` (T-004) — the captured model request's `tools` array offers a function named `mcp__fixture__echo` whose `parameters` are the server-**discovered** `inputSchema` verbatim (an object with a `text` property), and equal the frozen `mcp_tools[0].input_schema`. Directly proves discovery reaches the model request.
- `mcp_result_truncated_at_byte_cap` (T-005) — with a 256-byte result envelope and a 400-byte echo, the observation body is ≤256 bytes, `truncated == true`, and the event's `complete == false`. Proves the byte bound.
- `mcp_unapproved_call_denied` (T-005, C-001) — a run granting only `echo`; the model calls `mcp__fixture__poison` (not granted). Dispatch is `denied`, status `denied`, code `tool_denied` — decided against the frozen set before any `tools/call`. Proves the negative authority path.
- `mcp_poison_description_and_result_are_data` (T-007) — a run granting `poison`; its injected description and result are recorded verbatim as observation **data** (`body` contains the injection), mint no evidence, the post-run allow-list is exactly what the operator set (the server text granted no tool), and the injected "create owned.txt" instruction had no effect (no `owned.txt`). Proves untrusted-content-as-data.
- `mcp_missing_allowlisted_tool_fails_start` (T-003) — allow-listing a tool the fixture does not advertise fails run start with `mcp_tool_absent`. Proves the discovery completeness gate (negative path).
- `mcp_discovery_timeout_fails_start` (T-003, C-003) — the fixture in `--hang` mode never completes discovery; `connect` fails with `mcp_initialize_timeout` within a 2s bound rather than hanging. Proves bounded discovery.

## Intent/EARS coverage note
- Discovery-freeze (T-003 EARS 1) is proven by `mcp_approved_call_bounded_no_evidence` observing the frozen `mcp_tools`, and its round-trip through `FrozenContext` is proven by the E2E replay (which deserializes the frozen authority *with* `mcp_tools` and reproduces consistently).
- Schema emission (T-004 EARS 1) is proven directly by `mcp_tool_schema_emitted_to_model`; the identical-bytes-on-replay half (T-004 EARS 2) by the E2E replay's consistent verdict.

## Result
`cargo test --test mcp` → **8 passed / 0 failed** (7 integration + 1 E2E below). No flake surface: the fixture is in-repo and spawned per test; discovery/call are bounded by explicit timeouts; the `--hang` test uses a 2s bound.
