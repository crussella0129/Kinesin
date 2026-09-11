# Kinesin Roadmap — to a production-grade, secure local agent harness

This roadmap turns the sprint 5 review (whole-codebase + public State of the Art +
security standards) into a prioritized, tracked path from the current reference
implementation to a production-grade, general-purpose local agent harness with
SotA capabilities, hardening, and a very high ("NSA-grade") security bar. It is
the working view of the meta-intent [INT-0011](intents/INT-0011-production-readiness-roadmap.md);
the intent chapters remain the authority, this doc is navigation + sequencing.
Provenance: [sprint 5 research report](sprints/s5/sprint-research/research-report.md).

## Where we are

**Realized:** token accounting (INT-0001), context compaction (INT-0002), bounded
command execution (INT-0003), KV-cache reuse (INT-0004), uniform local/remote
model transport (INT-0008). The runtime has a pure decision core, deterministic
replay, capability-scoped file + command tools, a transactional owner-scoped
SQLite journal, an authenticated loopback service with admission/quotas, and
streaming. It is memory-safe by construction (Rust; `rustls`, no OpenSSL).

**The gap to production:** OS-level sandboxing of executed commands; a
supply-chain integrity gate; a tamper-evident, attestable journal; a
standards-mapped threat model; production observability; per-action human
oversight / just-in-time privilege; and SotA scaling (MCP, skills, subagents).

## Themes and intents

### A. Security hardening (NSA-grade)
- [INT-0012](intents/INT-0012-command-execution-sandboxing.md) — command-execution sandboxing (Landlock+seccomp / AppContainer+Job Object; mandatory-isolation mode). **proposed**
- [INT-0014](intents/INT-0014-tamper-evident-journal.md) — tamper-evident audit journal (hash-chain + signed receipts). **proposed**
- [INT-0017](intents/INT-0017-approval-gates-jit-privilege.md) — human-approval gates & just-in-time privilege. **proposed**
- [INT-0010](intents/INT-0010-cross-agent-write-coordination.md) — cross-agent write coordination (presence-aware leases; concurrency safety). **proposed**

### B. Supply-chain & assurance
- [INT-0013](intents/INT-0013-supply-chain-security.md) — supply-chain security & signed releases (cargo-deny/audit/auditable/vet, SBOM, cosign). **proposed**
- [INT-0015](intents/INT-0015-threat-model-assurance.md) — threat model & security assurance (OWASP/NIST/CISA mapping, memory-safety statement, red-team corpus). **proposed**

### C. Operability / production
- [INT-0016](intents/INT-0016-observability.md) — observability (OpenTelemetry traces + metrics, no prompts; measures runtime overhead). **proposed**
- [INT-0007](intents/INT-0007-managed-model-process.md) — managed local model-process supervision (Kineserve). **proposed**

### D. SotA capability
- [INT-0005](intents/INT-0005-mcp-tool-servers.md) — MCP tool-server integration. **proposed**
- [INT-0006](intents/INT-0006-skills-progressive-disclosure.md) — skills & progressive disclosure. **proposed**
- [INT-0009](intents/INT-0009-koil-overlay-transport.md) — Koil pure-Rust overlay transport (own repo; behind Tailscale). **proposed**
- [INT-0018](intents/INT-0018-subagents-parallel-orchestration.md) — subagents & bounded parallel orchestration. **proposed** (lowest priority)

## Recommended sequencing
1. **INT-0013** supply-chain gate — cheap, high assurance value, standalone.
2. **INT-0012** command sandboxing — unblocks safe execution and gates MCP/code tools.
3. **INT-0015** threat model & assurance — captures the posture; consumes 0012/0017 as controls mature.
4. **INT-0005 / INT-0006** — SotA capability (MCP, skills), executed under 0012's sandbox.
5. **INT-0016** observability, **INT-0017** approval gates/JIT, **INT-0014** tamper-evident journal, **INT-0010** coordination — as concurrency and shared-service use grow.
6. **INT-0007** Kineserve, **INT-0009** Koil, **INT-0018** subagents — capability depth once the security foundation holds.

The order is a recommendation; the operator picks each sprint's objective.

## SotA & standards mapping
| Source | What it establishes | Kinesin response |
|--------|--------------------|------------------|
| 5-layer harness (runtime/context/capability/governance/adapters) | modern harness shape | have runtime/capability/governance + CLI/loopback; adapters via INT-0005 (MCP), context via INT-0006 |
| Sandboxing spectrum (Landlock/seccomp → gVisor → Firecracker → WASM) | untrusted execution needs OS/HW isolation | INT-0012 (Landlock+seccomp first tier; microVM/WASM as heavier options) |
| Rust supply-chain (cargo-deny/audit/vet, SLSA, cosign) | dependency & release integrity | INT-0013 |
| OWASP LLM 2025 / Agentic 2026 (prompt injection, excessive agency, tool misuse, unbounded consumption, poisoning) | agent threat taxonomy | content-as-data + capability scoping + admission bounds (have); INT-0015 maps + tests; INT-0017 for excessive agency |
| CISA/NSA memory-safe roadmap; NIST AI-agent controls (JIT privilege, approval gates, task scoping, env separation) | manufacturer security expectations | Rust (memory-safe) + INT-0015 statement; INT-0017 for gates/JIT; INT-0014 for attestable records |
| OpenTelemetry (standard agent-runtime telemetry) | operability | INT-0016 |

## Parking-lot (deferred without a chapter yet)
Authored as an intent only when a sprint selects one:
- Episodic / procedural **memory** (recall past runs; tool-use heuristics).
- **Programmatic (file-pointer) tool results** — revisit when a result must exceed the context (decisions.md).
- **ACP / A2A protocol adapters** — additional surface adapters beyond CLI + loopback + MCP.
- **PostgreSQL + distributed queue** — when controllers/durable workers span hosts (decisions.md).
- Embedding / vector retrieval — only for a paraphrased-evidence task class (decisions.md, measured against inline grep).
