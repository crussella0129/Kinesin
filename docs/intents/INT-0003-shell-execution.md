# INT-0003 — Shell / command execution tool

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
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
