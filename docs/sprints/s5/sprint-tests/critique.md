# Test Critique — Sprint 5

Adversarial read-only screen of the sprint 5 verification against INT-0011.
This is a documentation/roadmap deliverable, so the screen weighs coverage,
completeness honesty, and structure.

## Concerns

### C-001: "gap-complete" is bounded by the review's thoroughness
- **Where:** INT-0011 acceptance "every gap … maps to a tracked intent"; `roadmap_maps_every_gap`.
- **Failure mode:** weak-assertion.
- **Why it matters:** the check proves every gap *named in the research report* maps to an intent — it cannot prove the research review itself found every gap. A missed gap would pass.
- **Suggested response:** accept-with-rationale. Absolute gap-completeness is unprovable for any roadmap. The review is anchored to three external checklists (the codebase's own deferral docs, the SotA survey, and OWASP/NIST/CISA standards), which bounds the blind spot far better than an ad-hoc list; and the roadmap is explicitly a living document (INT-0011 is superseded by a future revision, not frozen), so a later-found gap is added, not lost.

### C-002: verification is by inspection/grep, not a machine gate on every push
- **Where:** `integration-tests.md` checks.
- **Failure mode:** evidence-drift (screened).
- **Why it matters:** only `check-book.sh` is CI-enforced; the SUMMARY/roadmap/cross-ref/gap checks were run by hand this sprint.
- **Suggested response:** defer-with-rationale. `check-book` (the structural gate) does run in CI; the remaining checks verify a one-time documentation state, not runtime behavior that could regress silently, so a standing test would add little. The commands are recorded verbatim so any reviewer can re-run them.

### C-003: the workstream intents assert design, not working mechanisms
- **Where:** INT-0012..0018.
- **Failure mode:** intent-coverage (screened, not upheld).
- **Why it matters:** none of the seven is implemented; the roadmap could be mistaken for delivered hardening.
- **Suggested response:** reject (the concern misreads the sprint). INT-0011's deliverable is explicitly the *roadmap*, and each workstream intent is `proposed` (backlog) with its own future acceptance/E2E. The roadmap and every chapter say so plainly; nothing here claims the hardening is built.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — each INT-0011 acceptance criterion maps to a named, executed check, and each check traces to a criterion.
- **Stub leakage / negative-path / flake-risk:** not applicable — no code, no mocks, no error-path clauses; `check-book` and the greps are deterministic.
- **E2E cop-out:** none — `not-yet-possible` is correct for a no-runtime deliverable, with a credible unlocking (the named workstream sprints) and rationale.
- **Evidence drift:** none — artifacts name tested head `84fb7a3`, carry the exact confirmations, and identify the Test-evidence link attached to INT-0011.

## Confidence
proceed-with-caveats
