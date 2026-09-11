# Sprint 6 Research Report — Supply-chain security gate (INT-0013)

## Intents Reviewed
- [INT-0013](../../../intents/INT-0013-supply-chain-security.md) — selected; relevance: the roadmap's top-priority item (theme B, recommended first build sprint); state: `proposed`. SotA basis carried from the [sprint 5 research report](../../s5/sprint-research/research-report.md).

## 1. Sprint Goal
Add an enforced supply-chain integrity gate to CI so a known-vulnerable
dependency or a policy violation (advisory, disallowed license, banned/duplicate
crate, untrusted source) fails the build. Scope this sprint to the **dependency
gate** — the achievable, high-value core — and defer release-artifact hardening
(SBOM, `cargo-auditable`, cosign signing, reproducible builds) since the project
has no release pipeline yet; record that deferral explicitly.

## 2. Existing Code Survey
| File | Relevance | State / gap |
|------|-----------|-------------|
| .github/workflows/ci.yml | high | fmt/clippy/`test --locked` on windows+ubuntu, pinned 1.96.0, `permissions: contents: read`, `persist-credentials: false`, `actions/checkout@v7`. **No supply-chain step.** Add a gate here. |
| Cargo.toml / Cargo.lock | high | `Cargo.lock` committed (221 packages); rustls (no OpenSSL), `subtle`, bundled sqlite. No `deny.toml`. Deps are the policy surface. |
| rust-toolchain.toml | medium | pins 1.96.0 — the gate must use the same toolchain. |
| (repo build scripts) | medium | no first-party `build.rs` in the tree (only the `cmd-fixture` bin); the `build.rs` exfiltration risk (2026 TrapDoor) is a *dependency* concern, covered by advisory + source policy. |
| docs/intents/INT-0013-*, docs/roadmap.md | high | the intent + its roadmap placement (theme B, sequence #1). |

## 3. External Sources
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/) — policy engine for advisories, licenses, bans (duplicates/specific crates), and sources; driven by a committed `deny.toml`. The baseline gate.
- [cargo-audit + RustSec advisory DB](https://github.com/rustsec/rustsec) — CVE/advisory scan against `Cargo.lock`.
- [taiki-e/install-action](https://github.com/taiki-e/install-action) — installs prebuilt `cargo-deny`/`cargo-audit` binaries in CI in seconds (avoids a multi-minute `cargo install` compile), pinnable to a version — itself a supply-chain choice to pin deliberately.
- (SotA framing — cargo-deny vs. cargo-vet friction, SBOM limits, SLSA/cosign — carried from the [sprint 5 survey](../../s5/sprint-research/research-report.md#3-external-sources).)

## 4. Risks, Unknowns, Dependencies
- **Risk (advisory noise / license flags):** a fresh `deny.toml` may flag existing transitive advisories or licenses, turning CI red on unrelated crates. Mitigation: author `deny.toml` from the *current* tree, allow-list the licenses actually present with rationale, and record any accepted/ignored advisory with a justification + expiry, so the gate starts green and stays honest.
- **Risk (adding third-party CI actions is itself supply-chain surface):** installing the tools via an action adds trust. Mitigation: pin the install action to a version (matching the repo's pinned-toolchain discipline), or fall back to `cargo install --locked` if pinning an action is judged too much trust.
- **Unknown (CI cost/placement):** run the gate as a dedicated lightweight `ubuntu-latest` job (the advisory/license/ban/source checks are graph-level, not per-OS) rather than expanding the 2-OS matrix.
- **Dependency:** none blocking; this is a standalone CI + policy addition. It does **not** gate on a release pipeline (that is the deferred half).
- **Deferred (recorded, not this sprint):** `cargo-auditable` binaries, SBOM, cosign/sigstore signing, reproducible builds — all require a release workflow that does not exist yet; `cargo-vet` adoption remains a documented decision (high friction; cargo-deny is the baseline).

## 5. Recommended Approach
**Primary — a committed `deny.toml` + a dedicated CI supply-chain job.**
1. Author `deny.toml` from the current tree: `advisories` (deny vulnerabilities/unmaintained, with any justified `ignore` + rationale), `licenses` (allow-list the licenses actually present, deny-by-default otherwise), `bans` (warn/deny duplicates, no wildcard versions), `sources` (allow only crates.io + any explicitly trusted git source — none today).
2. Add a `supply-chain` job to `ci.yml` on `ubuntu-latest`, pinned toolchain, that installs `cargo-deny` + `cargo-audit` (via a pinned install action) and runs `cargo deny check` and `cargo audit --locked` as blocking steps; keep `permissions: contents: read`.
3. Keep `Cargo.lock` committed and reviewed; document that first-party `build.rs` is absent and dependency build scripts are covered by the advisory/source policy.
4. Document the release-hardening deferral (SBOM/auditable/signing/reproducible) and the `cargo-vet` decision in the intent's consequences / a short note.

**Alternative considered:** `cargo install --locked cargo-deny cargo-audit` in CI (no extra action trust, but a multi-minute compile every run) — reserve as the fallback if pinning an install action is rejected. Full `cargo-vet` now — rejected (high per-update audit friction; cargo-deny is the right baseline).

## Artifacts
- No code artifacts pre-authored; `deny.toml` and the CI job are the build-phase deliverables.
