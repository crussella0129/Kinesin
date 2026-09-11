# Sprint 3 Integration Tests

- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- **Tested head:** `326bc505391e58bedda17594cb32f21ec4a4e0fd`
- **Runners:** `cargo test --locked --test runner_tools`, `cargo test --locked --test replay`
- **Result:** all green.

## Reuse invariants (`tests/runner_tools.rs`)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `prepared_request_of_each_turn_extends_the_previous` | each turn's prepared `messages` list is a prefix of the next turn's (and every request carries `cache_prompt`) | ok |
| `cache_prompt_does_not_change_run_outcome` | the same scripted run on vs off yields the identical candidate and acceptance | ok |

The prefix-extension test is the offline proof that the server's prefix reuse is
valid: within a run the conversation only grows by appending, so turn K's request
is a byte-prefix of turn K+1's. The outcome-invariance test shows the flag is a
transparent, additive optimization.

## Replay (`tests/replay.rs`)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `replay_reproduces_a_cache_prompt_run` | a capture made with `cache_prompt` (default on) replays `consistent`, with matching request fingerprints | ok |

## Confirmation
```
test prepared_request_of_each_turn_extends_the_previous ... ok
test cache_prompt_does_not_change_run_outcome ... ok
test replay_reproduces_a_cache_prompt_run ... ok
```

Existing replay/model tests pass unchanged: `cache_prompt` is on by default, so the
dynamic replay harness and the byte-assertion tests already exercise it, and both
sides run the same `prepare`.
