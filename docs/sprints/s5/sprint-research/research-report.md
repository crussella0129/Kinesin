# Sprint 5 Research Report — Production-readiness & security roadmap

## Intents Reviewed
- [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) — created; relevance: the meta-intent this sprint realizes (define the gap-complete, prioritized roadmap); state: `proposed`.
- [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) — reviewed; keep, aligns with SotA (protocol adapters) and OWASP agentic guidance; state `proposed`.
- [INT-0006](../../../intents/INT-0006-skills-progressive-disclosure.md) — reviewed; keep (context/capability layer); `proposed`.
- [INT-0007](../../../intents/INT-0007-managed-model-process.md) — reviewed; keep (execution runtime); `proposed`.
- [INT-0009](../../../intents/INT-0009-koil-overlay-transport.md) — reviewed; keep, deferred behind INT-0008 (done); `proposed`.
- [INT-0010](../../../intents/INT-0010-cross-agent-write-coordination.md) — reviewed; keep, security-relevant (concurrency safety); `proposed`.
- New workstream intents to be authored in the build phase (gaps below): INT-0012 sandboxing, INT-0013 supply-chain, INT-0014 tamper-evident journal, INT-0015 threat model & assurance, INT-0016 observability, INT-0017 approval gates & JIT privilege, INT-0018 subagents/parallel orchestration.

## 1. Sprint Goal
Review the entire codebase and the public State of the Art, then produce a
prioritized, gap-complete roadmap of intents that takes Kinesin from "reference
implementation" to a production-grade, general-purpose local agent harness with
SotA capabilities, hardening, and a very high ("NSA-grade") security bar. The
sprint's deliverable is analysis + a roadmap document + the intent set, not code.

## 2. Existing Code Survey
| File / doc | Relevance | What exists / gap |
|------------|-----------|-------------------|
| src/core.rs, runner.rs, replay.rs | high | Pure decision core + effectful runner + deterministic replay. Strong. Gap: serial single-run only (no subagents/parallelism — deferred in decisions.md). |
| src/tools.rs | high | cap-std capability-scoped file tools + `CommandRunner` (argv-only, env-scrubbed, tree-kill, output/timeout bounds). **Gap: no OS-level syscall/network sandbox** for run_command — security.md §"Future external tools and hostile code" names Landlock/seccomp/AppContainer/Job Objects as required future work. |
| src/policy.rs, verification.rs | high | Immutable `RunAuthority` (capability scoping) + frozen task contracts + independent checker (execution≠acceptance). Strong least-privilege + excessive-agency posture. Gap: no per-action human-approval gate / JIT privilege for irreversible effects. |
| src/storage.rs | high | Transactional immutable SQLite journal, owner-scoped. Gap: **not tamper-evident** — adversarial-review.md: "digests are identity checks, not signed attestations"; a modified DB/compromised controller can invalidate a verdict. |
| src/auth.rs, service.rs, ingress.rs, operator.rs | high | Loopback authenticated service, credential provisioning/rotation, `subtle` constant-time compare, admission/quotas. Gap: no mTLS/identity beyond bearer; env (dev/test/prod) separation is manual. |
| src/scheduler.rs, dispatch.rs, config.rs | medium | Bounded admission, fair capacity, validated config, per-run limits, unbounded-consumption guards. Solid. |
| Cargo.toml | high | rustls (no OpenSSL), bundled sqlite, `subtle`, pinned rust 1.96, edition 2024. **Gap: no supply-chain gate** (cargo-audit/deny/vet), no SBOM, no `cargo-auditable`, no signed releases; `unsafe` FFI via windows-sys/libc undocumented. |
| docs/security.md, decisions.md, adversarial-review.md | high | Threat boundaries, deliberate deferrals, and out-of-scope claims are already articulated — this roadmap formalizes the deferrals into tracked intents. Gap: no OWASP/NIST/CISA mapping; no OpenTelemetry observability; latency overhead unmeasured. |
| tests/live_evaluation.rs, testing.md | medium | Task-card eval harness + adversarial/invariant tests. Gap: no standing red-team/prompt-injection corpus mapped to the release-evidence matrix. |

## 3. External Sources
- [Modern Agent Harness Blueprint 2026](https://gist.github.com/amazingvince/52158d00fb8b3ba1b8476bc62bb562e3) — the 5-layer harness (execution runtime, context, capability, governance, protocol adapters) and plan-in-code parallel subagents with adversarial verification. Kinesin has runtime/capability/governance + CLI+loopback; missing MCP, subagents, richer context/memory, protocol adapters (ACP/A2A).
- [awesome-harness-engineering](https://github.com/ai-boost/awesome-harness-engineering) — SotA feature checklist: evals, memory, MCP, permissions, observability, orchestration.
- [How to sandbox AI agents in 2026 (Northflank)](https://northflank.com/blog/how-to-sandbox-ai-agents) — isolation spectrum: containers (~500ms, insufficient for untrusted) < Landlock+seccomp (~6ms) < gVisor (~100ms) < Firecracker microVM (~125ms, hardware boundary) < WASM (ms). Guides INT-0012.
- [Sandlock: Confining AI Agent Code with Unprivileged Linux Primitives](https://arxiv.org/pdf/2605.26298) — split enforcement: static policy in Landlock+seccomp-bpf plus a seccomp-notify supervisor for runtime decisions; matches security.md's "mandatory isolation refuses launch when a protection is unavailable."
- [Microsoft Rust Engineering — Dependency & Supply-Chain Security](https://microsoft.github.io/RustTraining/engineering-book/ch06-dependency-management-and-supply-chain-s.html) + [cargo-auditable](https://github.com/rust-secure-code/cargo-auditable) — cargo-audit (CVEs), cargo-deny (license/bans/advisories), cargo-vet (source audit, for security-critical), auditable binaries. 2026 TrapDoor campaign shows build.rs exfiltration risk. Guides INT-0013.
- [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/) + [OWASP LLM Top 10 2025](https://aembit.io/blog/owasp-top-10-llm-risks-explained/) — prompt injection, excessive agency (functionality/permissions/autonomy), tool misuse, goal hijacking, memory/context poisoning, unbounded consumption, system-prompt leakage, supply chain. Guides INT-0015/0017.
- [CISA/NSA — The Case for Memory Safe Roadmaps](https://www.cisa.gov/case-memory-safe-roadmaps) + [NIST AI Agent Standards Initiative](https://www.pillsburylaw.com/en/news-and-insights/nist-ai-agent-standards.html) — CISA asks manufacturers to publish a memory-safety roadmap; NIST agent controls: just-in-time privilege, policy-based authz, task scoping, human approval gates, dev/test/prod separation. Kinesin is Rust (memory-safe) but must document its posture + `unsafe` FFI. Guides INT-0015/0017.

## 4. Risks, Unknowns, Dependencies
- **Risk (scope):** a roadmap can sprawl. Mitigation: create only concrete, high-value workstream intents (7), fold speculative items (episodic memory, programmatic tool results, ACP/A2A adapters) into the roadmap doc's "future/parking-lot" rather than chapters.
- **Risk (over-claim):** "NSA-grade security" is a bar, not a certification. The roadmap anchors it to concrete standards (memory safety, OWASP mitigations, supply-chain integrity, sandboxing, tamper-evidence, JIT privilege) rather than a slogan.
- **Unknown (priority):** sequencing among security workstreams — proposed order below is a recommendation, not locked; the operator picks the next sprint.
- **Dependency:** INT-0012 sandboxing hardens INT-0003 (done) and gates INT-0005 (MCP) / any future arbitrary-code tool. INT-0013 supply-chain is a cheap, standalone CI addition (do early). INT-0017 approval gates interact with INT-0010 (coordination) and the write/command tools. INT-0018 subagents interact with the scheduler and INT-0010.
- **Dependency:** this sprint produces no code; its close is the roadmap meta-intent (INT-0011), verified structurally (check-book) and by gap→intent coverage.

## 5. Recommended Approach
**Primary — realize INT-0011 by authoring a prioritized roadmap** organized in four themes, each gap mapped to a tracked intent. Build-phase tasks: (T-001) author the seven new workstream intent chapters; (T-002) write `docs/roadmap.md` (themes, priority, sequencing, SotA/standards mapping) and add roadmap cross-references to the five existing proposed intents.

**Themes & intents:**
- **A. Security hardening (NSA-grade):** INT-0012 command-execution sandboxing (Landlock+seccomp / AppContainer+Job Object, mandatory-isolation mode; microVM/WASM as heavier options); INT-0014 tamper-evident audit journal (hash-chained events + signed receipts); INT-0017 human-approval gates & just-in-time privilege for irreversible actions.
- **B. Supply-chain & assurance:** INT-0013 supply-chain security & signed releases (cargo-deny+audit CI gate, cargo-auditable, SBOM, cosign/sigstore, reproducible builds, cargo-vet for the security-critical posture); INT-0015 threat model & security assurance (OWASP LLM/Agentic + NIST controls mapping, CISA/NSA memory-safety statement documenting `unsafe` FFI, standing red-team/prompt-injection corpus tied to security.md's release-evidence matrix).
- **C. Operability / production:** INT-0016 observability (OpenTelemetry traces + metrics that never contain prompts; measure the runtime-overhead adversarial-review flagged as unknown).
- **D. SotA capability:** INT-0018 subagents & bounded parallel orchestration (the deferred parallel scheduler; bounded fan-out with the run-authority model preserved), alongside existing INT-0005 (MCP), INT-0006 (skills), INT-0007 (Kineserve), INT-0009 (Koil), INT-0010 (coordination).

**Recommended sequencing (next sprints):** INT-0013 (cheap, high assurance value) → INT-0012 (unblocks safe code/MCP execution) → INT-0015 (assurance/threat model) → INT-0005/0006 (SotA capability) → INT-0016, INT-0017, INT-0010, INT-0018 as capability grows.

**Alternative considered:** a single monolithic "harden everything" intent — rejected: it is unschedulable and untestable; distinct outcomes need distinct intents (Book rule). Parking-lot (roadmap doc, not chapters yet): episodic/procedural memory, programmatic (file-pointer) tool results, ACP/A2A protocol adapters, Postgres/distributed queue for multi-host controllers.

## Artifacts
- No code artifacts. The code survey (Section 2) is inline against the current tree; the roadmap intents and `docs/roadmap.md` are authored in the build phase per Section 5.

## Budget Override
This is a deliberately cross-cutting roadmap review spanning the whole codebase
and the public SotA, so it exceeds the default caps: ~15 code/doc files surveyed
(within the 20 cap) and **7 external sources** (over the 5-source cap). The extra
sources are load-bearing — sandboxing, supply-chain, agent-security, and the
memory-safety/NIST standards are separate domains that each anchor a distinct
roadmap workstream, and none is redundant.
