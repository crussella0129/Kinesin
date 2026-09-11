# Sprint 5 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) | roadmap.md exists, themed, sequenced, SUMMARY-linked | T-002 / WHEN complete THEN roadmap.md exists with four themes + sequencing + parking-lot | `roadmap_present_and_themed` |
| [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) | every gap maps to a tracked intent; all roadmap intents authored | T-001 + T-002 / WHEN read THEN every research gap maps to a named intent | `roadmap_maps_every_gap`, `check_book_validates_expanded_set` |
| [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) | existing proposed intents reviewed + cross-referenced | T-002 / WHEN read THEN INT-0005/0006/0007/0009/0010 carry a roadmap reference | `existing_intents_cross_referenced` |
| [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) | check-book validates the expanded set | T-001 / WHEN check-book runs THEN valid v2 Book | `check_book_validates_expanded_set` |

## Unit Tests
Not applicable — this sprint produces documentation (intent chapters + roadmap),
not code. Verification is structural/coverage, recorded below as named checks
(the same non-unit verification sprint 1's CI task used).

## Integration Tests
### Roadmap coverage & Book validity
- **Intents:** [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md)
- `check_book_validates_expanded_set`: `check-book.sh` reports a valid v2 Book with INT-0012..0018 present.
- `summary_links_all_new_intents`: each of INT-0012..0018 is reachable from `docs/SUMMARY.md`.
- `roadmap_present_and_themed`: `docs/roadmap.md` exists, is SUMMARY-linked, and presents the four themes with sequencing and a parking-lot.
- `roadmap_maps_every_gap`: every gap enumerated in the research report (sandboxing, supply-chain, tamper-evidence, threat model, observability, approval/JIT, parallel orchestration, MCP, skills, supervision, overlay, coordination) maps to a named tracked intent in the roadmap.
- `existing_intents_cross_referenced`: INT-0005/0006/0007/0009/0010 each carry a one-line roadmap cross-reference.

## End-to-End Tests
- **Status:** not-yet-possible — this sprint has no runtime deliverable.
- Unlocked by: the individual workstream sprints (INT-0012..0018), each of which
  carries its own runtime acceptance and E2E when implemented. Rationale: a
  roadmap is verified by coverage and structure, not by executing a program.
