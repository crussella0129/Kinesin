# INT-0015 — Threat model & security assurance

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0015
- **State:** realized
- **Work evidence:** [T-001 build plan](../sprints/s8/sprint-plans/build-plan.md#t-001-author-the-assurance-package)
- **Completion evidence:** [T-001–T-002 completion log](../work/completed-tasks.md)
- **Code evidence:** [T-002 `5b4c631` (tests/redteam.rs)](../work/completed-tasks.md)
- **Test evidence:** [sprint 8 test report](../sprints/s8/sprint-tests/test-report.md) — corpus green on both CI OSes (tested head `8ea2219`)
- **Documentation evidence:** [the threat model](../threat-model.md)

## Intent
Produce an explicit, maintained security-assurance package: a threat model that
maps Kinesin's design and boundaries onto OWASP LLM Top-10 (2025) and OWASP
Agentic Top-10 (2026) and the NIST AI-agent controls (just-in-time privilege,
policy-based authorization, task scoping, human-approval gates, dev/test/prod
separation); an explicit treatment of the **service transport-authentication /
identity boundary** — bearer-over-loopback today, with mTLS / mutual identity
named as the hardening required before any non-loopback (shared-service) exposure;
a CISA/NSA-aligned **memory-safety statement** that documents the `unsafe` FFI
surface (windows-sys, libc) and why the project meets the memory-safe roadmap
expectation; and a **standing red-team / prompt-injection corpus** wired into CI,
tied to security.md's release-evidence matrix. Non-goals: a formal
certification or audit sign-off; proving model behavior is safe (prompt-injection
evals measure the model, authorization tests prove the gate — both are recorded,
neither substitutes for the other).

## Acceptance criteria
- A `docs/threat-model.md` maps each OWASP LLM/Agentic risk and each NIST control
  to a specific Kinesin mechanism, gap, or accepted residual risk — including the
  service transport-authentication/identity boundary (bearer-over-loopback as the
  current control; mTLS/mutual-identity as the residual-risk mitigation gated
  before non-loopback exposure).
- A memory-safety statement enumerates every `unsafe` block / FFI dependency, its
  justification, and its containment, satisfying the CISA/NSA memory-safe-roadmap
  expectation.
- A red-team corpus (prompt-injection and authorization-bypass fixtures) runs in
  CI and maps to the release-evidence matrix; each authorization test proves a
  specific forbidden effect cannot pass the implemented gate.
- The package is versioned and has an owner/update cadence, not a one-off.

## Rationale
The project's security posture is strong but implicit and scattered across
security.md, decisions.md, and adversarial-review.md. A production, high-assurance
product needs a single, standards-mapped, testable assurance artifact — and the
memory-safety statement is now an explicit CISA/NSA expectation for software
manufacturers.

## Alternatives
Keep the posture implicit in the existing docs (current; not standards-mapped, not
CI-enforced). A third-party audit (valuable later; presupposes this internal
package exists first).

## Consequences
Ongoing maintenance as the design evolves; a growing red-team corpus and its CI
time; the discipline of keeping the OWASP/NIST mapping current; dependency on
INT-0017 (approval gates/JIT privilege) and INT-0012 (sandboxing) for some
controls to move from "gap" to "mechanism."

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme B — supply-chain & assurance).
- 2026-09-11: scope refined (still `proposed`) to explicitly own the service transport-authentication/identity boundary (bearer-over-loopback → mTLS for non-loopback exposure), closing the "no mTLS/identity beyond bearer" research-report gap flagged in ultrareview of the sprint 5 checkpoint.
- 2026-09-11: `proposed → planned`; selected for sprint 8 and linked to the build plan (T-001 docs/threat-model.md with the OWASP/NIST/CISA mapping + memory-safety statement + corpus→matrix map; T-002 tests/redteam.rs consolidating the corpus).
- 2026-09-11: `planned → active`; sprint 8 build began (T-001).
- 2026-09-11: `active → realized`; sprint 8 shipped `docs/threat-model.md` (versioned v1) — OWASP LLM/Agentic + NIST-control tables (mechanism/gap-with-owning-intent/residual, incl. the transport-auth/mTLS residual boundary), a CISA/NSA memory-safety statement enumerating the full unsafe/FFI surface with the no-unsafe-in-core posture, and a red-team-corpus → release-evidence-matrix map — plus `tests/redteam.rs` (ungranted-denied + content-as-data), green on both CI OSes. Recorded gaps remain owned by INT-0017 (approval gates/JIT), INT-0014 (tamper-evidence), INT-0019 (Windows sandbox); mTLS stays a documented residual boundary until non-loopback exposure is built. The package is versioned/owned, so it is superseded by a future revision rather than edited after realization.
