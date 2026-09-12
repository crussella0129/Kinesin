# Kinesin roadmap

This is the working roadmap for the main local agent harness. Intent chapters
own requirements and state; this page organizes dependencies and priorities.
[INT-0021](intents/INT-0021-harness-contract-review.md) supersedes the original
roadmap/assurance revision while retaining [sprint 5's review](sprints/s5/sprint-research/research-report.md)
and [sprint 10's intent-first audit](sprints/s10/sprint-research/research-report.md).

## Current position

Implemented foundations include a pure decision core, frozen run authority,
independent checked acceptance, immutable SQLite settlement, crash recovery,
retention/backup, pure replay, streaming, bounded independent concurrent runs
and an authenticated owner-scoped loopback API. File tools, argv commands and
local stdio MCP extend the effect surface. These mechanisms have useful tests;
they do not establish all proposed production outcomes.

[INT-0022](intents/INT-0022-completed-contract-repairs.md) owns sprint 10's repairs
to token accounting, compaction/continuation byte admission, command output and
process lifetime, Linux confinement, MCP bounds/admission, confidential model
origins, atomic destination no-replace and dependency policy. Its chapter and
sprint test evidence determine completion; a listed repair is not a passed test.

The main remaining gaps are Windows filesystem/network isolation, usable
session continuity and real token-window admission, per-action approvals,
shared-write fencing, operational/credential stewardship, realistic coding
evaluations, telemetry and verified encrypted two-host deployment. Read the
[threat model](threat-model.md) for exact controls and residuals.

## Four work themes

### A. Security hardening

| Intent | State / responsibility |
|---|---|
| [INT-0012](intents/INT-0012-command-execution-sandboxing.md) | Historical realized Linux Landlock/seccomp tier; confinement and lifecycle repairs are owned by INT-0022. |
| [INT-0019](intents/INT-0019-windows-command-sandboxing.md) | Proposed Windows AppContainer/LPAC confinement. Existing Jobs provide lifecycle control only. |
| [INT-0017](intents/INT-0017-approval-gates-jit-privilege.md) | Proposed per-effect approval and narrowed temporary authority, including bounded timeout/replay behavior. |
| [INT-0010](intents/INT-0010-cross-agent-write-coordination.md) | Proposed fenced leases, presence, fair handoff and stale-holder prevention. The repaired move collision contract does not implement shared-write coordination. |
| [INT-0014](intents/INT-0014-tamper-evident-journal.md) | Proposed hash-chain and signed receipt verification, with trust/key/rollback boundaries. |
| [INT-0025](intents/INT-0025-secrets-and-egress.md) | Proposed cross-adapter credential/egress lifecycle and complete inheritance inventory. INT-0022 repairs environment/stderr defaults and Linux command non-stdio descriptor closure with a synthetic probe; Windows handles and full embedding policy remain open. |

### B. Supply chain and assurance

| Intent | State / responsibility |
|---|---|
| [INT-0013](intents/INT-0013-supply-chain-security.md) | Historical realized dependency gate; INT-0022 repairs its non-blocking duplicate policy. [Dependency policy](supply-chain.md) owns exact exceptions, native builds and the cargo-vet decision. |
| [INT-0021](intents/INT-0021-harness-contract-review.md) | Current sprint's contract baseline and roadmap/assurance revision; all original acceptance criteria are assessed before deciding completion. |
| [INT-0022](intents/INT-0022-completed-contract-repairs.md) | Current sprint's completed-contract repairs and adversarial regression evidence. |
| [INT-0024](intents/INT-0024-harness-evaluation.md) | Proposed coding-task evaluation with frozen workspaces, independent checks, resource ceilings and retained failures. Scripted gate tests and live task quality are separate evidence. |

### C. Operability and deployment

| Intent | State / responsibility |
|---|---|
| [INT-0023](intents/INT-0023-lifecycle-data-stewardship.md) | Proposed complete lifecycle/data stewardship, restore drills and version-compatibility matrix. Existing recovery, backup, retention and owner isolation are the starting point. |
| [INT-0016](intents/INT-0016-observability.md) | Proposed off-by-default OpenTelemetry export with content-free labels and measured runtime overhead. Existing counters are not exported OTel traces. |
| [INT-0007](intents/INT-0007-managed-model-process.md) | Proposed managed local llama-server startup/readiness/cleanup. Existing model support attaches to an externally managed endpoint. |
| [INT-0027](intents/INT-0027-encrypted-remote-deployment.md) | Proposed verified encrypted deployment between two physical hosts. HTTPS URL validation is implemented separately from server binding/firewall/tunnel evidence. |

### D. Harness capability

| Intent | State / responsibility |
|---|---|
| [INT-0005](intents/INT-0005-mcp-tool-servers.md) | Historical realized local stdio MCP; INT-0022 repairs protocol bounds, durable discovery, admission, process lifetime and independent grants. Servers remain operator-trusted native programs. |
| [INT-0020](intents/INT-0020-remote-mcp-delegated-auth.md) | Proposed Streamable HTTP MCP and destination-scoped OAuth authorization, with no inbound-token passthrough. Depends on confidentiality and credential controls. |
| [INT-0006](intents/INT-0006-skills-progressive-disclosure.md) | Proposed installed skills with frozen origin/digest and bounded disclosure; workspace files do not install policy. |
| [INT-0026](intents/INT-0026-session-context-continuity.md) | Proposed bounded session continuity, actual model token-window admission and faithful live session cache/concurrency measurements. |
| [INT-0009](intents/INT-0009-koil-overlay-transport.md) | Proposed build/adopt decision and optional pure-Rust overlay integration. The model module's Koil role name is not an implemented VPN. |
| [INT-0018](intents/INT-0018-subagents-parallel-orchestration.md) | Proposed model-spawned child runs with narrower authority and aggregate budgets. Independent concurrent runs already exist; child coordination depends on INT-0010. |

## Historical intent accounting

[INT-0001](intents/INT-0001-token-accounting.md), [INT-0002](intents/INT-0002-context-compaction.md)
and [INT-0003](intents/INT-0003-shell-execution.md) retain their original realized
history; INT-0022 records current repairs without deleting their acceptance
criteria or old test results.

[INT-0004](intents/INT-0004-kv-cache-reuse.md) is superseded by INT-0026: the old
benchmark used a hand-built request sequence rather than actual harness session
assembly. Its recorded timing remains historical evidence for that workload,
not proof of the original session/concurrency outcome.

[INT-0008](intents/INT-0008-remote-model-over-overlay.md) is superseded by INT-0027:
the uniform client seam exists, but a same-host LAN request and private address
classification did not establish cross-host encrypted/private deployment.
Non-loopback client URLs now require HTTPS; public destination opt-in is separate
from encryption. No two-host proof is inferred from that client check.

The operator subsequently supplied nighthawk during sprint 10. A
[scoped actual deployment](sprints/s10/sprint-tests/remote-deployment.md) passed a
checked run and replay through authenticated SSH forwarding, with direct model
access rejected. Actual session requests also reused their shared prefix. The
broader INT-0026/0027 criteria remain proposed; neither a universal deployment
guarantee nor a causal speedup from the cache flag is claimed.

[INT-0011](intents/INT-0011-production-readiness-roadmap.md) and
[INT-0015](intents/INT-0015-threat-model-assurance.md) are superseded by INT-0021's
current revision. Their earlier roadmap and assurance evidence stays linked.

## Recommended next work

1. Finish and verify INT-0021/0022's repair checkpoint, including native Windows
   and Linux CI, durable MCP ordering and fresh dependency checks.
2. Prioritize INT-0019 for the operator's Windows command isolation, and
   INT-0026/0024 for session correctness and measurable coding-task reliability.
3. Establish INT-0010 and INT-0017 before expanding concurrent mutations and
   high-impact autonomous tools. Fencing and explicit approval address different
   failure modes and need separate tests.
4. Complete INT-0023/0025's data/credential lifecycle and INT-0016 observability.
   Extend INT-0027 from the verified nighthawk configuration to its remaining
   deployment and capacity criteria; preserve the scope of the existing proof.
5. Add skills (INT-0006), managed inference (INT-0007) and remote MCP (INT-0020)
   as measured workloads justify them, preserving their prerequisite boundaries.
6. Add signed journal authenticity (INT-0014) with an explicit deployment-key
   policy. Consider Koil (INT-0009) only after a verified remote deployment
   establishes a need beyond the adopted transport. Subagents (INT-0018) follow
   functioning shared-write coordination and aggregate resource budgets.

The operator selects each sprint. This is a dependency-aware recommendation, not
authorization to implement every proposal in the current repair sprint.

## Standards and evidence

| Source or review dimension | Tracked response |
|---|---|
| Runtime, context, capability, authority and adapter responsibilities | Existing core/journal/controller plus INT-0023/0026/0006/0017/0020. |
| Linux and Windows confinement | INT-0012/0022 and INT-0019; platform behavior must be tested on that platform. |
| Dependency/native-build and release integrity | INT-0013/0022 and the [dependency policy](supply-chain.md); release artifacts remain below. |
| OWASP LLM 2025 and Agentic 2026 | Individually mapped by [threat-model.md](threat-model.md) under INT-0021, with explicit gaps and residuals. |
| Least privilege, oversight and memory-safe engineering | Frozen grants plus INT-0017; first-party unsafe and native dependencies are inventoried without claiming the whole process is memory-safe. |
| Runtime performance and model competence | INT-0016 telemetry, INT-0024 evaluations and INT-0026 actual-session cache measurements. |

## Parking lot

These are not implemented commitments. Author a chapter when selecting the
distinct outcome:

- Release-artifact integrity: SBOMs, auditable binaries, reproducible builds and
  signing, requiring a release pipeline and key-management decision.
- Episodic/procedural memory beyond the bounded continuity of INT-0026.
- File-pointer tool results when a measured workload exceeds bounded inline data.
- ACP/A2A adapters beyond the CLI, loopback API and MCP.
- Distributed controllers or PostgreSQL/queues when one controller no longer
  meets measured capacity requirements.
- Embedding/vector retrieval for a task class that benefits beyond literal search.
