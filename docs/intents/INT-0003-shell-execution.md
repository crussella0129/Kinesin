# INT-0003 — Shell / command execution tool

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** realized
- **Work evidence:** [T-001 build plan](../sprints/s1/sprint-plans/build-plan.md#t-001-config-surface-allow-list-and-checked-run-bar-for-run_command)
- **Completion evidence:** [T-001–T-005 completion log](../work/completed-tasks.md)
- **Code evidence:** [T-001–T-005 commits](../work/completed-tasks.md) plus the run_command E2E
- **Test evidence:** [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md) — all acceptance criteria verified (tested head `13ffdc3`)
- **Documentation evidence:** none

## Intent
Add a bounded command-execution tool, the last and largest member of the write
surface. It is a distinct trust class from a file write: it spawns a process.
Run a command as an **argument vector**, never a shell string, so the model
never controls a shell parser; bound stdout/stderr size; enforce a timeout; do
not auto-retry; record the effect. Grant it per workspace by operator config,
and bar it from checked runs like every mutating tool. Non-goal: an interactive
shell, arbitrary shell interpolation, or unbounded output.

## Acceptance criteria
- A freeform run in a granted workspace runs an allowed command with bounded
  captured output and a timeout, and the effect is journalled.
- A command is always an argv vector; no string is passed to a shell.
- Denied where the workspace grants no command tool; barred in checked runs.
- Failure exercises: timeout, oversized output, non-zero exit, a missing
  executable, and cancellation mid-run each produce a defined outcome.

## Rationale
Real agent tasks often need to run a build, a test, or a formatter. It completes
the tool surface begun with the file-mutation set.

## Alternatives
Keep it deferred (current). A sandboxed code interpreter / CodeAct (heavier
dependency and security surface, considered in the paper review and not chosen).

## Consequences
The largest trust surface in the project; needs its own design pass covering
the allow-list shape, working directory and environment control, process-tree
cleanup, and reconciliation of an effect that cannot be un-run.

## Transition history
- 2026-09-08: created as `proposed`.
- 2026-09-10: `proposed → planned`; selected for sprint 1 and linked to the build plan (T-001–T-005). Design: a distinct `CommandRunner` capability with full process-tree cleanup via `command-group`; cross-platform CI added to exercise it on Windows and Linux.
- 2026-09-10: `planned → active`; sprint 1 build began.
- 2026-09-10: test phase verified all acceptance criteria (18 named tests, 224 green) with an accepted `proceed-with-caveats` critique; test report linked as Test evidence.
- 2026-09-10: `active → realized`; sprint 1 closed with completion, code, and test evidence attached. Follow-ups noted (dedicated per-command timeout; per-event command detail in inspect) as future intents, not gaps.
