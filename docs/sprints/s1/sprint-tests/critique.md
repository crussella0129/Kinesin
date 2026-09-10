# Test Critique — Sprint 1

Adversarial read-only screen of the run_command evidence against the locked plans
and INT-0003's acceptance criteria.

## Concerns

### C-001: T-005's CI EARS clause has no cargo test
- **Where:** `build-plan.md` T-005 / `e2e-tests.md` CI matrix.
- **Quote:** "WHEN CI runs … THEN it SHALL run fmt, clippy, and `cargo test --locked` on both windows-latest and ubuntu-latest."
- **Failure mode:** plan-test-mismatch / e2e-drift.
- **Why it matters:** every other EARS clause maps to a named cargo test; this one is verified by CI configuration and the checkpoint's dual-OS run.
- **Suggested response:** defer-with-rationale. A workflow matrix is configuration, not library behavior, so it has no unit test by nature. Its verification is explicit and checkable: the workflow names both OSes and both jobs must be green at the checkpoint (recorded in the test report's CI Confirmation). Accepted as a declared non-unit verification.

### C-002: two integration tests are timing-based
- **Where:** `tests/command_tool.rs::command_timeout_kills_process_tree` and `tests/runner_tools.rs::command_cancelled_midrun_is_killed`.
- **Quote:** timeout `700ms` then observe for `600ms`; cancel at `300ms` bounded by `timeout(8s)`.
- **Failure mode:** flake-risk.
- **Suggested response:** defer-with-rationale. The margins are deliberately large relative to what they discriminate: the tree-kill test compares the marker size across a 600ms window after the group is killed (a surviving grandchild appends every 50ms, so ~12 writes would be missed if kill failed); the cancellation test bounds settlement at 8s against a 60s command sleep and a 10s run deadline, so cancellation is unambiguous. These are outcome comparisons with wide separation, not tight races.

### C-003: the E2E surfaces the specific effect via the stored journal, not inspect output
- **Where:** `tests/cli_inspect.rs::test_cli_run_command_effect_is_journalled`; `test-plan.md` End-to-End.
- **Quote:** test-plan said "a subprocess `kinesin inspect` shows the run_command effect journalled with its exit code."
- **Failure mode:** e2e-drift.
- **Why it matters:** `inspect`'s event summaries drop event bodies (established in sprint 0), so the tool name and exit code cannot appear in `inspect` output.
- **Suggested response:** reject (the critique overreaches) with a note. INT-0003's acceptance criterion is that the effect is *journalled* (AC1), which the E2E verifies directly against the stored journal (`dispatch: executed`, `exit_code: 0`, `stdout: "hi"`), through a real `kinesin run` and real process spawn. `inspect` is additionally asserted to surface the run via `counters.tool_calls == 1`. The intent criterion is fully met; only the plan's optimistic inspect wording is refined by how inspect actually behaves. Surfacing per-event command detail through inspect is out of scope.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — all four acceptance criteria (runs with bounded output + timeout + journalled; always argv/no shell; denied without grant + barred in checked; each of the five failure modes) map to named executed tests.
- **Assertion weakness:** none — tests assert exit codes, truncation length equal to the cap, dispatch/classification values, and a marker that stops growing after kill.
- **Stub leakage:** none — the fixture is a real argv-driven binary; the E2E uses a real binary, real HTTP, real spawn, real SQLite.
- **Integration drift:** none — `command_tool.rs` covers the capability, `runner_tools.rs` the dispatch/journal integration; distinct boundaries.
- **Negative-path absence:** none — denial, checked bar, timeout, oversized output, missing executable, and cancellation each have a negative test.

## Confidence
proceed-with-caveats
