# Sprint 9 E2E Tests

**Status:** possible — executed.

## `e2e_mcp_echo_run_and_replay` (`tests/mcp.rs`) — T-004 + T-006
A fake (scripted) model emits an `mcp__fixture__echo` tool call; the runner
discovers the fixture server, dispatches the call over its stdio session, records
the observation, and settles a terminal record. The captured journal is then
replayed with `replay(&run, &events)`:

- **Result:** `report.consistency == "consistent"`, `report.tool_observations >= 1`.

This single E2E proves two locked promises at once:
- **T-004 (identical request bytes on replay):** replay recomputes each
  prepared-request's sha256 fingerprint from the **frozen** authority (which
  carries the discovered MCP schema) and reports `consistent` only when every
  fingerprint matches the recorded one. A consistent verdict is the byte-identity
  proof — the MCP tool schema emitted live is reproduced exactly on replay.
- **T-006 (reproduce without reconnecting):** replay constructs no
  `McpClientPool` and spawns no server; it re-validates the call against the
  frozen discovered set and replays the recorded observation. The run reproduces
  from data alone.

## Result
`cargo test --test mcp e2e_mcp_echo_run_and_replay` → **1 passed / 0 failed**.
