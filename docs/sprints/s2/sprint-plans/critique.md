# Plan Critique — Sprint 2

Adversarial read-only screen of `build-plan.md` / `test-plan.md` against the research
report and INT-0002.

## Concerns

### C-001: T-002 touches both runner.rs and replay.rs
- **Where:** `build-plan.md` T-002 (Touches: src/runner.rs, src/replay.rs).
- **Quote:** "replace the three `runner.rs` stop sites … and the mirrored `replay.rs` sites … with a bounded compaction loop."
- **Failure mode:** granularity.
- **Suggested response:** reject (the critique is wrong because …). The runner and replay gate on `max_history_bytes` in lockstep by design: replay recomputes state and must apply the identical drops, or the recorded request fingerprints diverge. Splitting them across two tasks would land an intermediate commit where the runner compacts but replay still stops — a self-inflicted replay failure. Enforcing compaction consistently is one logical concern; keeping it in one task is correct.

### C-002: evidence preservation depends on group integrity, not just the result
- **Where:** `build-plan.md` T-001 (protected set) / `test-plan.md` `drop_oldest_preserves_evidence_bearing_result`.
- **Quote:** "any tool result carrying an `evidence_id` (with its group's assistant message)."
- **Failure mode:** intent-drift (potential evidence contract break).
- **Why it matters:** dropping the assistant `tool_calls` message while keeping its evidence result would leave an orphan `Role::Tool` (API-invalid) and could detach the evidence from its call.
- **Suggested response:** fix-in-plan — already addressed. The protected unit is the whole group: when any result in a group is evidence-bearing, the assistant message and all its results are preserved together. `drop_oldest_removes_whole_group` asserts no dangling `tool_call_id`, and `drop_oldest_preserves_evidence_bearing_result` asserts the evidence group is skipped. No change needed beyond the stated group rule.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — T-001 (5 clauses) and T-002 (4 clauses) are each measurable `WHEN … THEN … SHALL`.
- **Plan-test mismatch:** none — every EARS clause maps to a named test in the traceability table, and every named test cites a clause.
- **Missing risk coverage:** none — the research risks (replay divergence, checked-evidence loss, partial groups, strategy determinism) each land on a task and a test; the summary strategy is explicitly deferred, not silently dropped.
- **Hidden dependencies:** none — T-002 `Depends on: T-001` (the core method + config it calls); touched paths are disjoint between tasks.
- **E2E status drift:** none — E2E is `possible` with a named feasible test (`test_cli_run_compacts_past_history_limit`), consistent with the token/command E2E pattern already in the repo.

## Confidence
clean
