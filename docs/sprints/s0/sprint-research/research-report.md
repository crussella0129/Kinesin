# Sprint 0 Research Report

## Intents Reviewed
- [INT-0001](../../../intents/INT-0001-token-accounting.md) — selected; relevance: this sprint realizes it directly; current state: `proposed` (moves to `planned` when the plan links a task).

## 1. Sprint Goal
Record the model token usage `llama-server` already returns, and surface it per
run without inventing counts. The non-streaming response is not parsed for
`usage` at all, and the streaming path validates a `usage` chunk only to set a
seen flag before discarding it. The goal is to capture prompt and completion
tokens for both paths, attach them to each `model_finished` journal event,
accumulate per-run totals into the terminal `counters`, and keep the numbers
honest — absent when the server does not report them, never zero. This is
observability only: no scheduling, pricing, or per-span tracing.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| src/model.rs | high | `WireResponse`/`WireChoice` (non-stream) have **no** `usage` field, so it is never parsed. `StreamingReply` parses a `usage` chunk into `usage_seen: bool` and discards the counts. `decode_reply` and the streaming assembler return `ModelReply` with no usage. The request body sets `stream` but **not** `stream_options.include_usage`. |
| src/core.rs | high | `ModelReply` (Answer/ToolCalls/Incomplete/Failure) is the pure-core type and must stay free of provider token detail. `Counters { model_turns, tool_calls }` is the per-run counter with no token field. |
| src/runner.rs | high | Records `model_planned` then `model_finished` with `classification` and dispatch, no tokens. A private counter struct mirrors `model_turns`/`tool_calls` into the terminal `counters` block (line ~425). This is where per-call usage is journalled and per-run totals accumulate. |
| src/storage.rs | medium | Events store a schemaless `data_json` column (`events(... data_json)`), and the run record's `counters` ride inside event data. **No SQLite migration is needed**: usage fits existing `data_json`. |
| src/cli.rs | medium | `inspect` reads `Command::Get` (run record) and `Command::Events` (event page), so tokens added to `model_finished` data and the terminal `counters` appear in `inspect` with no new surface. |
| src/replay.rs | medium | Replay rebuilds from recorded events and compares prepared-request fingerprints. Added usage fields are additive event data; replay must tolerate older captures that lack them and must not require usage. |
| tests/fixtures/live/text.response.json | medium | Confirms the non-stream shape: `"usage":{"completion_tokens":6,"prompt_tokens":37,"total_tokens":43}`. Many live response fixtures already carry `usage`. |

## 3. External Sources
- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) — the OpenAI-compatible `/v1/chat/completions` response carries a `usage` object with `prompt_tokens`, `completion_tokens`, `total_tokens`; streamed usage requires `stream_options: {include_usage: true}` and arrives in a final chunk.
- [Tracking LLM token usage 2026 (Braintrust)](https://www.braintrust.dev/articles/how-to-track-llm-token-usage-2026) — per-call prompt/completion capture is the base level of token observability; attaching it to each step's record is the standard pattern this sprint follows.

## 4. Risks, Unknowns, Dependencies
- **Risk:** setting `stream_options.include_usage` changes the streaming request body, so its `sha256` fingerprint changes. Streaming replay fixtures and any test asserting exact streamed request bytes must be updated in lockstep.
- **Risk:** carrying usage out of Koil must not add token fields to the pure-core `ModelReply`. Return usage as a Koil-layer value alongside the reply instead.
- **Unknown:** some llama.cpp builds may still omit streamed usage even with `include_usage`. Mitigation: honest absence — `Option` fields left unset, surfaced as unknown.
- **Dependency:** the scripted `ModelClient` returns no usage, so every scripted test naturally exercises the absent case; a live check confirms the reported case.
- **Dependency:** none external; no new crate.

## 5. Recommended Approach
Primary: add a typed `Usage { prompt_tokens, completion_tokens }` (a Koil-layer
struct, not on `ModelReply`). Parse it in the non-stream `WireResponse` and
capture it in the streaming assembler; set `stream_options.include_usage` on
streamed requests. Return it from `send`/`send_with_text` alongside the reply.
In the runner, add `prompt_tokens`/`completion_tokens` to each `model_finished`
event's data when present, and accumulate per-run totals into the terminal
`counters` block. All fields optional; unset means unknown, never zero. No SQLite
schema change; replay treats the new fields as additive.

Alternative considered: store the raw `usage` JSON blob verbatim. Rejected —
typed fields make totals and honest absence explicit and keep `inspect` output
stable.

Rationale: the data is already on the wire; the only work is parsing it, routing
it through Koil without touching the pure core, and journalling it in the
schemaless event data. This is the lowest-risk observability win and validates
the sprint pipeline end to end.

## Artifacts
- None beyond this report; the survey references the exact source files and the
  `text.response.json` fixture above.
