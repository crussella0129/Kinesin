# Sprint 11 Unit and Entry Tests

Intent: [INT-0028](../../../intents/INT-0028-first-use-documentation.md).
Implementation: `c7eb088`; ledger-only tested head `ee6a26b8c2b662b77511c1f25bfa01a57ea62cd5`.

| Named verification | EARS | Executed assertion and result |
| --- | --- | --- |
| help_without_configuration | T-001/A | New real-process `help_needs_no_configuration_or_state_and_rejects_extra_arguments`: --help and -h exit 0, usage names session and cwd configuration, stderr empty and empty working directory unchanged. Three malformed combinations exit 1 with stderr and no files. Pass. |
| default_product_entry | T-001/A | `cargo run --locked -- --help` selects kinesin and exits 0 on Windows and native Debian. Pass. |
| fixture_explanation_review | T-001/C | Cargo targets and CARGO_BIN_EXE fixture consumers agree with README/guide: command child and MCP stdio test server are separate processes; normal install selects --bin kinesin. Pass. |
| usage_reference_review | T-002/B | Independent review of README, guide, CLI reference, configuration reference and starter found two documentation issues (health port and temporary installer); both corrected and rechecked clean. 65 documentation links/anchors resolved. |

Local Windows confirmations, after the complete source change:

- `cargo test --locked --lib cli::tests`: 8 passed, 0 failed.
- `cargo test --locked --test cli_inspect`: 11 passed, 0 failed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.

These focused runs are not described as the full suite. Canonical Windows/Ubuntu
CI for the implementation head is recorded in the final test report once complete.
