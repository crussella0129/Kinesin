# INT-0024 — Coding-harness regression evaluation

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0024
- **State:** proposed
- **Work evidence:** [T-103 backlog](../work/tasks.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Evaluate useful coding-task completion with frozen workloads and independent outcome checks across supported model/configuration profiles. Separate deterministic runtime tests, live model-quality measurements and timing. Non-goal: treating model self-assessment or authorization corpus success as task competence.

## Acceptance criteria
- Workloads record initial workspace state, task prompt, expected effects, independent checks, model/tool/configuration versions and resource ceilings.
- Failures and inconclusive runs are retained; reports compare success, regressions, tokens and latency with workload/machine provenance.
- Context growth, compaction, tool errors, MCP and interrupted work have representative coding cases.
- Deterministic contract regressions block CI; noisy live-quality thresholds and release decisions have a documented comparison policy.

## Rationale
INT-0015 proves gates and INT-0016 measures telemetry; neither establishes that the main harness completes realistic work reliably.

## Alternatives
Use only toy file-reading tests (insufficient); make every stochastic score a hard CI gate (flaky and misleading).

## Consequences
Consumes controlled live inference and requires maintained fixtures; existing live-evaluation tooling should be extended rather than replaced.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.

