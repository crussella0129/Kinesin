# Plan Critique — Sprint 5

Adversarial read-only screen of `build-plan.md` and `test-plan.md` against the
research report and INT-0011. This is a documentation/roadmap sprint, so the
screen weighs coverage and structure over runtime behavior.

## Concerns

### C-001: T-001 authors seven intent chapters in one task
- **Where:** `build-plan.md` T-001.
- **Quote:** "docs/intents/ SHALL contain well-formed v2 chapters INT-0012 … INT-0018".
- **Failure mode:** granularity.
- **Why it matters:** a task producing seven chapters could hide an incoherent or uneven diff.
- **Suggested response:** defer-with-rationale. The seven chapters are one coherent deliverable — the roadmap's intent set — authored to a single schema in one pass; splitting into seven tasks would be over-granular ceremony for documentation. check-book plus the per-chapter well-formedness EARS clause keep each honest.

### C-002: both tasks edit `docs/SUMMARY.md`
- **Where:** T-001 and T-002 both touch `docs/SUMMARY.md`.
- **Failure mode:** hidden-dep.
- **Why it matters:** two tasks editing one file can collide.
- **Suggested response:** accept. They edit disjoint regions — T-001 adds the seven new-intent nav links, T-002 adds the single roadmap-doc link — and T-002 depends on T-001, so they run in order. No shared-line conflict.

### C-003: INT-0011 is a self-referential meta-intent realized by documentation
- **Where:** `INT-0011` acceptance criteria.
- **Failure mode:** intent-drift.
- **Why it matters:** a "roadmap of intents" intent risks being unfalsifiable.
- **Suggested response:** reject (the concern overstates it). The acceptance criteria are concrete and checkable: a file exists and is linked; check-book validates; every named research-report gap maps to a tracked intent; five existing intents carry a cross-reference. These are coverage/structure facts, not judgment calls. The workstream intents stay `proposed`/`deferred` (backlog), not silently realized.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — each task has measurable WHEN/THEN/SHALL clauses (chapter presence + well-formedness, check-book validity, SUMMARY reachability, roadmap themes/sequencing, gap coverage, cross-references).
- **Plan-test mismatch:** none — every EARS clause maps to a named verification (`check_book_validates_expanded_set`, `summary_links_all_new_intents`, `roadmap_present_and_themed`, `roadmap_maps_every_gap`, `existing_intents_cross_referenced`), and each verification traces to a clause.
- **Missing risk coverage:** none — research risks (scope sprawl, over-claim, sequencing uncertainty) land on the parking-lot, the standards-anchoring, and a recommended-not-locked sequence.
- **E2E status drift:** none — `not-yet-possible` is correct for a no-runtime deliverable, with a credible unlocking (the workstream sprints) and rationale (coverage, not execution).

## Confidence
proceed-with-caveats
