# Sprint 2 Unit Tests

- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- **Tested head:** `b22990b94966371a73f27f52836a00b2dec34dcf`
- **Runners:** `cargo test --locked --lib core::tests`, `cargo test --locked --lib config::tests`
- **Result:** all green.

## T-001 — pure drop-oldest compaction (`src/core.rs`)
| Test | EARS clause | Result |
|------|-------------|--------|
| `drop_oldest_removes_whole_group` | a droppable unit exists → remove the oldest and return true; a group's assistant + all its results go together (no dangling `tool_call_id`) | ok |
| `drop_oldest_preserves_system_and_recent_floor` | the system message and the most-recent `floor` messages are never dropped | ok |
| `drop_oldest_preserves_evidence_bearing_result` | a group whose tool result carries an `evidence_id` is skipped; a non-evidence group is dropped instead | ok |
| `drop_oldest_returns_false_when_only_protected_remain` | only protected messages remain → return false, drop nothing (also the all-evidence body) | ok |

## T-001 — compaction policy (`src/config.rs`)
| Test | EARS clause | Result |
|------|-------------|--------|
| `compaction_policy_validates_floor` | a positive floor parses (enabled defaults true); a zero or oversized floor is rejected | ok |

## Confirmation
```
test core::tests::drop_oldest_removes_whole_group ... ok
test core::tests::drop_oldest_returns_false_when_only_protected_remain ... ok
test core::tests::drop_oldest_preserves_system_and_recent_floor ... ok
test core::tests::drop_oldest_preserves_evidence_bearing_result ... ok
test config::tests::compaction_policy_validates_floor ... ok
```

The drop logic is pure (a `RunState` method), so both the runner and replay invoke
it identically — the property the replay test relies on.
