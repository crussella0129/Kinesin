# INT-0011 — Production-readiness & security roadmap

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0011
- **State:** superseded
- **Work evidence:** [T-001 build plan](../sprints/s5/sprint-plans/build-plan.md#t-001-author-the-seven-workstream-intent-chapters)
- **Completion evidence:** [T-001–T-002 completion log](../work/completed-tasks.md)
- **Code evidence:** none
- **Test evidence:** [sprint 5 test report](../sprints/s5/sprint-tests/test-report.md) — all acceptance criteria verified (tested head `84fb7a3`)
- **Documentation evidence:** [the roadmap](../roadmap.md)

## Intent
Define and maintain a prioritized, gap-complete roadmap that takes Kinesin from
its current reference-implementation state to a **production-grade,
general-purpose local agent harness** with State-of-the-Art capabilities,
hardening, and a very high ("NSA-grade") security bar. The deliverable is a
`docs/roadmap.md` that organizes the work into themes, sequences it, and maps
each identified gap — from a full codebase review, the public SotA, and the
OWASP / NIST / CISA-NSA standards — onto a tracked intent chapter (existing or
newly authored). Non-goals: implementing any workstream here (each is its own
intent/sprint); claiming a security certification (the bar is anchored to
concrete controls, not a slogan); creating a chapter for every speculative idea
(parking-lot items live in the roadmap doc until a sprint selects them).

## Acceptance criteria
- `docs/roadmap.md` exists, is reachable from `docs/SUMMARY.md`, and organizes
  the work into the four themes (security hardening; supply-chain & assurance;
  operability; SotA capability) with a recommended sequencing.
- Every gap named in the sprint 5 research report maps to a tracked intent
  (existing INT-0005/0006/0007/0009/0010 or new INT-0012..0018), with no gap
  left unmapped and no roadmap intent left un-authored.
- The five pre-existing proposed intents are reviewed and carry a roadmap
  cross-reference; each new workstream intent is a well-formed chapter with
  acceptance criteria, rationale, alternatives, and consequences.
- `check-book.sh` validates the expanded intent set.

## Rationale
The codebase and its own docs already articulate the deferrals (OS sandboxing,
signed attestations vs. digests, parallel scheduling, deployment-security
evidence); the SotA survey and security standards add the rest. Turning that
scattered knowledge into one prioritized, tracked roadmap is what lets the
project be driven to production deliberately rather than ad hoc, and gives the
operator an ordered menu of fundable sprints.

## Alternatives
A single monolithic "harden and productionize everything" intent (rejected:
unschedulable and untestable; distinct outcomes need distinct intents). Keeping
the roadmap only in prose/docs without intent chapters (rejected: the Book, not
a doc, is the semantic state machine the sprint loop schedules from).

## Consequences
Adds a durable roadmap document that must be kept in step with the intent set as
sprints complete; introduces seven new proposed/deferred intents (backlog, not
commitments); the meta-intent itself is realized once the roadmap is authored and
gap-complete, and is superseded by a future roadmap revision rather than edited
in place after realization.

## Transition history
- 2026-09-11: created as `proposed`; sprint 5 (roadmap review) selected it as the objective.
- 2026-09-11: `proposed → planned`; linked to the sprint 5 build plan (T-001 author INT-0012..0018, T-002 write docs/roadmap.md + cross-reference existing intents).
- 2026-09-11: `planned → active`; sprint 5 build began (T-001).
- 2026-09-11: `active → realized`; sprint 5 authored `docs/roadmap.md` (four themes, recommended sequencing, SotA/standards mapping, parking-lot) and seven workstream intents (INT-0012..0018), reviewed and cross-referenced the five existing proposed intents, and every gap from the codebase review + SotA survey maps to a tracked intent. check-book validates the 18-chapter Book. The workstream intents are `proposed` backlog, not delivered work; this meta-intent is realized by the roadmap's existence and gap-completeness, and will be `superseded` by a future roadmap revision rather than edited after realization.
- 2026-09-12: `realized → superseded` by [INT-0021](INT-0021-harness-contract-review.md). The sprint 10 baseline/roadmap revision adds missing foundational ownership and current evidence. Historical criteria and realization notes are retained as provenance, not current proof.
