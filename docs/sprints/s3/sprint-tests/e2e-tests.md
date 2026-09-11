# Sprint 3 End-to-End Tests

- **Status:** possible only against a live server.
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- **Tested head:** `326bc505391e58bedda17594cb32f21ec4a4e0fd`

## Live measurement (`tests/live_evaluation.rs`, `#[ignore]`d)
| Test | Acceptance criterion | Result |
|------|----------------------|--------|
| `kv_cache_reuse_reduces_prompt_eval_time` | measured reduction in prompt-eval time on a shared prefix, recorded with workload and machine | ignored in CI; run manually against the pinned server |

The headline acceptance is inherently a live, machine-specific measurement, so it
is an `#[ignore]`d benchmark requiring the manually started pinned model on
`127.0.0.1:8080` — the repo's established live-test pattern. It sends two requests
that share a prefix (the second a prefix-extension of the first), both with
`cache_prompt`, and asserts the second evaluates fewer prompt tokens than its full
prompt (`timings.prompt_n < usage.prompt_tokens`), recording `prompt_ms` and the
token counts. The test is present and compiles (`--ignored --list` shows it); it is
not a CI gate.

## CI-verifiable stand-in
The mechanism and every invariant the reduction depends on are covered offline (see
unit and integration): the request carries `cache_prompt`; each turn's prompt is a
prefix of the next so reuse is valid; the flag changes nothing a run does; and a
`cache_prompt` capture replays consistent. This is the same non-unit-verification
split the CI matrix used in sprint 1.

## Confirmation
```
kv_cache_reuse_reduces_prompt_eval_time: test   (from --ignored --list)
```
