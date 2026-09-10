# Sprint 2 Build Plan

## Intents
- [INT-0002](../../../intents/INT-0002-context-compaction.md) — state: planned; acceptance criteria covered: a run at the limit continues under a bounded policy rather than stopping; complete tool-call/result groups preserved (no partial group); checked-run acceptance unaffected (evidence never dropped/rewritten); immutability and replay contracts still hold.

## Schema Tree
- Sprint Goal: continue past `max_history_bytes` by bounded, evidence-preserving compaction
  - Pure core logic
    - T-001: `RunState::drop_oldest_compactable` + `Compaction` config policy
  - Runtime + replay integration
    - T-002: compact at the limit in the runner and replay; journal the event

## Execution Sequence

### T-001: Pure drop-oldest compaction in core and the compaction policy
- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- **Touches:** src/core.rs, src/config.rs
- **Depends on:** (none)
- **Acceptance criterion:** complete tool-call/result groups are preserved and a
  partial group is never left; evidence bytes are never dropped.
- **Success criterion (EARS):**
  - **WHEN** a droppable unit exists outside the protected set, **THEN** `drop_oldest_compactable` **SHALL** remove the oldest such unit and return true.
  - **WHEN** it drops a tool-call/result group, **THEN** it **SHALL** remove the assistant `tool_calls` message and all its tool results together.
  - **WHEN** a tool result carries an `evidence_id`, **THEN** it **SHALL NOT** be dropped.
  - **WHEN** only protected messages remain (system, initial turn, recent floor, evidence-bearing results), **THEN** it **SHALL** return false and drop nothing.
  - **WHEN** a `Compaction` policy is parsed, **THEN** validation **SHALL** accept a positive floor and reject a zero or oversized one.
- **Notes:** the drop logic is pure and lives in `core` (where `Message`/`RunState`
  live); the byte measurement stays in the caller. Protected messages: index 0
  (system), index 1 (initial task/prompt), the most-recent `floor` messages, and any
  evidence-bearing tool result with its group. Config `Compaction { enabled, floor }`
  sits beside `max_history_bytes`.

### T-002: Compact at the limit in the runner and replay, journal the event
- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- **Touches:** src/runner.rs, src/replay.rs
- **Depends on:** T-001
- **Acceptance criterion:** a run whose history reaches the limit continues under the
  bounded policy up to a configured ceiling rather than stopping; the immutability
  and replay contracts still hold; checked-run acceptance is unaffected.
- **Success criterion (EARS):**
  - **WHEN** a run's history reaches the limit and compaction can free space, **THEN** the runner **SHALL** compact and continue rather than stop, and **SHALL** journal a `history_compacted` event.
  - **WHEN** compaction cannot free enough space without violating a preservation rule, **THEN** the runner **SHALL** stop with `history_bytes_limit`.
  - **WHEN** replay recomputes a run that compacted, **THEN** it **SHALL** apply the identical drops so the recomputed request fingerprints match the recorded ones.
  - **WHEN** a checked run compacts, **THEN** cited evidence remains and acceptance is unchanged.
- **Notes:** replace the three `runner.rs` stop sites (l.691, l.861, l.1147) and the
  mirrored `replay.rs` sites (l.587, l.742, l.762) with a bounded compaction loop
  using the existing `serde_json::to_vec(&conversation_json(...)).len()` measure. The
  runner journals `history_compacted` (dropped count, bytes before/after); replay
  applies the identical pure drops and journals nothing. The added event is additive,
  so replay of older captures is unaffected.
