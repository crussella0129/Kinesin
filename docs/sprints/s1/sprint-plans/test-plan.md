Finalized - DO NOT EDIT

# Sprint 1 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | granted per workspace by operator config | T-001 / WHEN grant + non-empty allow-list THEN accept | `run_command_grant_requires_nonempty_allowlist` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | grant/allow-list must be coherent | T-001 / WHEN grant without list, or list without grant THEN reject | `commands_without_grant_is_rejected` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | allow-list entries are bare names | T-001 / WHEN entry has a path separator THEN reject | `command_name_with_separator_is_rejected` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | barred in checked runs | T-001 / WHEN checked workspace grants run_command THEN authorization refuses | `checked_workspace_cannot_grant_run_command` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | always an argv vector | T-002 / WHEN allow-listed argv THEN accept | `run_command_argv_accepted_when_allowlisted` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | reject bad/unlisted argv before execution | T-002 / WHEN empty/pathy/unlisted argv THEN deny, do not execute | `run_command_rejects_empty_or_unlisted_or_pathy_argv` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | argv is disjoint from file tools | T-002 / WHEN command on wrong tool or file field on run_command THEN reject | `command_field_is_disjoint_from_file_tools` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | runs with bounded output, cwd, scrubbed env | T-003 / WHEN allow-listed command runs THEN argv + cwd + scrubbed env + exit code + bounded output | `command_runs_and_captures_bounded_output`, `command_runs_with_scrubbed_env_and_workspace_cwd` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | oversized output outcome | T-003 / WHEN output exceeds cap THEN truncate + incomplete | `command_output_truncated_at_cap` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | timeout outcome + tree cleanup | T-003 / WHEN no exit within timeout THEN kill group incl. grandchildren | `command_timeout_kills_process_tree` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | missing executable outcome | T-003 / WHEN unspawnable THEN error, no panic | `command_missing_executable_errors` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | non-zero exit outcome | T-003 / WHEN non-zero exit THEN Ok with exit code | `command_nonzero_exit_reported` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | no shell parses the argv | T-003 / WHEN arg has metacharacters THEN passed literally | `run_command_passes_argv_without_shell_interpretation` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | effect is journalled in a granted run | T-004 / WHEN granted run calls run_command THEN execute + journal | `command_effect_is_journalled`, `test_cli_run_command_effect_is_journalled` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | denied without the grant | T-004 / WHEN workspace lacks grant THEN deny + journal denied | `run_command_denied_without_grant` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | cancellation mid-run outcome | T-004 / WHEN cancelled in flight THEN kill group + killed outcome | `command_cancelled_midrun_is_killed` |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | cross-OS process behavior is exercised | T-005 / WHEN CI runs THEN suite runs on windows-latest and ubuntu-latest | CI matrix (non-unit; see End-to-End Tests) |

## Unit Tests
### T-001 unit tests (`src/config.rs`)
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- `run_command_grant_requires_nonempty_allowlist`: grant + `commands=["cmd-fixture"]` → config accepted.
- `commands_without_grant_is_rejected`: grant with empty `commands`, and `commands` set without the grant → both rejected.
- `command_name_with_separator_is_rejected`: `commands=["../evil"]` or `"a/b"` → rejected.
- `checked_workspace_cannot_grant_run_command`: a checked task whose workspace grants `run_command` → `authorize` errors (run_command is mutating).

### T-002 unit tests (`src/tools.rs`)
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- `run_command_argv_accepted_when_allowlisted`: `["cmd-fixture","--print","hi"]` with allow-list → validation ok.
- `run_command_rejects_empty_or_unlisted_or_pathy_argv`: `[]`, `["/bin/sh"]`, `["../x"]`, and `["not-listed"]` → each denied, no execution.
- `command_field_is_disjoint_from_file_tools`: `command` present on `write_file`, or `content` present on `run_command` → shape check rejects.

## Integration Tests
### CommandRunner execution (`tests/command_tool.rs`, uses `CARGO_BIN_EXE_cmd-fixture`)
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- `command_runs_and_captures_bounded_output`: fixture prints a message, exits 0 → `Ok`, exit_code 0, captured output present, complete.
- `command_runs_with_scrubbed_env_and_workspace_cwd`: fixture reports its cwd and a probe env var → cwd is the workspace root and the probe var (set in the parent) is absent from the child.
- `command_output_truncated_at_cap`: fixture emits `MAX_COMMAND_OUTPUT_BYTES + N` bytes → output truncated at the cap, marked incomplete.
- `command_nonzero_exit_reported`: fixture exits 7 → `Ok` with exit_code 7.
- `command_missing_executable_errors`: allow-listed name that is not present on PATH → `Error` (`spawn_failed`), no panic.
- `command_timeout_kills_process_tree`: fixture sleeps beyond the timeout and spawns a grandchild that appends to a marker file → after the timeout the group is killed and the marker stops growing (grandchild dead).
- `run_command_passes_argv_without_shell_interpretation`: an argument of `"a; rm -rf b && c"` (or `"%PATH%"`) is echoed back by the fixture as exactly one unmodified argument.

### Runner integration (`tests/runner_tools.rs`)
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- `run_command_denied_without_grant`: a workspace without `run_command` → the call is denied and `tool_finished` records `dispatch: denied`.
- `command_effect_is_journalled`: a granting workspace runs the fixture → `tool_finished` records the command effect (tool `run_command`, dispatch `executed`).
- `command_cancelled_midrun_is_killed`: cancel the run while the fixture sleeps → the group is killed and a defined (killed/unsent) outcome is journalled.

## End-to-End Tests
- **Status:** possible
- `test_cli_run_command_effect_is_journalled` (`tests/cli_inspect.rs`): a real
  `kinesin run --workspace … --model … --prompt … --allow-unchecked` in a granting
  workspace drives the fixture through the loopback model, then a subprocess
  `kinesin inspect --run <id>` shows the `run_command` effect journalled with its exit code.
- **CI matrix (T-005) — non-unit verification:** the acceptance that cross-OS process
  behavior is exercised is verified by the workflow content (matrix names both
  `windows-latest` and `ubuntu-latest`) and by both OS jobs reporting green at the sprint
  checkpoint, recorded under the Test Phase report's CI Confirmation. It has no `cargo`
  test because a workflow matrix is configuration, not library behavior.

**Replay:** the added `tool_planned`/`tool_finished` fields are additive and optional;
existing replay tests must pass unchanged. `run_command` has no prepared-request
fingerprint, so no replay capture shifts.
