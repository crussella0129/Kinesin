# Test Critique — Sprint 3

Adversarial read-only screen of the KV-cache-reuse evidence against the locked plans
and INT-0004.

## Concerns

### C-001: the headline acceptance is not CI-verified — RESOLVED (measured)
- **Where:** `e2e-tests.md` / `kv_cache_reuse_reduces_prompt_eval_time`.
- **Failure mode:** e2e-drift.
- **Why it mattered:** INT-0004's first acceptance criterion is a *measured* reduction; CI cannot produce a wall-clock measurement, and the sprint must not close on an unproduced number.
- **Resolution:** the benchmark was **run** against the pinned b6500 server on 2026-09-11 (the same profile in `model-preflight.md`) and passed: the second, prefix-extending turn evaluated **16 of 59** prompt tokens. A cold-vs-warm probe recorded the prompt-eval *time* reduction with workload and machine — ≈18.4× (933 ms → 51 ms) on a ~2 576-token shared prefix, stable across 3 iterations. The criterion is satisfied by an executed measurement, not deferred. **Honest nuance recorded:** the wall-clock reduction scales with prefix size (≈1.0× at ~55 tokens, where per-request overhead dominates; large where prompt-eval is compute-bound). The measurement remains outside CI because it is inherently machine-specific, but that is a property of the metric, not an unverified claim.

### C-002: the static request fixtures were not re-recorded (plan deviation)
- **Where:** locked `build-plan.md` T-001 said "re-record the request fixtures (`tests/fixtures/live/*.request.json`)"; the completed task left them unchanged.
- **Quote:** T-001 completion — "the static `*.request.json` files were left as dated provider captures (no offline test loads them)."
- **Failure mode:** intent-drift (plan vs. implementation).
- **Why it matters:** a locked plan step was not executed as written.
- **Suggested response:** reject/accept-with-rationale. The step rested on a false premise: those files are not loaded by any offline test (only the `.sse` responses are `include_bytes!`'d), and they are dated 2026-09-08 provider captures. Inserting a `cache_prompt` field never sent on that date would falsify a dated record. The verifiable part of the plan step — the exact-bytes assertions in `model_protocol.rs` — was updated (its builders now carry `cache_prompt` and its tests pass), and current request behavior is unit-tested (`cache_prompt_present_when_enabled`). Re-recording is a live-capture follow-up, not an offline edit.

### C-003: the outcome-invariance test is weak with a scripted client
- **Where:** `cache_prompt_does_not_change_run_outcome`.
- **Failure mode:** weak-assertion.
- **Why it matters:** a scripted client ignores the request body, so on-vs-off identical outcome proves only that the flag does not perturb the harness — not that a real server's cache reuse preserves output.
- **Suggested response:** accept as layered. Offline can only prove the flag is additive and inert; the substantive "reuse never corrupts" guarantee is a llama.cpp property (it reuses only byte-identical prefix tokens) plus the live benchmark's requirement that the answer is unchanged from the non-cached baseline. Together they cover the invariant; no single offline test can.

### C-004: the prefix property is not tested under compaction
- **Where:** `prepared_request_of_each_turn_extends_the_previous` (uses a run that does not compact).
- **Failure mode:** negative-path.
- **Suggested response:** defer-with-rationale. The test proves the prefix-extension property that makes reuse valid; the compaction case (which deliberately truncates the prefix, reducing reuse without corrupting anything) is a known, correct interaction already covered by sprint 2's compaction tests. Exercising the two together adds no new correctness guarantee — compaction changing the prefix is by design.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — all three acceptance criteria map to named tests (immutability/replay → replay test + unchanged suite; never-corrupts → outcome-invariance + server semantics; measured reduction → live benchmark).
- **Plan-test mismatch:** none — every EARS clause maps to a named test; the live clause maps to the named ignored benchmark, which was executed against the pinned server and passed.
- **Stub leakage:** none beyond C-003, which is inherent to offline verification of a server-side optimization.
- **Missing risk coverage:** none — the research risks each landed on a test or an explicit deferral (slot pinning deferred to auto-prefix-match).
- **Granularity:** none — T-001 mechanism, T-002 verification; distinct.

## Confidence
proceed-with-caveats
