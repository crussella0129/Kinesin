# Test Critique — Sprint 9

## Concerns

### C-001: "denied without contacting a server" is proven structurally, not by observing the absence of an RPC
- **Where:** `integration-tests.md` / `mcp_unapproved_call_denied` (T-005, INT-0005 AC2)
- **Quote:** "Dispatch is `denied` … decided against the frozen set before any `tools/call`."
- **Failure mode:** weak-assertion
- **Why it matters:** the test asserts the denial classification (`dispatch == "denied"`, code `tool_denied`), but does not observe that no `tools/call` was sent to the server for the denied call.
- **Suggested response:** defer-with-rationale — the denial is computed in the runner's denial ladder *before* the tools permit is acquired and *before* `McpClientPool::call` is reached (a well-formed but un-granted `mcp__` name resolves to `tool_denied` from the frozen set with no dispatch). The recorded observation (`dispatch: denied`, no execution) confirms no call ran. Asserting the negative on the wire would need RPC-level instrumentation the fixture does not expose; the structural guarantee plus the classification is sufficient for the MVP.

### C-002: some executed test names differ from the illustrative names in the locked test plan
- **Where:** `test-plan.md` (e.g. `mcp_request_bytes_identical_on_replay`, `replay_reproduces_mcp_run_without_reconnect`, `authority_roundtrips_through_frozen_context`)
- **Quote:** "`mcp_request_bytes_identical_on_replay` (integration)"
- **Failure mode:** evidence-drift
- **Why it matters:** a reader tracing the plan's named tests to executed tests finds them consolidated rather than one-to-one.
- **Suggested response:** defer-with-rationale — every locked EARS clause and every INT-0005 acceptance criterion maps to a named executed test (see the coverage notes in `integration-tests.md` / `e2e-tests.md`): the byte-identical-on-replay and reproduce-without-reconnect clauses are both proven by `e2e_mcp_echo_run_and_replay` (replay reports `consistent` only when every prepared-request fingerprint matches, and constructs no client), and the frozen-authority round-trip by the same E2E deserializing the frozen `mcp_tools`. The consolidation strengthens rather than weakens coverage; the names are documented.

## Confidence
proceed-with-caveats

Both concerns are accepted defers with the rationale above; no intent criterion
or EARS promise is left unproved. CI confirmation on both OSes is recorded in
`test-report.md` before INT-0005 is marked realized.
