# Sprint 11 Research Report

## Intents Reviewed
- [INT-0028](../../../intents/INT-0028-first-use-documentation.md) — created for
  executable Windows/Linux onboarding and the reported binary/PATH errors.
- [INT-0021](../../../intents/INT-0021-harness-contract-review.md) — historical
  audit context only; terminal intent is not rewritten.

## 1. Sprint Goal
Repair the missing usage path with explicit Windows and Linux instructions and
small CLI entry-point corrections. The user explicitly directed these changes
to existing PR #11, which remains open; do not merge it or create another PR.

## 2. Existing Code Survey
| File | Relevance | Finding |
| --- | --- | --- |
| Cargo.toml | high | Three binaries, no default-run; product install must select --bin kinesin |
| README.md | high | No usable installation/first-run sequence |
| src/main.rs | high | CLI parse/execute entry |
| src/cli.rs | high | No run subcommand or successful --help path; no args reads cwd kinesin.toml |
| src/config.rs | high | Paths are config-relative; workspace must exist |
| src/storage.rs | medium | State parent creation does not constitute private ACL provisioning |
| docs/cli.md | high | Assumes PATH and guide milestones; replay example lacks replay capture |
| docs/configuration.md | high | Obsolete future-guide prose and placeholder model identity |
| docs/model-preflight.md | high | Pinned model/runtime evidence uses machine-specific paths |
| docs/integration.md | medium | Existing separate-server/SSH architecture |
| examples/first-turn.toml | high | Correct only with sibling state/workspace and compatible external server |
| examples/file-task.toml | high | Ready checked task, metadata capture default |
| tests/cli_inspect.rs | medium | Process-level CLI checks can verify side-effect-free help |
| rust-toolchain.toml | medium | Rust 1.96.0 pin |

`cargo run -- --help` reproduced Cargo's exact multi-binary error before edits.
Nighthawk is reachable as charles; Git exists, cargo/rustc are absent from PATH,
and its filesystem reports 415 GiB free. Model/runtime staging from sprint 10
remains separate from the software checkout.

## 3. External Sources
- [Cargo manifest](https://doc.rust-lang.org/cargo/reference/manifest.html#the-default-run-field) — default-run chooses the package entry.
- [Cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) — path installation, binary selection and Cargo bin directory.
- [Rust installation](https://doc.rust-lang.org/book/ch01-01-installation.html) — toolchain/linker prerequisites and shell setup.
- [rustup installation](https://rust-lang.github.io/rustup/installation/index.html) — installation environment and PATH.
- [AWS-LC Linux requirements](https://aws.github.io/aws-lc-rs/requirements/linux.html) — native compilation prerequisites.
- [AWS-LC Windows requirements](https://aws.github.io/aws-lc-rs/requirements/windows.html) — MSVC and NASM/prebuilt alternative.
- [AWS-LC crate documentation](https://docs.rs/aws-lc-rs/latest/aws_lc_rs/) — distinguish the current non-FIPS dependency requirements.

## 4. Risks, Unknowns, Dependencies
Existing model preflight is provenance, not an installer. Installation does not
create configuration, add a global command in every shell, or start a model.
PowerShell 5.1 syntax/encoding and Linux permissions must be explicit. Linux
toolchain setup on nighthawk must be scoped and recorded; never describe an
unexecuted shell snippet as tested. Existing files/configuration must survive
repeat setup. Prior WSL/disk issues make that host unsuitable for redundant full
builds; use bounded native-host builds and existing CI instead.

## 5. Recommended Approach
Add default-run and a read-only help entry. Supply a root-relative starter TOML
and a detailed getting-started chapter with Windows/Linux sections, private state
setup, external pinned server and Nighthawk connection examples, shell-specific
launch/install syntax, expected JSON and troubleshooting. Lead README with a
short launch path and the guide. Correct current CLI/configuration prose rather
than rewriting architecture docs. Keep fixture binaries for native process and
stdio tests; install only kinesin. Verify real first use and update PR #11.

## Budget Override
Seven primary reference pages were checked across Cargo installation, rustup
and two platform-specific native dependency requirements. This small exception
supports the user's explicit Windows/Linux instructions without guessing build
prerequisites; the code survey remains at 14 files and research stays bounded.
