# INT-0013 — Supply-chain security & signed releases

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0013
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Establish a supply-chain integrity gate for the dependency tree and the release
artifacts. Add `cargo-deny` (advisories, licenses, banned/duplicate crates,
source policy) and `cargo-audit` as blocking CI steps; build `cargo-auditable`
binaries so the dependency set is recoverable from a shipped binary; keep a
reviewed, committed `Cargo.lock`; generate an SBOM; sign release artifacts
(cosign/sigstore) and pursue reproducible builds. Evaluate `cargo-vet` for a
source-audit posture appropriate to a security-critical project. Non-goals:
rewriting dependencies; treating an SBOM as prevention (it is inventory, not a
control); vendoring the entire tree.

## Acceptance criteria
- CI fails on a known-vulnerable dependency or a `cargo-deny` policy violation
  (advisory, disallowed license, banned or duplicate crate, untrusted source).
- Released binaries are `cargo-auditable` (their dependency set is recoverable)
  and are signed; the signature and an SBOM are published with each release.
- `Cargo.lock` is committed and reviewed; `build.rs` scripts in the tree are
  inventoried and justified (mitigating the 2026 TrapDoor-style build-script risk).
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
New CI steps and their maintenance (advisory noise, license exceptions); a release
pipeline with signing keys to manage; SBOM generation to keep current; a policy
file (`deny.toml`) that must track intentional exceptions.

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme B — supply-chain & assurance); recommended as an early, standalone sprint.
