# Test Critique — Sprint 6

Adversarial read-only screen of the sprint 6 supply-chain gate verification
against INT-0013 (re-scoped to the CI dependency gate).

## Concerns

### C-001: fail-on-violation is verified structurally, not by fault injection
- **Where:** `e2e-tests.md` / `integration-tests.md` `deny_denies_by_default`.
- **Failure mode:** weak-assertion / negative-path.
- **Why it matters:** the evidence shows the gate is deny-by-default and runs as a blocking step, not a red run from a real bad crate.
- **Suggested response:** accept-with-rationale. Injecting a real advisory/banned/non-crates.io crate would pollute `Cargo.lock` and the tree; the blocking behaviour follows deterministically from `cargo deny check`/`cargo audit` exiting non-zero on a finding (no `continue-on-error`/`|| true`) plus an explicit allow-list `licenses` and `deny` `sources`. Verifying a policy gate by reading the deny-by-default config + the non-suppressed step is the standard practice.

### C-002: an advisory gate can turn CI red without a code change
- **Where:** the `supply-chain` job (`cargo audit`, `cargo deny advisories`).
- **Failure mode:** flake-risk (by-design, not a flake).
- **Why it matters:** `cargo audit` fetches the RustSec DB at run time; a newly-published advisory against a currently-pinned dependency will fail CI on an unchanged tree.
- **Suggested response:** accept-with-rationale. This is the gate working as intended — a new advisory *should* surface — not nondeterminism. The escape hatch is a justified `ignore` entry in `deny.toml` (and `cargo audit` respects the same). Documented so a future red audit is understood as a real signal, not a broken test.

### C-003: the install action is pinned to a major tag, not a SHA
- **Where:** `.github/workflows/ci.yml` (`taiki-e/install-action@v2`).
- **Failure mode:** hidden-dep (screened).
- **Why it matters:** on a supply-chain job, a moving major-tag pin is itself trust surface.
- **Suggested response:** defer-with-rationale. `@v2` matches the repo's existing pin granularity (`actions/checkout@v7`); tightening every action to a SHA is a consistent future hardening across the workflow, not a sprint-6-only gap. The fallback `cargo install --locked` is recorded if a stricter pin is wanted.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — each INT-0013 acceptance criterion maps to a named executed check (`deny_check_passes_on_current_tree`, `audit_clean_or_documented`, `deny_denies_by_default`, `ci_has_blocking_supply_chain_job`, `supply_chain_job_green_at_checkpoint`).
- **Stub leakage:** none — real tools run against the real tree.
- **Integration drift:** none — the checks exercise the actual gate, not a proxy.
- **E2E cop-out:** none — `possible`, with the named CI job as the live proof.
- **Evidence drift:** none — artifacts name tested head `62abee0`, carry the exact `cargo deny`/`cargo audit` confirmations, and identify the Test-evidence link attached to INT-0013.

## Confidence
proceed-with-caveats
