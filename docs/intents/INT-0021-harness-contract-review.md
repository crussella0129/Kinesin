# INT-0021 — Harness contract baseline and roadmap revision

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0021
- **State:** planned
- **Work evidence:** [sprint 10 build plan](../sprints/s10/sprint-plans/build-plan.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Maintain a traceable contract for Kinesin as the operator's main local agent harness. Review all original intents before code, then map every acceptance criterion to actual implementation and evidence; capture missing outcomes as explicit chapters, and revise the roadmap and assurance package with retained historical provenance. This follows INT-0011 and INT-0015. Non-goal: treating roadmap proposals or past green tests as proof that all capabilities are delivered.

## Acceptance criteria
- An intent-only baseline precedes implementation inspection; all INT-0001..0020 have criterion-level assessments, including proposed work.
- Pure decisions, frozen authority, checked acceptance, owner isolation, bounded admission, cancellation, immutable settlement and pure replay have named code/test evidence and supported limits.
- Missing lifecycle, evaluation, secret/egress, session/context and encrypted-deployment outcomes have explicit owning chapters; proposed intent ambiguities are repaired.
- The roadmap reflects current state and next priorities; assurance individually maps OWASP LLM and Agentic risks, inventories native FFI as well as first-party unsafe, and has an update owner/cadence.
- Book validation, link checks and independent critique pass, with unresolved evidence clearly retained.

## Rationale
The project evolved from a learning exercise into the primary harness; incremental feature chapters alone do not own its foundational promises.

## Alternatives
Keep the old snapshot (rejected: contradictory claims); implement every proposed feature in one sprint (rejected: the requested build is a repair sprint).

## Consequences
Supersedes the historical roadmap/assurance revision without erasing earlier evidence. Baseline coverage is an audit result, not a certification.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.
- 2026-09-12: `proposed → planned`; selected for sprint 10 under the user's instruction to audit, repair, verify and submit the PR.
