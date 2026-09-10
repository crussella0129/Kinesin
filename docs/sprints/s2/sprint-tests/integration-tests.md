# Sprint 2 Integration Tests

- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- **Tested head:** `b22990b94966371a73f27f52836a00b2dec34dcf`
- **Runners:** `cargo test --locked --test runner_tools`, `cargo test --locked --test replay`
- **Result:** all green.

## Runner integration (`tests/runner_tools.rs`)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `run_continues_past_history_limit_by_compacting` | history reaches the limit and compaction frees space → compact and continue; run completes with `compactions >= 1` in the terminal counters | ok |
| `compaction_stops_when_nothing_droppable` | a single oversized recent turn can't be compacted → the run stops with `history_bytes_limit` and no `compactions` counter | ok |
| `checked_run_evidence_survives_compaction` | a checked run compacts old non-evidence list turns while its cited read evidence is preserved → acceptance is `passed` (unchanged) with `compactions >= 1` | ok |

The integration tests use `list_files` groups (non-evidence, droppable) with distinct
paths to avoid the repeat detector, and bulky assistant content to overflow a small
`max_history_bytes`. The checked test additionally proves the evidence contract: the
read result is preserved and the candidate still cites `e0`.

> Location note: the plan named `tests/runner_journal.rs`; these live in
> `tests/runner_tools.rs`, which already carries the checked-run + read/list
> scaffolding (`run_case`, `checked`, `read`, `batch`) they reuse.

## Replay (`tests/replay.rs`)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `replay_reproduces_a_compacted_run` | replay applies the identical drops so a compacted capture replays `consistent` (fingerprints match) and `completed` | ok |

## Confirmation
```
test run_continues_past_history_limit_by_compacting ... ok
test compaction_stops_when_nothing_droppable ... ok
test checked_run_evidence_survives_compaction ... ok
test replay_reproduces_a_compacted_run ... ok
```

Determinism: scripted clients with fixed content and distinct calls; the replay test
captures a real compacting run (capture=replay) and asserts `report.consistency ==
"consistent"`, which would fail if replay did not reproduce the compaction.
