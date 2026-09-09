# Sprint 0 Build Plan

## Intents
- [INT-0001](../../../intents/INT-0001-token-accounting.md) — state: planned; acceptance criteria covered: usage captured for both response paths; honest absence (never zero); per-run totals surfaced in `inspect`; storage/replay compatibility handled.

## Schema Tree
- Sprint Goal: record and surface model token usage
  - Koil capture
    - T-001: parse and carry `usage` on both response paths
    - T-002: request streamed usage and update streaming fixtures
  - Runner journalling
    - T-003: journal per-call usage and accumulate per-run totals

## Execution Sequence

### T-001: Parse and carry usage out of Koil
- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- **Touches:** src/model.rs
- **Depends on:** (none)
- **Acceptance criterion:** usage is captured for both the streaming and
  non-streaming paths, and absence stays unknown rather than zero.
- **Success criterion (EARS):**
  - **WHEN** `decode_reply` parses a response carrying a `usage` object, **THEN** it **SHALL** return the reply with `Some(Usage)` holding prompt and completion counts.
  - **WHEN** a response omits `usage`, **THEN** it **SHALL** return the reply with `None`.
  - **WHEN** the streaming assembler observes its single usage chunk, **THEN** it **SHALL** capture the prompt and completion counts and return them with the reply.
- **Notes:** `Usage { prompt_tokens, completion_tokens }` is a Koil-layer type;
  do not add token fields to the pure-core `ModelReply`. Return a
  `ModelOutcome { reply, usage }` (or tuple) from `decode_reply`, `send`, and
  `send_with_text`. Reuse the existing single-usage-chunk validation, replacing
  `usage_seen: bool` with a captured `Option<Usage>`.

### T-002: Request streamed usage and update streaming fixtures
- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- **Touches:** src/model.rs, tests/fixtures/live/*stream*.request.json, tests/model_protocol.rs, tests/live_comparisons.rs
- **Depends on:** T-001
- **Acceptance criterion:** streamed usage is actually delivered so the streaming
  path can capture it.
- **Success criterion (EARS):**
  - **WHEN** `prepare` builds a request with `stream = true`, **THEN** the body **SHALL** include `stream_options.include_usage = true`.
  - **WHEN** `prepare` builds a request with `stream = false`, **THEN** the body **SHALL NOT** include `stream_options`.
- **Notes:** this changes the streaming request body and its `sha256`; update the
  streaming request fixtures and any test asserting exact streamed request bytes
  in lockstep. Non-streaming requests are unchanged. **Replay:** confirm whether
  any replay capture is a streaming request; replay recomputes the prepared
  request and compares its fingerprint to the recorded `model_planned`
  `request_sha256`, so a pre-change streaming capture would mismatch. Re-record or
  confirm none exists (non-streaming captures are unaffected).

### T-003: Journal per-call usage and accumulate per-run totals
- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- **Touches:** src/runner.rs
- **Depends on:** T-001
- **Acceptance criterion:** each model call's tokens are journalled, per-run
  totals are summed and surfaced via `inspect`, and a run with no reported usage
  omits token totals rather than recording zero.
- **Success criterion (EARS):**
  - **WHEN** a model call reports usage, **THEN** the runner **SHALL** include `prompt_tokens` and `completion_tokens` in that `model_finished` event.
  - **WHEN** a run finishes and at least one call reported usage, **THEN** the terminal `counters` **SHALL** include the summed `prompt_tokens` and `completion_tokens`.
  - **WHEN** no call reported usage, **THEN** the terminal `counters` **SHALL** omit token totals rather than record zero.
- **Notes:** usage rides in the existing schemaless `data_json`, so no SQLite
  migration. Extend `ScriptStep`/`ScriptedClient` so a scripted step can carry an
  optional `Usage`, enabling the accumulation test offline. `inspect` already
  reads events and the terminal record, so no new command surface.
