# INT-0026 — Session continuity and model-context admission

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0026
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** [sprint 10 partial live observation](../sprints/s10/sprint-tests/remote-deployment.md#faithful-session-cache-observation); broader criteria remain unverified
- **Documentation evidence:** none

## Intent
Preserve bounded, origin-marked task context over actual interactive session turns and admit requests against the model's real token window, while keeping each run immutable and replay pure. Verify session cache benefit using requests assembled by the harness itself. Non-goals: unbounded memory, silently modifying checked evidence, cross-owner context sharing or mandatory slot pinning.

## Acceptance criteria
- A multi-turn session retains the information required by its declared continuity policy; branching and interruption never inherit broader authority.
- Compaction/loading respect history, encoded request and model token budgets, including tool schemas/template/output reserve; unsupported token counting has an honest defined policy.
- Cache measurements use actual session-prepared requests with model/workload/machine provenance, distinguishing within-run reuse from cross-run reuse.
- Mixed-owner concurrent sessions and slot reassignment preserve response correlation and data boundaries; old runs replay without reconnecting.

## Rationale
INT-0002 owns byte compaction and INT-0004 claims session caching, but neither fully specifies semantic continuity or token-window admission. Sprint 10 found the historical benchmark constructs its own conversation.

## Alternatives
Continue carrying only the preceding candidate without specifying losses (insufficient for a general session contract); persist unrestricted transcript history (violates resource bounds).

## Consequences
Requires a bounded continuation format and compatibility evidence. Sprint 10 measured real shared-prefix reuse on a provisioned model. Omitting cache_prompt leaves backend defaults in effect; an explicit disabled-cache control, full-history continuity, token-window admission and concurrent-slot evidence remain outstanding.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.
