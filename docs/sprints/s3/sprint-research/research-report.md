# Sprint 3 Research Report

## Intents Reviewed
- [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) — selected; relevance: this sprint's sole goal is reusing llama.cpp's prompt/KV cache so a stable prefix is not re-evaluated; current state: `proposed` (moves to `planned` at plan finalization). No revision to the intent's outcome was needed, but research clarified the reuse opportunity (see below).

## 1. Sprint Goal
Let llama.cpp reuse its cached prompt prefix instead of re-evaluating it, without
weakening the immutable-run model. `model::prepare` is called on **every model
turn** with the whole conversation-so-far, and a run's conversation only grows by
appending (assistant/tool messages), so each turn's prompt is a prefix-extension of
the previous turn's. Today nothing tells the server to reuse that prefix, so a
multi-turn tool loop re-evaluates the growing prefix every turn — the real cost.
The lever is llama.cpp's `cache_prompt` (with the server's automatic
longest-common-prefix slot selection): set it, keep the prefix byte-stable, and
manage slot/cache lifetime honestly under concurrency. The reuse is bounded by the
design — across *session* turns only the stable system-instructions prefix repeats
(each turn cites rather than carries the prior conversation), but *within* a run's
tool loop the reusable prefix is large and grows. Non-goal: sharing a cache across
owners or unrelated runs, or carrying the full conversation forward across session
turns (that would break immutability).

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| [src/model.rs](../../../../src/model.rs) | high | `prepare` (l.167) builds the request body; it sets no `cache_prompt`. This is where the flag is added and where the request `sha256` (replay fingerprint) is computed. |
| [src/runner.rs](../../../../src/runner.rs) | high | Calls `model::prepare(state.messages(), …)` each model turn; the conversation grows by appending, so consecutive requests share a growing prefix — the within-run reuse opportunity. Compaction (`compact_state`) mutates the prefix, invalidating cache beyond the drop point. |
| [src/dispatch.rs](../../../../src/dispatch.rs) | high | `ModelDispatcher::acquire` (l.86) grants a per-backend-origin permit but tracks no llama-server slot id, so there is no run→slot affinity today; the server's auto prefix-match is what reuses a slot. |
| [src/cli.rs](../../../../src/cli.rs) | medium | `run_session` (l.1106): each session turn is a fresh Freeform run with `continues` (the prior answer, framed). The only cross-turn stable prefix is the system-instructions message. |
| [src/core.rs](../../../../src/core.rs) | medium | `RunState.messages` is the growing conversation; the prefix-extension property (turn K messages are a prefix of turn K+1) is what makes reuse possible, and drop-oldest compaction is the one thing that breaks it. |
| [src/config.rs](../../../../src/config.rs) | medium | `ModelConfig` (l.146): `base_url`, `verified_slots`; a `cache_prompt` toggle (default on) would live here. |
| [src/replay.rs](../../../../src/replay.rs) | medium | Recomputes `prepare` and compares `request_sha256`; adding `cache_prompt` deterministically shifts the fingerprint, so replay stays consistent for new captures but the recorded live fixtures shift. |
| [tests/live_evaluation.rs](../../../../tests/live_evaluation.rs) | high | Holds `#[ignore]`d live tests against a manually started `127.0.0.1:8080` model (`pinned_live_task_cards`, l.437). The prompt-eval-time measurement belongs here. |
| [tests/model_protocol.rs](../../../../tests/model_protocol.rs) | medium | Asserts exact prepared request bytes; a `cache_prompt` field shifts those assertions. |
| [tests/fixtures/live/*.request.json](../../../../tests/fixtures/live) | medium | `context-boundary`, `context-overflow`, `greeting-with-tools` request fixtures encode expected bytes; they gain `cache_prompt` (as the stream fixtures did in sprint 0). |

## 3. External Sources
- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) — `cache_prompt` (reuse the KV cache of a matching prefix; default varies by version), automatic slot selection by longest common prefix, and the `timings.prompt_ms`/`prompt_n` fields used to measure prompt-evaluation time.
- [Kinesin paper review](https://github.com/crussella0129/building-an-agent-harness/blob/main/paper-review.md) — records that async is already handled (Tokio) and prefix re-evaluation is the remaining scaling cost, matching this intent's rationale.

## 4. Risks, Unknowns, Dependencies
- **Risk — the headline is a live measurement.** "Measured reduction in prompt-eval time" requires a real llama.cpp server; offline CI cannot produce it. Mitigation: verify the *mechanism and invariants* offline (request carries `cache_prompt`; the prefix-extension property holds; immutability/replay preserved) and record the measurement via an `#[ignore]`d live test in `tests/live_evaluation.rs` plus a benchmark note — the same non-unit-verification pattern the CI matrix used.
- **Risk — fingerprint/fixture shift.** Adding `cache_prompt` changes every request's bytes and `sha256`. The `*.request.json` fixtures and any exact-bytes assertions in `model_protocol.rs`/`live_comparisons.rs` must be updated in lockstep (as sprint 0's `stream_options` change did). Replay of new captures stays consistent because the change is deterministic.
- **Risk — compaction invalidates reuse.** When `drop_oldest_compactable` removes old groups, the prefix changes from the drop point, so the server can only reuse up to the surviving prefix. This is correct (the request genuinely changed) and needs no code, but the measurement/benchmark should note it.
- **Unknown — slot affinity under concurrency.** Without tracking llama-server slot ids, consecutive turns of one run may not land on the same slot under load, reducing reuse. The server's automatic longest-prefix-match recovers most of it; explicit `id_slot` pinning is a larger change (needs the `/completion` endpoint and dispatcher slot tracking). Recommend relying on auto-match this sprint.
- **Correctness invariant is free.** llama.cpp reuses only byte-identical prefix tokens; a mismatch simply re-evaluates. So "a dropped or reassigned slot never corrupts a run" holds by construction — cache reuse is a transparent optimization, never a source of wrong output.
- **Dependency — none on other intents.** Reuses the existing request/dispatch/replay paths; interacts with sprint 2's compaction only as noted.

## 5. Recommended Approach
**Primary.** Add `cache_prompt: true` to the request body in `model::prepare` (behind
a `ModelConfig` toggle defaulting to on), so llama.cpp reuses the longest common
prefix on whichever slot holds it — capturing within-run tool-loop reuse and the
cross-session system-prefix. Assert offline: the request carries the flag; the
prepared request of turn K is a JSON-message prefix of turn K+1 (the property the
server relies on), including that compaction is the only operation that truncates
it; and the immutability/replay contract still holds (update the request fixtures
and exact-bytes assertions; a replay of a compacted+cached capture stays
consistent). Record the prompt-eval-time reduction with an `#[ignore]`d live test
against the pinned local model and a benchmark note stating workload and machine.

**Alternative considered.** Explicit `id_slot` pinning with run→slot affinity in the
dispatcher. Deferred: it needs slot-id tracking and the `/completion` endpoint, and
the server's automatic prefix-match already delivers the reuse; pinning is a
follow-up optimization if measurement shows the auto-match is insufficient under
concurrency.

**Rationale.** `cache_prompt` is the minimal, deterministic change that unlocks the
reuse the intent asks for while preserving immutability and replay; the invariants
are offline-testable and the measurement follows the repo's established live-test
pattern.

## Artifacts
- No snippets saved; the survey cites live source at the paths and lines above.
