# INT-0028 — Runnable Windows and Linux first-use instructions

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0028
- **State:** realized
- **Work evidence:** [sprint 11 build plan](../sprints/s11/sprint-plans/build-plan.md)
- **Completion evidence:** [T-001 entry/documentation](../work/completed-tasks.md#t-001-sprint-11), [T-002 platform verification](../work/completed-tasks.md#t-002-sprint-11)
- **Code evidence:** [default product](../../Cargo.toml), [CLI help](../../src/cli.rs), [entry regression](../../tests/cli_inspect.rs), [starter configuration](../../kinesin.example.toml)
- **Test evidence:** [sprint 11 test report](../sprints/s11/sprint-tests/test-report.md)
- **Documentation evidence:** [README entry](../../README.md#start-here), [Windows/Linux usage guide](../getting-started.md), [CLI reference](../cli.md), [configuration reference](../configuration.md)

## Intent
Enable an operator with a checkout to install or launch the actual Kinesin CLI,
prepare valid configuration and workspace/state directories, connect a separately
started model, and interpret a first result on Windows and Linux. Explain the
two fixture binaries as integration-test support. Non-goals: a GUI, automatic
model provisioning, new inference backends or unrelated runtime refactoring.

## Acceptance criteria
- Plain `cargo run` selects Kinesin; users can inspect `--help` without a model,
  configuration or state effects. Only the product binary is selected for install.
- README prominently links and shows usable Windows PowerShell and Linux launch
  commands, distinguishing checkout, built executable, PATH installation and the
  current-directory configuration rule.
- The detailed guide gives prerequisites, private state setup, a copyable starter
  configuration, model startup/connection, session and one-shot/checked examples,
  expected outcomes and fixes for the reported errors. Paths resolve as written.
- Windows and Linux smoke checks verify the documented entry points; Nighthawk
  evidence distinguishes actual commands from steps not executed. CLI/configuration
  references agree with current syntax and capture/replay behavior.

## Rationale
The operator could neither find `kinesin` on PATH nor select a binary with
`cargo run`. Existing documentation assumes installation and earlier guide steps.

## Alternatives
Document `--bin` everywhere without fixing the default (needlessly retains the
reported trap); remove fixtures (loses actual process/stdio integration tests);
expand to a package manager or installer (outside this quick documentation sprint).

## Consequences
The operator still supplies a compatible model server. Build tooling and model
files are separate from Kinesin. First-use examples grant only read/list tools;
broader capabilities require explicit configuration.

## Transition history
- 2026-09-12: created as `proposed` for the user's Windows/Linux usage sprint.
- 2026-09-12: `proposed → planned`; the requested usage sprint selects the bounded entry/documentation and platform-verification tasks.
- 2026-09-12: `planned → active`; the independently reviewed plan is locked and the entry/documentation work begins.
- 2026-09-12: `active → realized`; completed entry and platform tasks, successful Windows/Ubuntu CI, native Windows/Debian walkthroughs, PowerShell 5.1 setup/help, preserved repeat setup, disconnected replay and clean independent TEST review satisfy this bounded usage intent. Fresh machine provisioning and release installation remain explicitly unexecuted rather than implicit acceptance claims.
