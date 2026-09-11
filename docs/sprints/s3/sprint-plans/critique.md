# Plan Critique — Sprint 3

Adversarial read-only screen of `build-plan.md` / `test-plan.md` against the research
report and INT-0004.

## Concerns

### C-001: the headline acceptance is not CI-verifiable
- **Where:** `build-plan.md` T-002 / `test-plan.md` End-to-End.
- **Quote:** "measured reduction in prompt-eval time … requires the manually started pinned model."
- **Failure mode:** e2e-drift.
- **Why it matters:** INT-0004's first acceptance criterion is a measured reduction, and CI cannot produce it, so the sprint cannot make the headline green offline.
- **Suggested response:** defer-with-rationale / accept. The measurement is inherently live and machine-specific; there is no offline proxy for wall-clock prompt-eval time. The mechanism and every invariant it depends on (the request carries `cache_prompt`; each turn's prompt is a prefix of the next so reuse is valid; the flag changes nothing a run does; replay stays consistent) are CI-verified, and the reduction is recorded via an `#[ignore]`d live test in the repo's existing `live_evaluation.rs` harness — the same non-unit verification the CI matrix used. The intent already frames the measurement as recorded-with-workload-and-machine, i.e. a live artifact.

### C-002: model_protocol.rs is touched by both T-001 and T-002
- **Where:** `build-plan.md` T-001 and T-002 (Touches: tests/model_protocol.rs).
- **Quote:** T-001 updates "exact-request-bytes assertions in `model_protocol.rs`"; T-002 adds `prepared_request_of_each_turn_extends_the_previous` there.
- **Failure mode:** hidden-dep.
- **Suggested response:** reject (the critique overreaches). `T-002 depends on T-001`, so the edits are ordered: T-001 updates the existing byte assertions to include `cache_prompt`, then T-002 adds a new prefix-extension test. Sequential edits to one file under a declared dependency are the normal case, not a hidden dependency. (The new test may equally live in `runner_journal.rs`, where the scripted multi-turn harness already exists.)

### C-003: the "never corrupts" invariant is only weakly exercised offline
- **Where:** `test-plan.md` `cache_prompt_does_not_change_run_outcome`.
- **Quote:** "the same scripted run … with `cache_prompt` enabled and disabled yields the identical candidate and acceptance."
- **Failure mode:** weak-assertion.
- **Why it matters:** a scripted client ignores the request, so the on/off-identical test cannot exercise the server's actual cache; it proves only that the flag does not perturb the harness.
- **Suggested response:** fix-in-plan / accept as layered. Offline can only prove the flag is additive and inert to the harness; the substantive "reuse never corrupts" guarantee is a llama.cpp property (it reuses only byte-identical prefix tokens) plus the live benchmark's requirement that the run's answer is unchanged from the non-cached baseline. Together — additive-flag test + documented server semantics + live output-equivalence — they cover the invariant; no single offline test can.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — T-001 (3 clauses) and T-002 (4 clauses) are each measurable.
- **Plan-test mismatch:** none — every EARS clause maps to a named test; the live clause maps to the named ignored benchmark.
- **Missing risk coverage:** none — the research risks (live measurement, fixture/fingerprint shift, compaction-invalidates-reuse, slot affinity under concurrency) each land on a task, a test, or an explicit deferral; slot pinning is deferred with the auto-prefix-match rationale.
- **Intent drift:** none — INT-0004 is linked and `planned`; the approach and the deferred slot-pinning are recorded in the intent's Transition history.
- **Granularity:** none — T-001 is the mechanism + its fixtures, T-002 is verification; distinct, coherent commits.

## Confidence
proceed-with-caveats
