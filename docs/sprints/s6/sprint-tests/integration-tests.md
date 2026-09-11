# Sprint 6 Integration Tests (supply-chain gate)

- **Intent:** [INT-0013](../../../intents/INT-0013-supply-chain-security.md)
- **Tested head:** `62abee0e94eb819828f1986795a1faee0c0bca34`
- **Runners:** `cargo deny check`, `cargo audit` (cargo-deny 0.x, cargo-audit installed locally); YAML inspection of `.github/workflows/ci.yml`.
- **Result:** all checks pass.

| Check (EARS clause) | Command / inspection | Result |
|---------------------|----------------------|--------|
| `deny_check_passes_on_current_tree` (T-001) | `cargo deny check` | **ok** — advisories ok, bans ok, licenses ok, sources ok (duplicate-crate lines are warnings, `multiple-versions = "warn"`, non-failing) |
| `audit_clean_or_documented` (T-001) | `cargo audit` | **ok** — 221 crate dependencies scanned, 0 vulnerabilities; `ignore = []` (no advisory suppressed) |
| `deny_denies_by_default` (T-002 fail-on-violation) | read `deny.toml` | **ok** — `licenses` is an explicit allow-list (unlisted → reject), `sources` sets `unknown-registry`/`unknown-git = "deny"` and allows only crates.io, `bans` denies `wildcards`; any advisory failure exits non-zero |
| `ci_has_blocking_supply_chain_job` (T-002) | read `.github/workflows/ci.yml` | **ok** — a `supply-chain` job runs `cargo deny check` and `cargo audit` as plain (blocking, non-`continue-on-error`) steps on the pinned toolchain |

## Notes
- The license allow-list was tightened to exactly the licenses present in the
  221-crate tree (via `cargo deny list`); `CDLA-Permissive-2.0` (webpki-root-certs)
  is the only non-obvious entry. Multi-licensed copyleft/other crates
  (LGPL/Unlicense/BSL/MIT-0) resolve through their MIT/Apache option and are not
  allowed by identifier.
- Kinesin's own crate is `publish = false` with `[licenses] private.ignore = true`,
  so its absent license field does not fail the gate.

## Confirmation
```
advisories ok, bans ok, licenses ok, sources ok        # cargo deny check
Scanning Cargo.lock for vulnerabilities (221 crate dependencies)   # cargo audit → 0 vulns
ci.yml jobs: ['check', 'supply-chain']
```
