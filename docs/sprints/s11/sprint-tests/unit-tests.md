# Sprint 11 Unit and Entry Tests

Intent: [INT-0028](../../../intents/INT-0028-first-use-documentation.md).
Implementation: `c7eb088`; ledger-only tested head `ee6a26b8c2b662b77511c1f25bfa01a57ea62cd5`.

| Named verification | EARS | Executed assertion and result |
| --- | --- | --- |
| help_without_configuration | T-001/A | New real-process `help_needs_no_configuration_or_state_and_rejects_extra_arguments`: --help and -h exit 0, usage names session and cwd configuration, stderr empty and empty working directory unchanged. Three malformed combinations exit 1 with stderr and no files. Pass. |
| default_product_entry | T-001/A | `cargo run --locked -- --help` selects kinesin and exits 0 on Windows and native Debian. Pass. |
| fixture_explanation_review | T-001/C | Cargo targets and CARGO_BIN_EXE fixture consumers agree with README/guide: command child and MCP stdio test server are separate processes; normal install selects --bin kinesin. Pass. |
| usage_reference_review | T-002/B | Independent review of README, guide, CLI reference, configuration reference and starter found two documentation issues (health port and temporary installer); both corrected and rechecked clean. 65 documentation links/anchors resolved. |
| book_and_test_critic | T-002/B | Installed check-book validates 28 intent chapters. Independent navigation scan checked README plus 188 docs chapters: 790 relative links (786 file targets, 4 fragment-only) and 37 heading anchors, all resolving. The final TEST critique is recorded before the report; Loop separately owns realization, closure and the existing PR checkpoint. |

Local Windows confirmations, after the complete source change:

- `cargo test --locked --lib cli::tests`: 8 passed, 0 failed.
- `cargo test --locked --test cli_inspect`: 11 passed, 0 failed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.

These focused runs are separate from canonical CI. The implementation head
`ee6a26b8c2b662b77511c1f25bfa01a57ea62cd5` passed
[PR CI 34710282235](https://github.com/crussella0129/Kinesin/actions/runs/34710282235)
with all three jobs successful: Windows, Ubuntu and supply-chain. Both platforms
passed format, all-target/all-feature locked clippy with warnings denied and
`cargo test --locked`; cargo-deny and cargo-audit passed. The retained log is
`target/s11-ci.log`. Later sprint evidence commits change no runtime/test inputs.

| Canonical suite | Windows passed | Ubuntu passed |
| --- | ---: | ---: |
| Library | 164 | 168 |
| adversarial_runtime | 3 | 3 |
| cli_inspect | 11 | 11 |
| command_tool | 12 | 12 |
| live_comparisons (offline) | 1 | 1 |
| live_evaluation (offline) | 2 | 2 |
| mcp | 20 | 20 |
| model_protocol | 17 | 17 |
| process_recovery | 3 | 3 |
| redteam | 2 | 2 |
| replay | 16 | 16 |
| runner_journal | 9 | 9 |
| runner_tools | 24 | 24 |
| sandbox_linux | 0 | 9 |
| service | 16 | 16 |
| settlement | 3 | 3 |
| windows_signal | 1 | 0 |
| Total | 304 | 316 |

Zero failures; nine ignored on each platform. The isolated MCP environment worker
is exercised by its passing parent. Live model walkthroughs are separate evidence,
not included in these counts. Final submitted-head checkpoint checks belong to Loop.

An additional real Windows PowerShell 5.1.26100.9444 check passed the exact
setup block, sentinel/ACL preservation and installed help outside the checkout.
See the platform record and retained `target/s11-ps51/results.json`.
