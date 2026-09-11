# Sprint 8 Research Report — Threat model & security assurance (INT-0015)

## Intents Reviewed
- [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) — selected; roadmap theme B, the strategically-valuable consolidation of the security posture after the sandbox (INT-0012) and supply-chain gate (INT-0013) shipped; state `proposed`. SotA/standards basis carried from the [sprint 5 research report](../../s5/sprint-research/research-report.md).

## 1. Sprint Goal
Produce the security-assurance package: a `docs/threat-model.md` mapping the
design onto OWASP LLM Top-10 (2025) / OWASP Agentic Top-10 (2026) and the NIST
AI-agent controls (with the service transport-auth/identity boundary), a
CISA/NSA-aligned memory-safety statement enumerating the `unsafe`/FFI surface,
and a standing red-team corpus (prompt-injection + authorization-bypass) mapped
to security.md's release-evidence matrix and running in CI. This is primarily
documentation plus a small consolidating test; it introduces no runtime feature.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| src/private_state.rs | high | 30 `unsafe` sites — Windows security-descriptor/SID/token FFI (windows-sys) + `libc::geteuid` (unix). The bulk of the memory-safety statement. |
| src/storage.rs (3), src/signal.rs (1), src/tools.rs (1) | high | `unsafe`: Windows `CreateDirectoryW` w/ security attrs (storage), `SetConsoleCtrlHandler` (signal), the Linux sandbox `pre_exec` closure (tools). |
| docs/security.md | high | The release-evidence matrix (Tool authority, Untrusted content, Task acceptance, Data handling, Concurrent runs, Shared owners, Authentication, Load/storage) — the rows the red-team corpus must map to. |
| docs/adversarial-review.md | high | The attack/failure matrix + "what remains outside the claim" — feeds the threat model's residual-risk column. |
| tests/adversarial_runtime.rs, command_tool.rs, sandbox_linux.rs, service.rs, settlement.rs, replay.rs, live_evaluation.rs (hostile-text cards) | high | The **already-executing** corpus: DoS/capacity bounds, command bounds + sandbox denial, auth + owner-scoping, forged-verdict/cross-owner-evidence rejection, prompt-injection-as-data. INT-0015 maps these to the matrix + adds any gap, rather than duplicating. |
| docs/intents/INT-0015, docs/roadmap.md | high | The intent (incl. the transport-auth boundary added in the sprint-5 ultrareview) + roadmap placement. |

## 3. External Sources
- [OWASP Top 10 for LLM Applications 2025](https://genai.owasp.org/llm-top-10/) and [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/) — the risk taxonomy to map (prompt injection, excessive agency, tool misuse, sensitive-info disclosure, unbounded consumption, system-prompt leakage, supply chain, memory/context poisoning, goal hijacking).
- [NIST AI Agent Standards Initiative](https://www.pillsburylaw.com/en/news-and-insights/nist-ai-agent-standards.html) — controls: just-in-time privilege, policy-based authorization, task scoping, human-approval gates, dev/test/prod separation.
- [CISA/NSA — The Case for Memory Safe Roadmaps](https://www.cisa.gov/case-memory-safe-roadmaps) + [NSA Memory Safe Languages CSI](https://media.defense.gov/2023/Apr/27/2003210083/-1/-1/0/CSI_SOFTWARE_MEMORY_SAFETY_V1.1.PDF) — the manufacturer memory-safety-statement expectation the statement satisfies.
- (Full framing carried from the [sprint 5 survey](../../s5/sprint-research/research-report.md#3-external-sources).)

## 4. Risks, Unknowns, Dependencies
- **Risk (over-claim):** a threat model can imply more assurance than exists. Mitigation: every row is a mechanism, a **gap** (pointing to the owning intent, e.g. INT-0017 for approval gates, INT-0014 for tamper-evidence, INT-0019 for Windows sandbox), or an **accepted residual risk** — never "covered" without evidence.
- **Risk (corpus duplication):** re-implementing existing adversarial tests wastes effort and drifts. Mitigation: the corpus is a **mapping** of matrix rows → the tests already proving them, plus a small `tests/redteam.rs` only for genuinely-unmapped negatives.
- **Unknown (mapping completeness):** bounded by the review's thoroughness (same honest caveat as the roadmap sprint); anchored to the two OWASP lists + NIST + the security.md matrix.
- **Dependency:** none blocking; it consolidates shipped work. It documents gaps owned by other (proposed) intents without implementing them.

## 5. Recommended Approach
**Primary — author the assurance package (docs) + a consolidating corpus map/test.**
1. `docs/threat-model.md`: (a) an OWASP LLM/Agentic risk table → Kinesin mechanism / gap(owning intent) / residual risk; (b) a NIST-control table (incl. the service transport-auth/identity boundary: bearer-over-loopback now, mTLS before non-loopback exposure); (c) a **memory-safety statement** enumerating the `unsafe`/FFI sites (private_state.rs, storage.rs, signal.rs, tools.rs sandbox, libc) with justification + containment, satisfying the CISA/NSA expectation; (d) a versioning/owner/update-cadence header.
2. A **red-team corpus map**: a section (and a small `tests/redteam.rs`) tying each security.md release-evidence-matrix row to the executed test(s) proving it, adding a focused authorization-bypass and prompt-injection-as-data test only where a row is unmapped.
3. Link `docs/threat-model.md` from `docs/SUMMARY.md`.

**Alternative considered:** a heavyweight standalone red-team framework (rejected — the existing tests are the corpus; formalize the mapping, don't rebuild). A third-party audit (later; presupposes this internal package).

## Artifacts
- No code pre-authored; `docs/threat-model.md`, the SUMMARY link, and `tests/redteam.rs` are build-phase deliverables.
