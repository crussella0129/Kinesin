# Sprint 6 Build Plan

## Intents
- [INT-0013](../../../intents/INT-0013-supply-chain-security.md) — state: planned; acceptance criteria covered: CI fails on a known-vulnerable dependency or a cargo-deny policy violation; reviewed committed Cargo.lock + build.rs inventory; cargo-vet decision and release-hardening deferral recorded.

## Schema Tree
- Sprint Goal: an enforced CI supply-chain gate
  - Policy
    - T-001: author deny.toml from the current tree (green)
  - Enforcement
    - T-002: add the blocking CI supply-chain job

## Execution Sequence

### T-001: Author `deny.toml` supply-chain policy
- **Intent:** [INT-0013](../../../intents/INT-0013-supply-chain-security.md)
- **Touches:** deny.toml
- **Depends on:** (none)
- **Acceptance criterion:** the policy (advisories/licenses/bans/sources) passes on the current tree, so the gate starts green; any ignored advisory is justified.
- **Success criterion (EARS):**
  - **WHEN** `cargo deny check` runs against the current tree with `deny.toml`, **THEN** it **SHALL** report no advisory, license, ban, or source violation.
  - **WHEN** an advisory is deliberately ignored, **THEN** its `deny.toml` entry **SHALL** carry a one-line justification.
- **Notes:** install `cargo-deny` locally and iterate to green; allow-list only the licenses actually present (deny by default); `sources` allows crates.io only; `bans` warns on duplicates and denies wildcard versions. Record that the repo has no first-party `build.rs` (dependency build scripts are covered by the advisory/source policy) and note the cargo-vet decision + release-hardening deferral in the intent's consequences.

### T-002: Add the CI supply-chain gate
- **Intent:** [INT-0013](../../../intents/INT-0013-supply-chain-security.md)
- **Touches:** .github/workflows/ci.yml
- **Depends on:** T-001
- **Acceptance criterion:** CI runs cargo-deny and cargo-audit as blocking steps and fails on a known-vulnerable dependency or a policy violation.
- **Success criterion (EARS):**
  - **WHEN** CI runs, **THEN** a `supply-chain` job **SHALL** execute `cargo deny check` and `cargo audit --locked` on the pinned 1.96.0 toolchain as blocking steps.
  - **WHEN** a dependency carries a known advisory or violates the `deny.toml` policy, **THEN** that job **SHALL** fail (non-zero, blocking the check).
- **Notes:** dedicated `ubuntu-latest` job, `permissions: contents: read`, `actions/checkout@v7` with `persist-credentials: false`; install the tools via a pinned `taiki-e/install-action` (fallback `cargo install --locked`); keep the existing windows+ubuntu fmt/clippy/test jobs unchanged.
