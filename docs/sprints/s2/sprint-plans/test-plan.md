Finalized - DO NOT EDIT

# Sprint 2 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | complete groups preserved; no partial group | T-001 / WHEN dropping a group THEN remove assistant + all its results together | `drop_oldest_removes_whole_group` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | oldest droppable unit removed | T-001 / WHEN a droppable unit exists THEN remove the oldest and return true | `drop_oldest_removes_whole_group` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | evidence bytes never dropped | T-001 / WHEN a tool result carries an evidence_id THEN it is not dropped | `drop_oldest_preserves_evidence_bearing_result` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | bounded — protects system, initial turn, recent floor | T-001 / WHEN only protected messages remain THEN return false, drop nothing | `drop_oldest_returns_false_when_only_protected_remain`, `drop_oldest_preserves_system_and_recent_floor` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | explicit policy with a configured ceiling | T-001 / WHEN a Compaction policy is parsed THEN validate the floor | `compaction_policy_validates_floor` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | a run at the limit continues rather than stops | T-002 / WHEN history reaches the limit and compaction frees space THEN compact and continue + journal | `run_continues_past_history_limit_by_compacting`, `test_cli_run_compacts_past_history_limit` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | bounded — still stops when nothing droppable | T-002 / WHEN compaction cannot free space THEN stop with history_bytes_limit | `compaction_stops_when_nothing_droppable` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | replay/immutability contracts hold | T-002 / WHEN replay recomputes a compacted run THEN fingerprints match | `replay_reproduces_a_compacted_run` |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | checked-run acceptance unaffected | T-002 / WHEN a checked run compacts THEN cited evidence remains, verdict unchanged | `checked_run_evidence_survives_compaction` |

## Unit Tests
### T-001 unit tests (`src/core.rs`)
- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- `drop_oldest_removes_whole_group`: a conversation with an old tool-call/result group → one call drops the assistant message and all its tool results together, returns true; no dangling `tool_call_id` remains.
- `drop_oldest_preserves_system_and_recent_floor`: the system message (index 0), the initial turn (index 1), and the most-recent `floor` messages are never dropped.
- `drop_oldest_preserves_evidence_bearing_result`: an old group whose tool result carries an `evidence_id` is skipped; a non-evidence group is dropped instead.
- `drop_oldest_returns_false_when_only_protected_remain`: when everything left is protected, it returns false and mutates nothing.

### T-001 unit tests (`src/config.rs`)
- `compaction_policy_validates_floor`: a positive floor parses; zero or an oversized floor is rejected; `enabled` defaults as specified.

## Integration Tests
### Runner + replay (`tests/runner_journal.rs`)
- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- `run_continues_past_history_limit_by_compacting`: a freeform run with a tiny `max_history_bytes` and several tool-call turns completes (phase not stopped-by-history) and a `history_compacted` event is journalled.
- `compaction_stops_when_nothing_droppable`: with the floor/evidence blocking any drop, a run at the limit still stops with `history_bytes_limit`.
- `checked_run_evidence_survives_compaction`: a checked run that compacts old prose keeps its cited evidence result; acceptance status is unchanged from the non-compacting baseline.

## End-to-End Tests
- **Status:** possible
- `test_cli_run_compacts_past_history_limit` (`tests/cli_inspect.rs`): a real
  `kinesin run … --allow-unchecked` with a tiny `max_history_bytes` against a loopback
  model that returns several tool-call turns completes rather than stopping at the
  limit, and a subprocess `kinesin inspect` shows a `history_compacted` event.

**Replay coverage (`tests/replay.rs`):** `replay_reproduces_a_compacted_run` — a
captured run (replay capture) that compacted replays `consistent`, proving the pure
drop is reproduced and the recorded request fingerprints match. Existing replay
tests must also pass unchanged, since the `history_compacted` event is additive and
old captures never carry it.
