# INT-0013 — Supply-chain dependency gate

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0013
- **State:** planned
- **Work evidence:** [T-001 build plan](../sprints/s6/sprint-plans/build-plan.md#t-001-author-denytoml-supply-chain-policy)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Establish an enforced supply-chain integrity gate over the **dependency tree**:
`cargo-deny` (advisories, licenses, banned/duplicate crates, source policy) and
`cargo-audit` as blocking CI steps, driven by a committed `deny.toml`, with a
reviewed, committed `Cargo.lock`, and a documented `cargo-vet` decision. Scope is
the dependency gate; **release-artifact integrity** (SBOM, `cargo-auditable`
binaries, cosign/sigstore signing, reproducible builds) is a distinct outcome
that needs a release pipeline the project does not have yet, and is tracked as a
roadmap parking-lot item rather than in this intent. Non-goals: rewriting
dependencies; treating an SBOM as prevention (it is inventory, not a control);
vendoring the entire tree.

## Acceptance criteria
- CI fails on a known-vulnerable dependency or a `cargo-deny` policy violation
  (advisory, disallowed license, banned or duplicate crate, untrusted source).
- `deny.toml` passes on the current tree (the gate starts green); any ignored
  advisory carries a justification.
- `Cargo.lock` is committed and reviewed; the repository's `build.rs` surface is
  inventoried (first-party build scripts noted; dependency build scripts covered
  by the advisory/source policy — mitigating the 2026 TrapDoor-style risk).
- A documented decision records whether `cargo-vet` is adopted and, if so, the
  audit policy; if deferred, the rationale and trigger.

## Rationale
The manifest already avoids OpenSSL (rustls), pins the toolchain, and uses
`subtle`, but there is no supply-chain gate at all — a production, high-security
posture requires one, and the tooling (cargo-deny/audit/auditable/vet, SLSA,
cosign) is mature and mostly low-friction. This is a cheap, high-assurance early
sprint.

## Alternatives
Manual periodic `cargo audit` (current: none; easy to forget, not enforced).
Full `cargo-vet` from day one (high friction — median ~8.7k changed lines/week to
stay fully vetted; reserve for the security-critical tier). SBOM-only (rejected as
a control: a malicious crate can omit itself from an SBOM).

## Consequences
New CI steps and their maintenance (advisory noise, license exceptions); a policy
file (`deny.toml`) that must track intentional exceptions with justifications;
an added (pinned) CI action or tool install. Release-artifact integrity, split
out to the roadmap parking-lot, will bring its own signing-key and SBOM
maintenance when a release pipeline is built.

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme B — supply-chain & assurance); recommended as an early, standalone sprint.
- 2026-09-11: `proposed → planned`; selected for sprint 6 and linked to the build plan (T-001 deny.toml policy, T-002 CI supply-chain gate). Re-scoped to the CI **dependency gate** (a distinct, deliverable-now outcome): release-artifact integrity (SBOM, cargo-auditable, cosign signing, reproducible builds) was split out to the roadmap parking-lot because it needs a release pipeline that does not exist yet; cargo-vet remains a documented decision (cargo-deny is the baseline). Title changed from "Supply-chain security & signed releases" to "Supply-chain dependency gate" to match.
