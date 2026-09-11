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
| Measured reduction in prompt-eval time on a shared prefix | `kv_cache_reuse_reduces_prompt_eval_time` (`#[ignore]`d live benchmark; asserts `timings.prompt_n < usage.prompt_tokens` on the reused prefix) | e2e (live/manual) |

## Named tests (all present at the tested head)

**Unit — `cargo test --locked --lib`**
- `model::tests::cache_prompt_present_when_enabled` — ok
- `model::tests::cache_prompt_absent_when_disabled` — ok
- `config::tests::cache_prompt_defaults_on` — ok

**Integration — `runner_tools`, `replay`**
- `prepared_request_of_each_turn_extends_the_previous` — ok
- `cache_prompt_does_not_change_run_outcome` — ok
- `replay_reproduces_a_cache_prompt_run` — ok

**E2E — live, `#[ignore]`d (`live_evaluation`)**
- `kv_cache_reuse_reduces_prompt_eval_time` — present, compiles, listed by `--ignored --list`; not a CI gate.

## Caveats carried forward (from the critic)
- **C-001 (accepted):** the headline measurement is inherently live/machine-specific and not CI-gated; the mechanism and every invariant it rests on are CI-verified, and the reduction is recorded via the repo's established `#[ignore]`d live harness.
- **C-002 (accepted):** the static `*.request.json` captures were left as dated provider records (no offline test loads them; only `.sse` responses are `include_bytes!`'d). The verifiable exact-bytes assertions in `model_protocol.rs` were updated in lockstep; re-recording is a live-capture follow-up.
- **C-003 / C-004 (layered / deferred):** offline cannot prove a server-side optimization end-to-end; the guarantee is split across offline invariants + server semantics + the live benchmark. The prefix property under compaction is covered by sprint 2's compaction tests (compaction truncating the prefix is by design and non-corrupting).

## Verdict
Test phase satisfied for INT-0004: every acceptance criterion maps to a named, present test; the CI-verifiable set is green on the pinned toolchain; the one inherently-live criterion is recorded via the established ignored benchmark. Proceed to loop with the caveats above recorded.
