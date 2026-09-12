# INT-0023 — Durable lifecycle and data stewardship

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0023
- **State:** proposed
- **Work evidence:** [T-104 backlog](../work/tasks.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Own the durable operational lifecycle: crash recovery, ambiguous external effects, cancellation/shutdown, retention, quota exhaustion, storage upgrade compatibility and backup/restore. Existing mechanisms are inputs to this outcome, not assumed absent. Non-goal: automatic repetition of effects or silently resuming an interrupted run.

## Acceptance criteria
- Crash checkpoints distinguish never-dispatched, completed and unknown effects; recovery never auto-repeats an unknown effect.
- Run cancellation and controller shutdown settle a defined outcome and join owned processes/work before releasing resources.
- Owner-scoped retention and quota/full-disk outcomes are bounded and preserve required receipts/idempotency semantics.
- A restore drill and supported-version migration matrix demonstrate preservation of owner identity, terminal records and replay compatibility.

## Rationale
Signed history (INT-0014) does not own recovery, retention or restore. These foundational mechanisms need explicit acceptance and recurring evidence.

## Alternatives
Leave dispersed lifecycle tests without an owning contract (rejected); automatic crash resume (excluded until effect reconciliation can prove safety).

## Consequences
Requires recurring operational drills and an explicit compatibility window; sprint 10 inventories existing coverage but does not claim a completed restore/migration program.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.

