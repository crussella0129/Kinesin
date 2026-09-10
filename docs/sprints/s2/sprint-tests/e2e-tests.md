# Sprint 2 End-to-End Tests

- **Status:** possible — executed.
- **Intent:** [INT-0002](../../../intents/INT-0002-context-compaction.md)
- **Tested head:** `b22990b94966371a73f27f52836a00b2dec34dcf`
- **Runner:** `cargo test --locked --test cli_inspect test_cli_run_compacts_past_history_limit`
- **Result:** green.

## Executed test (`tests/cli_inspect.rs`)
| Test | Acceptance criterion | Result |
|------|----------------------|--------|
| `test_cli_run_compacts_past_history_limit` | a run whose history reaches the limit continues under the bounded policy rather than stopping | ok |

## What it drives, end to end
A real `kinesin run … --workspace practice --model local --prompt … --allow-unchecked`
subprocess runs against a loopback HTTP model with a tiny `max_history_bytes = 2600`
and `[limits.compaction] floor = 2`. The loopback returns three bulky, distinct
`list_files` tool-call turns (which overflow the budget) then a final answer. The run
**completes** (`phase == "completed"`) rather than stopping with `history_bytes_limit`,
and a subprocess `kinesin inspect --run <id>` shows `counters.compactions >= 1`.

This exercises the full path INT-0002 names: real CLI → real HTTP model → runner
compaction loop → SQLite persistence → read-back through `inspect`.

## Confirmation
```
test test_cli_run_compacts_past_history_limit ... ok
```

## Replay and immutability
Covered by `replay_reproduces_a_compacted_run` (see integration): a captured
compacting run replays `consistent`, proving replay applies the identical
deterministic drops and the immutability/replay contract holds. Existing replay
tests pass unchanged — the `compactions` counter is additive and absent from old
captures, and replay's counter-divergence check now compares only the
deterministically recomputable `model_turns`/`tool_calls`.
