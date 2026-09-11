# Sprint 5 Integration Tests (structural / coverage verification)

- **Intent:** [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md)
- **Tested head:** `84fb7a3e6b070536fa5fda220c7fdc9e634c34e5`
- **Result:** all checks pass.

Documentation deliverable, so verification is structural/coverage, executed with
`check-book.sh` plus repository greps.

| Check (EARS clause) | Command | Result |
|---------------------|---------|--------|
| `check_book_validates_expanded_set` (T-001: check-book valid with new chapters) | `check-book.sh` | **ok** — valid v2 Book, 18 intent chapters |
| `summary_links_all_new_intents` (T-001: INT-0012..0018 reachable from SUMMARY) | `grep -cE "INT-001[2-8]-" docs/SUMMARY.md` | **7** of 7 |
| `roadmap_present_and_themed` (T-002: roadmap has four themes) | `grep -cE "^### [A-D]\. " docs/roadmap.md` | **4** of 4 |
| `summary_links_roadmap` (T-002: SUMMARY links roadmap.md) | grep `roadmap.md` in SUMMARY | **ok** — `[The roadmap](roadmap.md)` present |
| `existing_intents_cross_referenced` (T-002: INT-0005/0006/0007/0009/0010 carry a roadmap ref) | `grep -l "**Roadmap:** theme"` | **5** of 5 |
| `roadmap_maps_every_gap` (T-001+T-002: every research gap maps to a tracked intent) | manual trace + all 12 intent files resolve | **ok** — see mapping below |

## `roadmap_maps_every_gap` — gap → intent trace
Every gap named in the [research report](../sprint-research/research-report.md)
maps to a tracked intent in [roadmap.md](../../../roadmap.md):
- command sandboxing → INT-0012; supply-chain gate → INT-0013; tamper-evidence → INT-0014; threat-model/OWASP/NIST/CISA + memory-safety + **service transport-auth/identity (mTLS beyond bearer) + env (dev/test/prod) separation** → INT-0015; observability/latency-overhead → INT-0016; approval gates/JIT privilege → INT-0017; parallel orchestration/subagents → INT-0018; concurrency safety → INT-0010; MCP → INT-0005; skills → INT-0006; managed supervision → INT-0007; overlay transport → INT-0009.
- Parking-lot items (memory, programmatic tool results, ACP/A2A, Postgres/queue, vector retrieval) are explicitly listed as deferred-without-a-chapter, not silently dropped.

## Confirmation
```
check-book: valid v2 Book (18 intent chapters)
SUMMARY new-intent links: 7 ; roadmap themes: 4 ; existing cross-refs: 5 ; roadmap intent files resolved: 12
```
