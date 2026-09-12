# Sprint 11 Build Plan

## Intents
- [INT-0028](../../../intents/INT-0028-first-use-documentation.md): runnable
  Windows/Linux onboarding, product selection, configuration and evidence.

## Authorization and source freeze
The user requested this small usage sprint and Linux coverage, and explicitly
directed its changes into existing PR #11. No merge is authorized. This host
provides neither EnterPlanMode nor ExitPlanMode; preserve the source freeze and
independent plan review. No source changes occur before plan finalization.

## Schema Tree
- First-use contract
  - T-001: entry point and documentation
  - T-002: real command verification and closure evidence

## Execution Sequence

### T-001: Make the documented entry paths usable
- **Intent:** [INT-0028](../../../intents/INT-0028-first-use-documentation.md)
- **Touches:** Cargo.toml; src/cli.rs; tests/cli_inspect.rs; kinesin.example.toml;
  README.md; docs/getting-started.md; docs/cli.md; docs/configuration.md; docs/SUMMARY.md
- **Depends on:** none
- **Acceptance criterion:** Product selection/help, OS-specific usage, starter configuration and current command reference.
- **Success criterion (EARS):**
  - A: **WHEN** an operator uses cargo run or installs with --bin kinesin, **THEN** the product SHALL be selected; --help/-h SHALL print usage and exit zero without configuration, model access or state creation.
  - B: **WHEN** an operator follows the Windows or Linux first-use path, **THEN** the docs SHALL give executable build/install/PATH, private-state/config/workspace and model-server steps, correct launch/session/checked/replay syntax, expected outcomes and targeted troubleshooting; repeated setup SHALL preserve existing configuration, workspace content and directory permissions.
  - C: **WHEN** fixture binaries are listed, **THEN** the docs SHALL explain their process/stdio-test purpose and show that routine product installation selects only kinesin.

### T-002: Verify both platforms and finish the documentation checkpoint
- **Intent:** [INT-0028](../../../intents/INT-0028-first-use-documentation.md)
- **Touches:** docs/sprints/s11/sprint-tests; docs/intents/INT-0028-first-use-documentation.md; docs/work; sprint metadata; affected usage docs for corrections
- **Depends on:** T-001
- **Acceptance criterion:** Actual OS checks, truthful deployment evidence and coherent references.
- **Success criterion (EARS):**
  - A: **WHEN** usage verification runs, **THEN** Windows and Linux SHALL prove help/default selection and product-only installation; actual Nighthawk commands and first-use outcomes SHALL be recorded separately from unexecuted prerequisite steps.
  - B: **WHEN** the sprint closes, **THEN** formatting, clippy, affected CLI tests, independent TEST critique and Book/link checks SHALL pass; the closed sprint SHALL join existing PR #11 with remaining platform limitations stated accurately.

## Scope and ownership
Root owns documentation, Book and commits; one agent may own the CLI help/default
entry change, and another may verify the Linux environment without editing source.
Use an isolated install prefix to test cargo install without replacing an existing
binary. Nighthawk may use scoped user-local Rust tooling if its existing compiler
prerequisites suffice; no root package installation or host-wide changes. If a
prerequisite cannot be supplied within this boundary, record exactly what remains
unexecuted and use supported Linux CI for those CLI checks. No new model download
is needed when the retained pinned model/runtime is available.
