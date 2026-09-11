# Sprint 6 Test Report — Supply-chain dependency gate (INT-0013)

- **Tested head:** `62abee0e94eb819828f1986795a1faee0c0bca34`
- **Deliverable:** a policy file (`deny.toml`) + a blocking CI job; verification is
  a local tool run plus CI (the non-unit verification sprint 1's CI task used).
- **Result:** local checks pass; the CI `supply-chain` job is confirmed green on
  the checkpoint PR (see the loop-phase checkpoint).
- **Critic verdict:** `proceed-with-caveats` (see [critique.md](critique.md)).

## Intent acceptance → evidence
| INT-0013 acceptance criterion | Verification | Result |
|-------------------------------|--------------|--------|
| CI fails on a known-vuln dependency or a cargo-deny policy violation | `deny_denies_by_default` (deny-by-default licenses/sources, blocking steps) + `supply_chain_job_green_at_checkpoint` | ok (structural + CI) |
| `deny.toml` passes on the current tree (starts green); ignored advisories justified | `deny_check_passes_on_current_tree` (`cargo deny check` all-ok) + `audit_clean_or_documented` (`cargo audit` 0 vulns, `ignore = []`) | ok |
| Cargo.lock committed/reviewed; build.rs surface inventoried | Cargo.lock committed (221 deps); no first-party `build.rs` (only the `cmd-fixture` bin); dependency build scripts covered by advisory/source policy | ok |
| cargo-vet decision recorded; release-hardening deferral recorded | INT-0013 consequences + roadmap parking-lot (release-artifact integrity split out) | ok |

Details/commands: [integration-tests.md](integration-tests.md). Unit layer: n/a
([unit-tests.md](unit-tests.md)). E2E: the CI `supply-chain` job
([e2e-tests.md](e2e-tests.md)).

## Caveats carried forward (from the critic)
- **C-001 (accepted):** fail-on-violation is verified structurally (deny-by-default
  + blocking non-`continue-on-error` steps), not by injecting a real bad crate.
- **C-002 (accepted):** `cargo audit`/`cargo deny advisories` can turn CI red on an
  unchanged tree when a new advisory is published — by design; the escape hatch is a
  justified `ignore` entry.
- **C-003 (deferred):** `taiki-e/install-action@v2` is a major-tag pin consistent
  with `actions/checkout@v7`; SHA-pinning every action is a future workflow-wide hardening.

## Verdict
Test phase satisfied for INT-0013 (re-scoped to the CI dependency gate): the gate
is authored green (`cargo deny check` all-ok, `cargo audit` 0 vulns), the policy is
deny-by-default, and the blocking `supply-chain` CI job is in place and confirmed
green at the checkpoint. Release-artifact integrity is tracked in the roadmap
parking-lot. Proceed to loop.
