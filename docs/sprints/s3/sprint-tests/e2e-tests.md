# Sprint 3 End-to-End Tests

- **Status:** live; **executed** against the pinned server on 2026-09-11.
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- **Tested head:** `326bc505391e58bedda17594cb32f21ec4a4e0fd`

## Live measurement (`tests/live_evaluation.rs`, `#[ignore]`d) — RUN

| Test | Acceptance criterion | Result |
|------|----------------------|--------|
| `kv_cache_reuse_reduces_prompt_eval_time` | measured reduction in prompt-eval time on a shared prefix, recorded with workload and machine | **ok** — second turn evaluated **16 of 59** prompt tokens (43 reused), `prompt_ms=44.158` |

The headline acceptance is a live, machine-specific measurement, so the assertion
lives in an `#[ignore]`d benchmark run against the pinned model on `127.0.0.1:8080`
(the repo's established live-test pattern). It sends two requests that share a
prefix (the second a prefix-extension of the first), both with `cache_prompt`, and
asserts the second evaluates fewer prompt tokens than its full prompt
(`timings.prompt_n < usage.prompt_tokens`). It is not a CI gate, but it **was run**
and passed at the tested head.

### Machine & workload (provenance)
- **Runtime:** llama.cpp **b6500** (commit `a7a98e0f`), official Windows x64 Vulkan.
- **Model:** `qwen2.5-coder-7b-instruct-q4_k_m.gguf` (Q4_K_M, sha256 `509287f7…94d3c`).
- **GPU:** NVIDIA RTX 2080 Ti (11 GiB), all layers offloaded (`-ngl 99`); `-c 4096`, one slot.
- **Launch:** the pinned command from [model-preflight.md](../../../model-preflight.md).
- Raw numbers: `validation-output/kv-cache-measurement/INT-0004-measurement.json` (gitignored, per the repo's live-artifact convention).

### Measured prompt-eval time reduction (cold vs. warm, `cache_prompt` on)
The reduction **scales with the reused prefix size** — the honest headline:

| Shared prefix | Cold (no reuse) | Warm (reuse) | Wall-clock |
|---------------|-----------------|--------------|------------|
| ~55 tokens | 54 tok eval, ~45.9 ms | 25 tok eval, ~45.0 ms | **~1.0×** (overhead-bound) |
| ~2 576 tokens | 2 572 tok eval, ~932.7 ms | 16 tok eval, ~50.7 ms | **~18.4×** (933 ms → 51 ms) |

Both large-prefix rows are the mean of 3 consecutive iterations (stable to <1 ms).
Token-level reduction always holds (the cached prefix is never re-evaluated); the
wall-clock reduction is negligible for trivially short prompts (fixed per-request
overhead dominates) and large where prompt-eval is compute-bound — which is exactly
INT-0004's scaling target: long tool-loop conversations and stable system prefixes.

## CI-verifiable stand-in
The mechanism and every invariant the reduction depends on are also covered offline
(see unit and integration): the request carries `cache_prompt`; each turn's prompt
is a prefix of the next so reuse is valid; the flag changes nothing a run does; and
a `cache_prompt` capture replays consistent. Those gate CI; the measurement above is
the live confirmation of the headline.

## Confirmation
```
running 1 test
KV-cache reuse: second request evaluated 16 of 59 prompt tokens; prompt_ms=44.158
test kv_cache_reuse_reduces_prompt_eval_time ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out
```
