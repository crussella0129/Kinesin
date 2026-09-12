# INT-0028 — Runnable Windows and Linux first-use instructions

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0028
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

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
