# INT-0002 — Context compaction

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Keep a long run or session going when its conversation approaches
`max_history_bytes`, instead of stopping. Today the run stops with
`history_bytes_limit`; that was acceptable when every run was one shot, but the
interactive session (each turn a new run citing the prior answer) makes a long
thread die. Apply a bounded, evidence-preserving strategy — drop-oldest or a
summary of older turns — under an explicit policy. Non-goal: unbounded history,
or any change that lets a checked run's evidence be silently altered.

## Acceptance criteria
- A run whose history reaches the limit continues under a bounded policy rather
  than stopping, up to a configured ceiling.
- Complete tool call/result groups are preserved; a partial group is never left.
- Checked-run acceptance is unaffected: evidence bytes are never dropped or
  rewritten in a way that changes a verdict.
- The immutability and replay contracts still hold.
- Tests cover the boundary, the preserved-group rule, and the checked-run case.

## Rationale
Sessions exist now, so a thread that outgrows the budget must degrade
gracefully rather than terminate. This was the earliest-flagged gap and is the
most urgent of the open items for interactive use.

## Alternatives
Keep stopping at the limit (current; poor for sessions). File-pointer
(programmatic) tool results were considered and rejected earlier because weak
local backbones close the read-then-integrate loop least reliably.

## Consequences
Changes conversation assembly; requires care to preserve immutability, replay,
and the checked-run evidence contract; a summary strategy adds a model call.

## Transition history
- 2026-09-08: created as `proposed`.
