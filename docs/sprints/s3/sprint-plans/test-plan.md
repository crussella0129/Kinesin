Finalized - DO NOT EDIT

# Sprint 3 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | request reuses the cached prefix | T-001 / WHEN cache_prompt enabled THEN body includes cache_prompt | `cache_prompt_present_when_enabled` |
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | reuse is opt-outable | T-001 / WHEN disabled THEN body omits cache_prompt | `cache_prompt_absent_when_disabled` |
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | explicit policy, default on | T-001 / WHEN config omits it THEN default enabled | `cache_prompt_defaults_on` |
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | reuse is valid (prefix stability) | T-002 / WHEN successive turns THEN each request is a prefix of the next | `prepared_request_of_each_turn_extends_the_previous` |
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | reuse never corrupts a run | T-002 / WHEN on vs off THEN candidate + acceptance identical | `cache_prompt_does_not_change_run_outcome` |
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | immutability/replay preserved | T-002 / WHEN a cache_prompt capture is replayed THEN consistent | `replay_reproduces_a_cache_prompt_run` |
| [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) | measured prompt-eval reduction | T-002 / WHEN live benchmark runs THEN reduced prompt_ms on the reused prefix | `kv_cache_reuse_reduces_prompt_eval_time` (live, `#[ignore]`) |

## Unit Tests
### T-001 unit tests (`src/model.rs`, `src/config.rs`)
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- `cache_prompt_present_when_enabled`: `prepare` with the flag on → body has `cache_prompt == true`.
- `cache_prompt_absent_when_disabled`: `prepare` with the flag off → body has no `cache_prompt` key.
- `cache_prompt_defaults_on`: a `ModelConfig` parsed without the field → `cache_prompt == true`.

## Integration Tests
### Reuse invariants (`tests/model_protocol.rs`, `tests/runner_journal.rs`)
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- `prepared_request_of_each_turn_extends_the_previous`: across a scripted multi-turn tool-loop run, capture each prepared request; the `messages` array of turn K is a prefix of turn K+1's (no compaction in this fixture).
- `cache_prompt_does_not_change_run_outcome`: the same scripted run with `cache_prompt` enabled and disabled yields the identical candidate and acceptance status (the additive flag changes nothing the run does).

## Replay (`tests/replay.rs`)
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- `replay_reproduces_a_cache_prompt_run`: a dynamic replay capture made with `cache_prompt` replays `consistent`, proving the recorded and recomputed request fingerprints match. Existing replay tests must pass unchanged (they self-capture with the same `prepare`).

## End-to-End Tests
- **Status:** possible only against a live server.
- `kv_cache_reuse_reduces_prompt_eval_time` (`tests/live_evaluation.rs`, `#[ignore]`d,
  requires the manually started pinned model on 127.0.0.1:8080): run consecutive
  tool-loop turns and record the reduction in `timings.prompt_ms` on the reused
  prefix, with workload and machine. This is the headline measurement, verified
  manually; the offline invariants above are the CI-verifiable stand-in — the same
  non-unit verification the CI matrix used.

**Fixtures/immutability:** T-001 updates the static request fixtures
(`tests/fixtures/live/*.request.json`) and the exact-request-bytes assertions in
`model_protocol.rs`; those tests passing confirms the request-shape change is
recorded consistently.
