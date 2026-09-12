# Sprint 10 — assurance and foundational-runtime audit

Research audit performed after the frozen intent-only baseline. This is a
criterion-level review of INT-0011, INT-0013, INT-0015, INT-0007, INT-0009 and
INT-0016, plus existing lifecycle, data-management, owner/API and scheduler
mechanisms. It changes no implementation or historical intent. File/line
references identify the pre-build tree; later edits may move lines.

Status meanings: **satisfied** = implementation/artifact directly matches the
criterion; **partial** = a substantive part exists but the full criterion is not
established; **absent** = not implemented; **unverified** = requires fresh
execution or external evidence. Existing tests were inspected; this sub-audit
did not re-run Rust test suites. The sprint's TEST phase owns fresh execution.

## INT-0011 — realized roadmap

| Criterion | Assessment | Evidence and limitation |
|---|---|---|
| Reachable roadmap with four themes and sequencing | Satisfied structurally; stale operational content | `docs/roadmap.md:25` has themes A–D and `:49` sequencing; `docs/SUMMARY.md` links the roadmap. Opening `:13`/`:20` still lists sandboxing, supply-chain gate, threat model and MCP as gaps despite realized theme rows at `:28`, `:35`, `:36`, `:43`. Sequencing repeats delivered work. |
| Each sprint 5 research gap maps to authored intent | Satisfied for that historical review; partial for current harness | `docs/sprints/s5/sprint-research/research-report.md:23` surveys mechanisms/gaps; roadmap themes map its sandbox, assurance, supply-chain, observability, approval, MCP, skills and subagent gaps. Release-artifact work is explicitly parked following the INT-0013 split. Foundational run/data outcomes highlighted by the frozen s10 baseline still lack intent ownership. |
| Five pre-existing proposed chapters reviewed/cross-referenced; new chapters well formed | Satisfied | INT-0005/0006/0007/0009/0010 carry roadmap references; INT-0012..0018 have acceptance/rationale/alternatives/consequences. Historical research records their review. |
| Book validator accepts expanded set | Historical evidence exists; fresh result owned by root sprint verification | INT-0011 cites s5 head `84fb7a3`; this audit does not infer current Book validity from an old pass. |

Repair through a follow-on roadmap revision, preserving the realized chapter and
s5 provenance. Bring current snapshot, remaining-work order, transport-evidence
limitations and newly owned foundational outcomes into agreement.

## INT-0013 — realized dependency gate

| Criterion | Assessment | Evidence and limitation |
|---|---|---|
| Vulnerabilities and every named policy violation fail CI | Partial; duplicate requirement contradicted | `.github/workflows/ci.yml:40` and `:42` execute deny/audit without error suppression. `deny.toml:15`/`:42` constrain licenses/sources, but `:37` makes duplicate versions warnings. `cargo deny --offline check bans` executed during this audit, exited **0**, and reported duplicates for base64, hashbrown, io-lifetimes, nix, syn and windows-sys. INT-0013:26 explicitly includes duplicate crates in blocking violations. |
| Gate starts green; advisory exceptions justified | Bans currently green with warnings; complete gate needs fresh check | The above invocation is current evidence for bans only. `deny.toml:13` has no advisory ignores. S9 reports complete deny/audit success; do not present that as a fresh advisory-database evaluation. |
| Committed/reviewed lockfile and build-script inventory | Partial | `Cargo.lock` is present and reviewed by prior dependency sprints; s6 research records no first-party `build.rs`. Native/build dependencies have evolved and need a durable inventory. Current `cargo tree --locked --offline --duplicates` demonstrates aws-lc-sys via reqwest/rustls and libsqlite3-sys via rusqlite, with cc/cmake build dependencies. Source/advisory checks cannot establish that new malicious build code is harmless. |
| Documented cargo-vet decision, rationale and adoption trigger | Partial | INT alternatives and s6 research defer cargo-vet for audit friction, but no concrete adoption/reconsideration trigger is recorded in durable guidance. |

Concrete repair: deny unreviewed duplicates, use version-specific exceptions with
dependency-path rationale for unavoidable existing duplicates, and prove a
negative path with an isolated temporary policy against the existing lockfile.
No vulnerable crate needs to be added to the repository. Record the cargo-vet
decision/trigger and native build surface. Moving action tags were explicitly
accepted in the s6 critique; they are not a newly discovered runtime defect.

## INT-0015 — realized threat assurance

| Criterion | Assessment | Evidence and limitation |
|---|---|---|
| Each OWASP LLM/Agentic risk plus NIST controls gets mechanism/gap/residual disposition; service identity boundary named | Partial | `docs/threat-model.md:8` combines risks in a ten-row table, without the complete separate Agentic taxonomy. ASI07, ASI08, ASI09 and ASI10 lack explicit dispositions. The NIST control rows at `:23` and loopback/mTLS residual at `:32` exist. `src/config.rs` validates the loopback service and `src/service.rs:195` authenticates before requests reach storage. |
| Every unsafe/FFI surface has justification and containment | Partial | `docs/threat-model.md:34` inventories four first-party files, matching production unsafe locations in private_state/storage/signal/tools. It omits the native dependency surface: `Cargo.lock:54` aws-lc-sys, `:1025` libsqlite3-sys and `:1367` ring (target/feature relevance must be identified from the graph). Tests also contain unsafe environment mutations and Windows signal fixtures; scope them explicitly instead of claiming the four rows enumerate every block. |
| CI corpus maps release rows and proves forbidden effects denied | Substantive mechanism evidence; narrow corpus claims need correction | `.github/workflows/ci.yml:24` runs ordinary tests on Windows/Linux. `tests/redteam.rs:49` proves an ungranted executable is denied; `:70` proves shell-like argv stays literal or the mandatory sandbox refuses execution. `tests/service.rs:353` proves owner-route isolation. `tests/live_evaluation.rs:425` executes scripted cards; real model cards at `:436` are ignored by normal CI. Scripted hostile-content responses do not prove a model obeys instructions in the presence of injection. |
| Version, owner and update cadence | Satisfied structurally; follow-on update now due | `docs/threat-model.md:3` names version 1, maintainers and security-sprint/release review. S9 introduced trusted MCP processes and schemas; the threat package needs an explicit current disposition for those capabilities. |

Primary-source check: the [official Agentic 2026 publication](https://genai.owasp.org/download/52117/?tmstv=1765059207)
enumerates ASI01–ASI10 separately. The four omitted dimensions concern
inter-agent messages, failure propagation, user trust, and agents departing
their assigned role. The [official LLM 2025 index](https://genai.owasp.org/llm-top-10/)
also separates model/data poisoning from prompt injection. Explicit N/A or
residual rows are valid when the feature is absent; omission is not a disposition.

Concrete repair: publish a provenance-preserving assurance revision with separate
LLM01–LLM10 and ASI01–ASI10 mappings, concrete test names, MCP trust boundaries,
native-dependency/build inventory and precise empirical limits. Replace blanket
“all content is never instructions” phrasing with the enforceable fact that
content does not grant capabilities; models can still be influenced within the
granted capability set. Preserve historical model measurements separately from
scripted control tests. This is documentation/control-evidence repair, not a
request to implement absent multi-agent or approval features this sprint.

## Proposed intents — implementation assessment

| Intent / criterion | Assessment | Evidence |
|---|---|---|
| INT-0007 managed spawn, health-ready admission, bounded never-ready error | Absent managed mode; existing attach health is partial groundwork | `src/config.rs:263` models contain endpoint/model/context/slot/cache options, no executable/model-file/managed-mode configuration. `src/operator.rs:115` builds HTTP clients; `:220` monitors attached endpoint readiness. |
| INT-0007 model process-tree cleanup on shutdown | Absent for models | Service shutdown joins controller/writer; command tools have process-tree machinery, but no owned model child exists to supervise/reap. |
| INT-0007 preserve external attach endpoint | Satisfied existing behavior | CLI `src/cli.rs:606` and service startup build `ModelClient::http`; neither owns or stops the external inference process. |
| INT-0007 readiness/death/tree tests | Absent managed-model tests | `src/operator.rs` tests supervise readiness worker failures, not llama-server process lifecycle. Leave proposed. |
| INT-0009 build-vs-adopt decision with criteria | Partial prose, no completed decision/evidence | Intent rationale/alternatives favor Tailscale first; `docs/decisions.md` does not record the specified assessed Koil decision. |
| INT-0009 built transport confidentiality/two-host proof | Absent in this repository | No Koil transport dependency in `Cargo.toml`; `src/model.rs:1` uses “Koil” as a module role label, not a WireGuard implementation. External repository was not audited. |
| INT-0009 runner-independent transport seam | Partial architecture groundwork | Runner depends on `ModelClient`; overlay-address policy is in `src/config.rs:997`. No implemented Koil/Tailscale interface was found. |
| INT-0009 handshake/round-trip/live integration proofs | Absent Koil proof | `tests/live_evaluation.rs:531` is an ignored same-server LAN/tailnet endpoint comparison, not a Koil handshake or demonstrated two-host encrypted link. Leave proposed. |
| INT-0016 configurable OTel exporter, off by default | Absent | No OTel dependency/exporter configuration or first-party tracing calls in Cargo.toml/src. |
| INT-0016 telemetry privacy scan | Absent exporter test | Existing count-only `src/service.rs:103`, `src/scheduler.rs:88` and storage stats avoid content, but no emitted OTel stream exists to scan. |
| INT-0016 measured runtime overhead separate from inference | Partial performance groundwork | `docs/performance-baseline.md` has prior experiments; existing counters are not an OTel stage-level baseline for queue/scheduler/tool/journal overhead. |
| INT-0016 owner-scoped run correlation | Absent OTel correlation | Owner/run scope exists in journal and service, not exported traces. Leave proposed. |

## Foundational outcomes already implemented without dedicated intents

| Outcome | Present mechanisms and evidence | Remaining contract gap |
|---|---|---|
| Durable lifecycle and recovery | `src/storage.rs:751` atomically classifies unfinished runs interrupted, checked assessment inconclusive, pending effects unknown. `tests/process_recovery.rs:305`, `:309`, `:313` kill a real child after admission/effect intent/checker result. Atomic terminal commit at `src/storage.rs:691`; cancellation/settlement tests in `tests/settlement.rs` and scheduler. | Name and preserve this behavior in a foundational intent. Recovery intentionally does not re-execute uncertain effects or claim resumability (`docs/architecture.md:230`). Crash proofs are not physical power-loss tests. |
| Private persistent data and credentials | `src/operator.rs:23` validates state/verifier trees for service; `src/auth.rs` provisions hashed verifiers; `src/storage.rs:1041` rejects private observations in metadata capture. Final results remain private in both capture modes. | Dedicated owner for confidentiality/capture/retention/restore/upgrade policy. Do not promise disk encryption or secure erasure: neither is implemented. Local operator filesystem permissions remain documented responsibility. |
| Retention and recovery-capable backups | `src/storage.rs:474` bounded terminal deletion with minimum window; `:490` SQLite backup/integrity check; `:1701` verifies restored receipts and interrupted unfinished runs. `tests/cli_inspect.rs:376` covers overwrite/owner/retention behavior. `docs/cli.md:174` explains operator-wide scope and physical-disk limitations. | No automated retention scheduler or schema migration chain; these are absent capabilities, not regressions against current explicit policy. Unknown schemas fail closed (`src/storage.rs:395`, test `:2314`). Model/export data has no blanket secure-erasure claim. |
| Owner API and admission | `src/service.rs:125` current-policy/owner filtering, `:195` authenticate-before-decode, `:161` owner routes. `tests/service.rs:281`, `:353`, `:426`, `:453`, `:570` cover ambiguous credentials, isolation, retry races, revocation and cancellation. | Explicit durable acceptance for API contract, identity, idempotency and retrieval continuity. External service exposure remains loopback-only; do not confuse remote model transport with remote service authentication. |
| Bounded independent-run scheduler | `src/scheduler.rs:88` counters, `:129` per-owner readiness, `:170` retained reservation ownership, `:518` dispatch and `:573` controller. Tests `:727`, `:789`, `:845`, `:972`, `:1064` cover bounds, overlap, writer failure, dropped admission receiver and owner fairness. | Name the existing concurrent-run contract separately from INT-0018 model-spawned parent/child subagents, which remains proposed. Existing concurrent runs are not evidence of subagent orchestration. |

## Prioritized repair findings

1. **P1 — assurance overclaims:** complete the standards mapping and native FFI
   inventory; make model-behavior versus deterministic-gate evidence explicit.
   Existing declarations of complete assurance are materially broader than the
   documented controls/evidence.
2. **P2 — duplicate gate is non-blocking:** strengthen duplicate policy with
   reviewed exceptions and an actual isolated failure proof; add durable
   cargo-vet trigger and build-surface inventory.
3. **P2 — roadmap maintenance drift and missing foundational ownership:**
   follow-on revision with current delivered/pending distinction and intents for
   already substantive runtime/data/API guarantees plus explicitly unbuilt gaps.
4. **Keep proposed work proposed:** managed inference, bespoke encrypted overlay
   and OTel remain mostly absent. Do not inflate the repair sprint into their
   implementation or certify their acceptance from nearby mechanisms.

Commands executed: `cargo tree --locked --offline --duplicates` (success),
`cargo deny --version` (0.20.2), `cargo audit --version` (0.22.2), and
`cargo deny --offline check bans` (exit 0 with duplicate warnings). Full
advisory/license/source checks, Rust tests, Book checks and CI are intentionally
not claimed as freshly passed by this document.
