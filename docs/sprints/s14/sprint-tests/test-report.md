# Sprint 14 Failure Assessment

**Result: failed.** All eight scored fresh live attempts failed the independent
usability gate. This is a failure assessment, not a passing verification report.

| Intent criterion | Disposition |
| --- | --- |
| INT-0032 AC1 recovery/review | Implemented experimentally; formal verification not performed; self-review did not establish usable work. |
| AC2 bounds/authority/replay | Source reviewed and compiled; required official regression verification deferred. |
| AC2a defaults/tool guidance | Implemented and compiled; no demonstrated end-to-end usefulness. |
| AC3 fresh app plus natural follow-up | Failed: 0 of 8 accepted attempts. |
| AC4 evidence/live-first verification | Failed attempts retained; prerequisite for official unit/integration verification never passed. |

Official unit tests: not run. Official integration tests: not run. Clippy: not
run for Sprint 14. Compilation/formatting are recorded in the
[implementation history](../sprint-research/implementation-adjustment.md) and do
not qualify the workload. Earlier Sprint 13 checks are not Sprint 14 evidence.

See the [failure report](../failure-report.md) for root-cause distinctions,
unresolved hypotheses, task disposition and the next research obligations;
the [live scorecard](e2e-tests.md) owns detailed results and cost.
