# Sprint 10 Meta

- **Sprint number:** 10
- **Book schema version:** 2
- **Start timestamp:** 2026-09-12T14:43:16Z
- **End timestamp:** 2026-09-12T16:36:18Z
- **Model:** unknown
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** unknown (not exposed by this host)
- **Summary:** Audit harness intents and repair execution, isolation, replay and transport contracts
- **Intents:** [INT-0021](../../intents/INT-0021-harness-contract-review.md), [INT-0022](../../intents/INT-0022-completed-contract-repairs.md) — realized; remaining capability chapters stay proposed.
- **Completion evidence:** INT-0021 and INT-0022 realized through all-intent audit and completed-contract repairs; accepted clean TEST critique, Windows303/Ubuntu315 CI on f3e3d0f, dependency gates, actual nighthawk serving/cache observations and disconnected pure replay; limits and unscheduled follow-ups preserved in the Book.

## Build integration notes

Independent command and MCP transport tasks ran while the core task retained
exclusive ownership of runner/replay. T-007 follows that committed core boundary.
Temporary compilation gaps while shared APIs changed were not recorded as
passing task checks; each task retains a verified coherent commit boundary.

The operator supplied a second physical host during Build and established
Tailscale SSH. A dedicated pinned model server was staged on nighthawk, with
loopback-only binding and an authenticated SSH local forward. This unlocks a
scoped real deployment measurement; INT-0026/0027 remain proposed until all
their broader acceptance criteria are met.

Independent review during integration identified a Unix process-ID reuse race
and a service-verifier path that could overlap Linux runtime read grants.
T-011's explicit repair-and-retest clause owns these additional contract fixes.
The live cache probe also showed that omitting cache_prompt is not a cache-off
control on b6500; the measurement reports observed reuse without claiming a
causal speedup from the flag.

## Loop reconciliation

The completed task ledger, [accepted TEST report](sprint-tests/test-report.md),
[clean independent critique](sprint-tests/critique.md) and
[canonical verification record](sprint-tests/verification-record.md) satisfy the
two advanced intents. Windows 303 and Ubuntu 315 ordinary tests passed in CI,
with format/clippy and dependency gates. Additional actual nighthawk serving,
session-cache observation and disconnected replay passed within their recorded
scope; model/tunnel cleanup was verified. Failed local WSL/storage attempts and
the later recovered Ubuntu startup are preserved as separate observations.

Seven executable carry-forward entries remain unscheduled backlog, linked to
their proposed owning intents. Historical supersessions and broader Windows,
session, lifecycle, credential, evaluation and deployment gaps remain explicit.
The confidence throttle uses `patched`: integration review required additional
contract fixes before the accepted final report. The final submitted-head checks
are retained by the PR's check suite; the merge policy remains human-approve.
