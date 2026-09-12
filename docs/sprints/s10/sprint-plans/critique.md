# Plan Critique — Sprint 10

## Concerns

### C-001: Continuation and cache-scaffolding outcomes lack a stable selected criterion
- **Where:** `build-plan.md` T-003/B and T-003/D; `test-plan.md` Intent Traceability; `INT-0022-completed-contract-repairs.md` Acceptance criteria.
- **Quote:** "a continuation's actual framed prior answer makes initial context exceed budget"; "use actual harness session-prepared requests".
- **Failure mode:** intent-drift
- **Why it matters:** INT-0022's current context criterion owns freeform compaction, checked evidence and replay, but does not explicitly own either actual continuation byte-budget admission or replacement benchmark scaffolding. INT-0026 owns the broader session/cache outcome and remains proposed, so the selected repair intent should state these narrower deliverables and retain the outstanding live, concurrency and token-window evidence.
- **Suggested response:** fix-in-plan — add narrowly scoped acceptance to INT-0022 for actual framed initial byte/request-budget admission and faithful session-prepared cache verification scaffolding; preserve INT-0026 as proposed and never count offline or ignored tests as a live timing result.
- **Disposition:** fixed and accepted on re-review. INT-0022 now explicitly owns framed continuation admission and faithful session-prepared cache verification without claiming live timing/full-history completion; T-003/B–D retain named checks and INT-0026 remains proposed.

### C-002: MCP preparation moves after admission without a named durable-freeze ordering proof
- **Where:** `build-plan.md` T-007/A–B; `test-plan.md` MCP integration and end-to-end cases; `INT-0005-mcp-tool-servers.md` Intent and replay criterion.
- **Quote:** "accepted active preparation SHALL stay within bounded controller ownership"; "the discovered tool set is frozen into the run config at capture".
- **Failure mode:** hidden-dep
- **Why it matters:** Moving discovery beneath controller admission changes the ordering among durable run creation, schema freezing, model dispatch and failure settlement. Valid-run replay plus a no-extra-spawn assertion can pass while discovery failure leaves an admitted run without a defined terminal record, or the first model request precedes durable capture of its authoritative definitions.
- **Suggested response:** fix-in-plan — explicitly require the discovered definitions to be durably recorded and immutable before the first model dispatch, and require startup timeout/cancellation/failure to settle the admitted run without dispatch; add named verification for commit ordering and these terminal outcomes, preserving pure replay and bounded reservations.
- **Disposition:** fixed and accepted on re-review. INT-0022 and T-007/C now require durable definitions before dispatch and terminal startup failure/cancellation with no model/tool effect; `mcp_discovery_freeze_precedes_dispatch` is in traceability and named verification. T-006 also explicitly depends on T-004's shared process-owner integration.

### C-003: The inherited-descriptor research question disappears without disposition
- **Where:** `sprint-research/command-audit.md` "narrow executable/system read grants and exclude private roots"; `build-plan.md` T-005; `test-plan.md` Linux integration cases.
- **Quote:** "Test /proc aliases and a deliberately inherited private descriptor using synthetic data."
- **Failure mode:** missing-risk
- **Why it matters:** The research explicitly distinguishes private-path confinement from already-open descriptor inheritance and says it did not establish an actual application descriptor leak. The current plan lists private-state reads but does not say whether it includes inherited handles/descriptors or defers this unproven surface; successful pathname denial would not answer that question.
- **Suggested response:** fix-in-plan — add a bounded synthetic-descriptor inheritance probe and a required disposition, or explicitly defer this unverified concern with rationale and a named owning intent. Do not turn the uncertainty into a claim that all inherited descriptors are already closed.
- **Disposition:** deferred with rationale and accepted. INT-0025 explicitly owns the descriptor inventory/probe; the plan states that the audit established no application leak, untrusted ambient-descriptor embedding is unsupported, and pathname restrictions do not revoke existing descriptors. T-005 retains pathname/proc-alias checks. This is not a claim that descriptor inheritance has been verified safe.

### C-004: Stable process-cleanup wording exceeds the clarified Unix mechanism
- **Where:** `INT-0022-completed-contract-repairs.md` command acceptance criterion; `build-plan.md` Explicit outstanding capability and evidence.
- **Quote:** "reap descendants"; "orphan zombie reaping belongs to the OS".
- **Failure mode:** intent-drift
- **Why it matters:** The plan now correctly distinguishes terminating the owned process group from waiting on the direct child and from OS reaping of orphan zombies. The selected intent still unconditionally promises descendant reaping, while the trusted MCP no-group-escape boundary exists only in sprint prose.
- **Suggested response:** fix-in-plan — preserve the stop-further-effects and awaited-cleanup guarantees while stating direct-child reaping and OS orphan reaping accurately in INT-0022; record the operator-trusted MCP group boundary in durable consequences.
- **Disposition:** fixed and accepted on final re-review. INT-0022 now requires process-group/job termination, awaited owned handles and direct-child reaping, explicitly assigns Unix orphan-zombie reaping to the OS, and records trusted MCP non-escape/no-subreeper limits in its consequences. T-004/T-006/T-007 share the corresponding ownership dependency.

## Confidence
proceed-with-caveats

All four concerns have accepted dispositions. The descriptor-inheritance inventory,
full session continuity/live cache timing, and verified two-host deployment remain
named proposed outcomes; none may be reported as newly passed by this sprint.
Planned verification labels must be mapped to the actual executed Rust tests or
documented procedures in TEST evidence, including all startup/cancellation branches
under T-007/C. No implementation evidence was used to accept this source-free plan.
