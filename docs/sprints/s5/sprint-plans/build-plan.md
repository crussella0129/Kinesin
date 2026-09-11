# Sprint 5 Build Plan

## Intents
- [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) — state: planned; acceptance criteria covered: roadmap doc exists/themed/sequenced/linked; every research-report gap maps to a tracked intent with all roadmap intents authored; existing proposed intents reviewed + cross-referenced; check-book validates the expanded set.

## Schema Tree
- Sprint Goal: a gap-complete, prioritized production & security roadmap
  - Roadmap intents
    - T-001: author the seven workstream intent chapters
  - Roadmap document
    - T-002: write docs/roadmap.md + cross-reference existing intents

## Execution Sequence

### T-001: Author the seven workstream intent chapters
- **Intent:** [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md)
- **Touches:** docs/intents/INT-0012..0018-*.md, docs/SUMMARY.md
- **Depends on:** (none)
- **Acceptance criterion:** every gap named in the research report maps to a tracked intent with all roadmap intents authored; check-book validates the expanded set.
- **Success criterion (EARS):**
  - **WHEN** the build phase completes, **THEN** `docs/intents/` **SHALL** contain well-formed v2 chapters INT-0012 (sandboxing), INT-0013 (supply-chain), INT-0014 (tamper-evident journal), INT-0015 (threat model & assurance), INT-0016 (observability), INT-0017 (approval gates & JIT privilege), INT-0018 (subagents/parallel), each with non-empty Intent/Acceptance/Rationale/Alternatives/Consequences/Transition history.
  - **WHEN** `check-book.sh` runs, **THEN** it **SHALL** report a valid v2 Book covering the new chapters.
  - **WHEN** the chapters exist, **THEN** each **SHALL** be reachable from `docs/SUMMARY.md`.
- **Notes:** state `proposed` (or `deferred` where clearly downstream, e.g. INT-0018 behind the scheduler); follow `schemas/intent.md`; cross-link related intents in prose.

### T-002: Write the roadmap doc + cross-reference existing intents
- **Intent:** [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md)
- **Touches:** docs/roadmap.md, docs/SUMMARY.md, docs/intents/INT-0005/0006/0007/0009/0010-*.md
- **Depends on:** T-001
- **Acceptance criterion:** the roadmap doc exists, is themed/sequenced/linked, maps every research-report gap to a tracked intent, and the existing proposed intents carry a roadmap cross-reference.
- **Success criterion (EARS):**
  - **WHEN** the build phase completes, **THEN** `docs/roadmap.md` **SHALL** exist, be reachable from `docs/SUMMARY.md`, present the four themes (security hardening; supply-chain & assurance; operability; SotA capability) with a recommended sequencing, and list a parking-lot of deferred-without-a-chapter items.
  - **WHEN** the roadmap is read, **THEN** every gap in the sprint 5 research report **SHALL** map to a named tracked intent (existing or INT-0012..0018).
  - **WHEN** the existing proposed intents INT-0005/0006/0007/0009/0010 are read, **THEN** each **SHALL** carry a one-line roadmap cross-reference.
- **Notes:** documentation task; verification is structural/coverage (check-book + reading), not cargo tests.
