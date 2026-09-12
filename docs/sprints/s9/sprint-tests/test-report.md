# Sprint 9 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Allow-listed MCP tools run under the same per-run gates and mint no evidence | T-005 / `mcp_approved_call_bounded_no_evidence`, `mcp_result_truncated_at_byte_cap` | pass | Test evidence links this report |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Two-gate approval: undeclared server denied; un-allow-listed tool denied | T-002 / `mcp_config_rejects_undeclared_server`; T-005 / `mcp_unapproved_call_denied` | pass | " |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Server text (descriptions + results) is untrusted data; grants no tool / policy | T-007 / `mcp_poison_description_and_result_are_data` | pass | " |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Replay reproduces from the frozen schema set without reconnecting | T-004 / `mcp_tool_schema_emitted_to_model`; T-006 / `e2e_mcp_echo_run_and_replay` | pass | " |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | Fixture-based coverage: approved, denied, untrusted, replay (no network) | T-007 / all of `tests/mcp.rs`; T-003 / `mcp_missing_allowlisted_tool_fails_start`, `mcp_discovery_timeout_fails_start` | pass | " |
| [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) | MCP identity coexists with compiled tools; barred from checked runs | T-001 / `toolref_compiled_roundtrip`, `toolref_mcp_parse`, `toolref_mcp_barred_from_checked_run` (+ full pre-existing suite regression) | pass | " |

## Summary
- Unit tests: 155 passed / 0 failed / 155 total (`cargo test --lib`; includes the 10 MCP unit tests + full regression)
- Integration tests: 7 passed / 0 failed / 7 total (`tests/mcp.rs`, fixture-driven)
- E2E tests: 1 passed / 0 failed / 1 total (`e2e_mcp_echo_run_and_replay`)
- CI status: green

## CI Confirmation
- **Head SHA:** `a2b3bcee8273c8caeab0bb1631cb9c71d2913adc`
- **CI run:** 34676700936 — https://github.com/crussella0129/Kinesin/actions/runs/34676700936
- **Conclusion:** success
- **Confirmations:** `check (windows-latest)` and `check (ubuntu-latest)` — fmt + clippy(all-targets) + `cargo test` (incl. `tests/mcp.rs` and the fixture bin) green on both OSes; `supply-chain` (ubuntu) — `cargo deny check` (advisories/licenses/bans/sources) + `cargo audit` green (262 deps, 0 vulns; the rmcp `server`/`transport-io`/`macros` features and the direct `schemars` dep add no new crates).

## Failures
None.

## Technical Debt Identified
- [INT-0020](../../../intents/INT-0020-remote-mcp-delegated-auth.md) — remote (HTTP) MCP transport + OAuth resource-server auth (RFC 8707, no token passthrough); split from INT-0005.
- Full JSON-Schema argument validation (the MVP does a lightweight object + required-keys check — critique C-002 in the plan phase); sandboxing a not-fully-trusted MCP server process; MCP resources/prompts/sampling — named follow-ups.

## Coverage Observations
- Every INT-0005 acceptance criterion and every locked EARS clause maps to a named executed test (see `unit/integration/e2e-tests.md`). Two accepted defers are recorded in `critique.md` (proceed-with-caveats): the "no server contacted on denial" guarantee is proven structurally rather than by observing the absent RPC, and a few executed test names consolidate the plan's illustrative names.
- Environmental note (not a test failure): on the local Windows box a full-parallel `cargo test` intermittently hit `link.exe` errors caused by a full disk; reclaiming space resolved it, and CI (clean runners) is green on both OSes.
