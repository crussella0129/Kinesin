Finalized - DO NOT EDIT

# Sprint 3 Build Plan

## Intents
- [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) — state: planned; acceptance criteria covered: measured reduction in prompt-eval time on a shared prefix (live benchmark); run immutability and the trace/replay contract preserved; slot/cache lifetime honest under concurrency (reuse never corrupts a run).

## Schema Tree
- Sprint Goal: reuse llama.cpp's cached prompt prefix instead of re-evaluating it
  - Mechanism
    - T-001: emit `cache_prompt` (config-toggled) and re-record the request fixtures
  - Verification
    - T-002: offline reuse invariants + a live measurement harness

## Execution Sequence

### T-001: Emit cache_prompt and re-record the request fixtures
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- **Touches:** src/model.rs, src/config.rs, src/runner.rs, src/replay.rs, tests/fixtures/live/*.request.json, tests/model_protocol.rs
- **Depends on:** (none)
- **Acceptance criterion:** run immutability and the trace/replay contract are
  preserved while the request reuses a cached prefix.
- **Success criterion (EARS):**
  - **WHEN** `prepare` builds a request with `cache_prompt` enabled, **THEN** the body **SHALL** include `cache_prompt = true`.
  - **WHEN** `cache_prompt` is disabled, **THEN** the body **SHALL NOT** include a `cache_prompt` key.
  - **WHEN** a `ModelConfig` omits the setting, **THEN** it **SHALL** default to enabled.
- **Notes:** add `cache_prompt: bool` to `ModelConfig` (default true) and `ModelOptions`;
  thread it through `runner::options`/`finalize_options` and `replay`'s frozen
  `options`. The flag is stored in the frozen config, so a new capture records it and
  replay recomputes the identical request; only the static request fixtures and the
  exact-bytes assertions in `model_protocol.rs` shift, updated in lockstep (as sprint
  0's `stream_options` change did). Dynamic replay tests self-capture and stay
  consistent.

### T-002: Offline reuse invariants and a live measurement harness
- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- **Touches:** tests/model_protocol.rs, tests/runner_journal.rs, tests/replay.rs, tests/live_evaluation.rs
- **Depends on:** T-001
- **Acceptance criterion:** the measured prompt-eval reduction is recorded on a live
  server; slot/cache reuse never corrupts a run (transparent optimization).
- **Success criterion (EARS):**
  - **WHEN** a run performs successive model turns, **THEN** each prepared request's message list **SHALL** be a prefix of the next turn's (until compaction truncates it).
  - **WHEN** a scripted run is executed with `cache_prompt` enabled and disabled, **THEN** its candidate and acceptance status **SHALL** be identical.
  - **WHEN** a capture made with `cache_prompt` is replayed, **THEN** it **SHALL** be `consistent` with matching request fingerprints.
  - **WHEN** the live benchmark runs on the pinned server, **THEN** it **SHALL** record a reduced prompt-evaluation time on the reused prefix (manual/live; not a CI gate).
- **Notes:** the prefix-extension property is the offline proof that the server's
  reuse is valid; the on/off-identical-outcome test shows the additive flag changes
  nothing a run does; the live measurement (`#[ignore]`d, requires the pinned
  127.0.0.1:8080 model) reads `timings.prompt_ms`/`prompt_n` and records workload and
  machine — the same non-unit verification pattern the CI matrix used.
