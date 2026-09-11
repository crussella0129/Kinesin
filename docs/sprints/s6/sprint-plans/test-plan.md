# Sprint 6 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0013](../../../intents/INT-0013-supply-chain-security.md) | gate passes on the current tree (starts green) | T-001 / WHEN cargo deny check runs THEN no violation | `deny_check_passes_on_current_tree` |
| [INT-0013](../../../intents/INT-0013-supply-chain-security.md) | no known-vulnerable dependency (or documented) | T-001 / ignored advisory carries a justification | `audit_clean_or_documented` |
| [INT-0013](../../../intents/INT-0013-supply-chain-security.md) | CI runs deny+audit as blocking steps | T-002 / WHEN CI runs THEN supply-chain job runs deny+audit | `ci_has_blocking_supply_chain_job`, `supply_chain_job_green_at_checkpoint` |
| [INT-0013](../../../intents/INT-0013-supply-chain-security.md) | CI fails on advisory/policy violation | T-002 / WHEN a dep violates policy THEN the job fails | `deny_denies_by_default` (policy is deny-by-default; blocking step propagates non-zero) |

## Unit Tests
Not applicable — the deliverable is a policy file and a CI job, not Rust code.
Verification is a local tool run plus CI (the same non-unit verification sprint 1's
CI task used).

## Integration Tests
### Supply-chain gate (local + CI content)
- **Intents:** [INT-0013](../../../intents/INT-0013-supply-chain-security.md)
- `deny_check_passes_on_current_tree`: `cargo deny check` exits 0 on the current tree with the committed `deny.toml`.
- `audit_clean_or_documented`: `cargo audit --locked` exits 0 (no known vuln), or any advisory is ignored in `deny.toml` with a justification.
- `ci_has_blocking_supply_chain_job`: `.github/workflows/ci.yml` contains a `supply-chain` job that runs `cargo deny check` and `cargo audit --locked` (not `|| true`, not `continue-on-error`).
- `deny_denies_by_default`: the `licenses`/`sources` policy is deny-by-default (an unlisted license or a non-crates.io source would fail), verified by reading `deny.toml`.

## End-to-End Tests
- **Status:** possible (CI). `supply_chain_job_green_at_checkpoint`: the new
  `supply-chain` job runs on the checkpoint PR and is green; the existing
  windows+ubuntu fmt/clippy/test jobs remain green. This is the authoritative
  proof that the gate executes and blocks — the same non-unit CI verification the
  sprint 1 matrix used.
