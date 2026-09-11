# Sprint 3 Test Report — KV-cache reuse (INT-0004)

- **Tested head:** `326bc505391e58bedda17594cb32f21ec4a4e0fd`
- **Toolchain:** pinned 1.96.0 (rust-toolchain.toml)
- **Commands:** `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked`
- **Suite:** **241 passed / 0 failed / 7 ignored**; fmt + clippy clean.
- **Critic verdict:** `proceed-with-caveats` (see [critique.md](critique.md)).

## Intent acceptance → evidence

INT-0004's acceptance criteria and where each is verified:

| Acceptance criterion | Verification | Layer |
|----------------------|--------------|-------|
| The immutable-run / pure-replay contract is preserved | `replay_reproduces_a_cache_prompt_run` + the unchanged 241-test suite; `cache_prompt` is stored in the frozen `ModelConfig`, so both sides run the same `prepare` | integration/replay (CI) |
| Prefix reuse never corrupts output | `cache_prompt_does_not_change_run_outcome` (flag is additive/inert offline) + `prepared_request_of_each_turn_extends_the_previous` (reuse is over a genuine byte-prefix) + llama.cpp reusing only byte-identical tokens | integration (CI) + server semantics |
| Measured reduction in prompt-eval time on a shared prefix, recorded with workload and machine | `kv_cache_reuse_reduces_prompt_eval_time` **run** against the pinned b6500 server on 2026-09-11: second turn evaluated **16 of 59** tokens; a ~2 576-token prefix cut prompt-eval **933 ms → 51 ms (~18.4×)**. Provenance + numbers in [e2e-tests.md](e2e-tests.md). | e2e (live, executed) |

## Named tests (all present at the tested head)

**Unit — `cargo test --locked --lib`**
- `model::tests::cache_prompt_present_when_enabled` — ok
- `model::tests::cache_prompt_absent_when_disabled` — ok
- `config::tests::cache_prompt_defaults_on` — ok

**Integration — `runner_tools`, `replay`**
- `prepared_request_of_each_turn_extends_the_previous` — ok
- `cache_prompt_does_not_change_run_outcome` — ok
- `replay_reproduces_a_cache_prompt_run` — ok

**E2E — live, `#[ignore]`d (`live_evaluation`), executed 2026-09-11**
- `kv_cache_reuse_reduces_prompt_eval_time` — **ok** against the pinned b6500 server; token and time reductions recorded in [e2e-tests.md](e2e-tests.md).

## Caveats carried forward (from the critic)
- **C-001 (resolved):** the headline measurement was **performed**, not merely deferred — the ignored benchmark was run against the pinned server and passed, and the cold-vs-warm prompt-eval time reduction was measured with workload and machine (see e2e-tests.md). Remains outside CI (inherently machine-specific), but is no longer an unverified claim. Honest nuance recorded: the wall-clock reduction scales with prefix size (≈1.0× at ~55 tokens, ≈18.4× at ~2.6k tokens); the token-level reduction always holds.
- **C-002 (accepted):** the static `*.request.json` captures were left as dated provider records (no offline test loads them; only `.sse` responses are `include_bytes!`'d). The verifiable exact-bytes assertions in `model_protocol.rs` were updated in lockstep; re-recording is a live-capture follow-up.
- **C-003 / C-004 (layered / deferred):** offline cannot prove a server-side optimization end-to-end; the guarantee is split across offline invariants + server semantics + the now-executed live benchmark. The prefix property under compaction is covered by sprint 2's compaction tests (compaction truncating the prefix is by design and non-corrupting).

## Verdict
Test phase satisfied for INT-0004: every acceptance criterion maps to a named, present test; the CI-verifiable set is green on the pinned toolchain; and the headline live measurement was executed against the pinned server (18.4× prompt-eval reduction on a ~2.6k-token shared prefix). Proceed to loop.
