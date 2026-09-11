# INT-0004 — KV-cache reuse across session turns

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** realized
- **Work evidence:** [T-001 build plan](../sprints/s3/sprint-plans/build-plan.md#t-001-emit-cache_prompt-and-re-record-the-request-fixtures)
- **Completion evidence:** [T-001–T-002 completion log](../work/completed-tasks.md)
- **Code evidence:** [T-001 `5670688`, T-002 `8be0e84`](../work/completed-tasks.md) — `cache_prompt` mechanism + offline invariants and live benchmark
- **Test evidence:** [sprint 3 test report](../sprints/s3/sprint-tests/test-report.md) — all acceptance criteria verified; headline live measurement executed (tested head `326bc50`)
- **Documentation evidence:** none

## Intent
Let an interactive session reuse llama.cpp's prompt/KV cache across turns
instead of rebuilding a fresh conversation for every turn. Each session turn is
currently a new run citing the prior answer, which is clean for immutability but
discards the server's cached prefix and re-evaluates it, wasting compute at
scale. Preserve cache reuse where the prefix is stable, without weakening the
immutable-run model. Non-goal: sharing a cache across owners or across
unrelated runs.

## Acceptance criteria
- Measured reduction in prompt-evaluation time across consecutive session turns
  on a shared prefix, recorded with its workload and machine.
- Run immutability and the trace/replay contract are preserved.
- Slot and cache lifetime are managed honestly under concurrency; a dropped or
  reassigned slot never corrupts a run.

## Rationale
This is the actual performance-at-scale lever, distinct from more async: the
runtime already uses Tokio, so the remaining cost is re-evaluated prefixes.

## Alternatives
Accept the cost (current). Add more Tokio concurrency (already present; not the
bottleneck).

## Consequences
Couples the runtime to llama.cpp slot and cache semantics; needs careful
interaction with the scheduler, concurrency, and the immutable-run design.

## Transition history
- 2026-09-08: created as `proposed`.
- 2026-09-11: `proposed → planned`; selected for sprint 3 and linked to the build plan (T-001, T-002). Approach: emit llama.cpp `cache_prompt` (config-toggled, default on) so the growing within-run prefix and the stable system prefix are reused; offline-verify the prefix-extension property, outcome-invariance, and replay consistency; measure the reduction via an ignored live benchmark.
- 2026-09-11: `planned → active`; sprint 3 build began.
- 2026-09-11: test phase verified all acceptance criteria (7 named tests, 241 green offline) with an accepted `proceed-with-caveats` critique; test report linked as Test evidence.
- 2026-09-11: `active → realized`; the headline live measurement was **executed** against the pinned b6500 server (not deferred): the prefix-extending turn re-evaluated only 16 of 59 prompt tokens, and a ~2 576-token shared prefix cut prompt-eval time from ~933 ms to ~51 ms (~18.4×, mean of 3 runs). Immutability/replay preserved; reuse is over a genuine byte-prefix and llama.cpp reuses only byte-identical tokens, so a mismatch re-evaluates and never corrupts. Honest nuance: the wall-clock reduction scales with prefix size (~1.0× for trivially short prompts, which are per-request-overhead-bound). Follow-ups noted (explicit slot pinning; re-recording the dated request fixtures on a live capture) as future work, not gaps.
