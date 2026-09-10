# Sprint 1 Build Plan

## Intents
- [INT-0003](../../../intents/INT-0003-shell-execution.md) — state: planned; acceptance criteria covered: allow-listed command runs with bounded output + timeout + journalled effect; always an argv vector (no shell); denied without the grant and barred in checked runs; each failure mode (timeout, oversized output, non-zero exit, missing executable, cancellation) has a defined outcome.

## Schema Tree
- Sprint Goal: a bounded `run_command` tool — the last member of the write surface
  - Grant & policy
    - T-001: `ToolName::RunCommand` + per-workspace `commands` allow-list + validation + checked-run bar
  - Tool arguments
    - T-002: argv `command` field, shape check, and pure allow-list validation
  - Execution capability
    - T-003: `CommandRunner` — argv-only spawn, scrubbed env, cwd, bounded output, timeout, tree-kill
  - Runtime integration
    - T-004: runner dispatch route, `RunResources` wiring, effect journalling
  - Infrastructure
    - T-005: cross-platform CI matrix (Windows + Linux)

## Execution Sequence

### T-001: Config surface, allow-list, and checked-run bar for run_command
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Touches:** src/config.rs, src/policy.rs
- **Depends on:** (none)
- **Acceptance criterion:** the command tool is granted per workspace by operator
  config and barred from checked runs.
- **Success criterion (EARS):**
  - **WHEN** a workspace grants `run_command` with a non-empty `commands` allow-list, **THEN** config validation **SHALL** accept it.
  - **WHEN** a workspace grants `run_command` with an empty or absent `commands` list, or lists `commands` without granting `run_command`, **THEN** validation **SHALL** reject the config.
  - **WHEN** a `commands` entry contains a path separator or is not a bare executable name, **THEN** validation **SHALL** reject it.
  - **WHEN** a checked task's workspace grants `run_command`, **THEN** authorization **SHALL** refuse the configuration.
- **Notes:** `ToolName::RunCommand` sets `is_mutating() == true` (so the existing
  checked-run bar at `policy.rs:355` applies), `mints_evidence() == false`,
  `as_str() == "run_command"`. Reuse the existing id/name validation for allow-list entries.

### T-002: argv command argument, shape check, and allow-list validation
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Touches:** src/tools.rs
- **Depends on:** T-001
- **Acceptance criterion:** a command is always an argv vector; no string is passed to a shell.
- **Success criterion (EARS):**
  - **WHEN** `run_command` args carry a non-empty argv whose `argv[0]` is in the allow-list, **THEN** validation **SHALL** accept them.
  - **WHEN** argv is empty, `argv[0]` contains a path separator, or `argv[0]` is not in the allow-list, **THEN** validation **SHALL** reject with a denial and the command **SHALL NOT** reach execution.
  - **WHEN** the `command` field appears on a non-`run_command` tool, or a file field appears on `run_command`, **THEN** the shape check **SHALL** reject it.
- **Notes:** add `command: Option<Vec<String>>` to `TypedToolArgs`; extend `shape_for`
  disjointly like the existing file-tool checks. Bound argv count and total bytes with
  the existing `MAX_ARGUMENT_BYTES`. This task is pure — no process is spawned.

### T-003: CommandRunner execution capability
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Touches:** src/tools.rs, Cargo.toml, src/bin/cmd-fixture.rs
- **Depends on:** T-001, T-002
- **Acceptance criterion:** an allowed command runs with bounded captured output and a
  timeout; timeout, oversized output, non-zero exit, and a missing executable each
  produce a defined outcome.
- **Success criterion (EARS):**
  - **WHEN** an allow-listed command is executed, **THEN** `CommandRunner` **SHALL** spawn it as argv with cwd = workspace root and a scrubbed environment, and return its exit code with bounded captured output.
  - **WHEN** the combined stdout+stderr exceeds `MAX_COMMAND_OUTPUT_BYTES`, **THEN** the result **SHALL** be truncated at the cap and marked incomplete.
  - **WHEN** the command does not exit within the timeout, **THEN** `CommandRunner` **SHALL** kill the process group, including grandchildren, and return a timeout outcome.
  - **WHEN** the executable is missing or unspawnable, **THEN** it **SHALL** return an error outcome rather than panic.
  - **WHEN** the command exits non-zero, **THEN** the result **SHALL** report that exit code as a completed command (`Ok`).
  - **WHEN** an argv argument contains shell metacharacters, **THEN** it **SHALL** be passed literally as one argument.
- **Notes:** add `command-group` (tokio feature) for portable tree-kill (Unix process
  group + Windows Job Object). `env_clear()` then restore a minimal OS-appropriate set
  (`PATH` everywhere; `SystemRoot`/`SystemDrive`/`PATHEXT`/`TEMP`/`TMP` on Windows).
  The `cmd-fixture` `[[bin]]` is argv-driven (print, emit N bytes, exit code, sleep,
  spawn a marker-writing grandchild) so integration tests are deterministic and
  cross-platform via `CARGO_BIN_EXE_cmd-fixture`.

### T-004: Runner dispatch route, RunResources wiring, and journalling
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Touches:** src/runner.rs
- **Depends on:** T-001, T-002, T-003
- **Acceptance criterion:** a freeform run in a granted workspace runs an allowed
  command whose effect is journalled; denied without the grant; cancellation mid-run
  is a defined outcome.
- **Success criterion (EARS):**
  - **WHEN** a freeform run in a granting workspace calls `run_command` for an allow-listed command, **THEN** the runner **SHALL** execute it via `CommandRunner` and journal the effect in `tool_finished`.
  - **WHEN** a run's workspace does not grant `run_command`, **THEN** the call **SHALL** be denied and journalled as denied.
  - **WHEN** a run is cancelled while a command is in flight, **THEN** the runner **SHALL** kill the process group and record a defined killed outcome.
- **Notes:** map `"run_command" -> ToolName::RunCommand`; add a third dispatch route
  beside reader/writer, run as an async child killable via the existing `tokio::select!`
  on deadline/cancel (a process is actively killed, unlike the filesystem worker's
  detach-and-wait). Build a `command_runners` map in `RunResources::from_config` mirroring
  the `writers` map (`runner.rs:82`). Effects reuse `tool_planned`/`tool_finished`, so
  `inspect` surfaces them with no new command surface; the loop never auto-retries.

### T-005: Cross-platform CI matrix
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Touches:** .github/workflows/ci.yml
- **Depends on:** (none)
- **Acceptance criterion:** the process-tree cleanup and scrubbed-environment behavior,
  which differ by OS, are actually exercised on both Windows and Linux.
- **Success criterion (EARS):**
  - **WHEN** CI runs on a push or pull request, **THEN** it **SHALL** run `fmt`, `clippy`, and `cargo test --locked` on both `windows-latest` and `ubuntu-latest`.
- **Notes:** convert the single job to a `strategy.matrix.os` of
  `[windows-latest, ubuntu-latest]`, keeping the pinned 1.96.0 toolchain, the three
  check steps, and the 20-minute timeout. This task's verification is non-unit: the
  workflow's matrix content plus both OS jobs reporting green at the checkpoint,
  recorded under the test report's CI Confirmation (see the test plan).
