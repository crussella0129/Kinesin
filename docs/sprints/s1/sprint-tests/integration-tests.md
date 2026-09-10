# Sprint 1 Integration Tests

- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Tested head:** `13ffdc3fcf1d7895d37629f3f46899780f7ce601`
- **Runners:** `cargo test --locked --test command_tool`, `cargo test --locked --test runner_tools`
- **Result:** all green.

## CommandRunner execution (`tests/command_tool.rs`, via `CARGO_BIN_EXE_cmd-fixture`)
These spawn the real, argv-driven `cmd-fixture` binary — never a shell — so the
process machinery is exercised on both Windows and Linux.

| Test | EARS clause (T-003) | Result |
|------|---------------------|--------|
| `command_runs_and_captures_bounded_output` | allow-listed command runs → Ok, exit code, bounded output | ok |
| `command_runs_with_scrubbed_env_and_workspace_cwd` | cwd is the workspace root; a parent secret does not reach the child | ok |
| `command_output_truncated_at_cap` | output beyond `MAX_COMMAND_OUTPUT_BYTES` → truncated + incomplete | ok |
| `command_nonzero_exit_reported` | non-zero exit → Ok with the exit code | ok |
| `command_missing_executable_errors` | allow-listed but absent executable → Error (`spawn_failed`), no panic | ok |
| `command_timeout_kills_process_tree` | no exit within the timeout → group (incl. grandchild) killed | ok |
| `run_command_passes_argv_without_shell_interpretation` | metacharacter argument reaches the program as one literal argument | ok |

The tree-kill test spawns a grandchild that appends to a marker every 50ms; after
the timeout it stops growing, proving the whole group (Windows Job Object / Unix
process group) was terminated, not just the direct child.

## Runner integration (`tests/runner_tools.rs`)
| Test | EARS clause (T-004) | Result |
|------|---------------------|--------|
| `command_effect_is_journalled` | a granted run's `run_command` is executed and journalled in `tool_finished` (dispatch `executed`, classification `ok`) | ok |
| `run_command_denied_without_grant` | a workspace without the grant → denied and journalled `denied` | ok |
| `command_cancelled_midrun_is_killed` | a run cancelled mid-command settles well before the 60s sleep / run deadline (the group was killed) | ok |

## Confirmation
```
test command_runs_and_captures_bounded_output ... ok
test command_runs_with_scrubbed_env_and_workspace_cwd ... ok
test command_output_truncated_at_cap ... ok
test command_nonzero_exit_reported ... ok
test command_missing_executable_errors ... ok
test command_timeout_kills_process_tree ... ok
test run_command_passes_argv_without_shell_interpretation ... ok
test run_command_denied_without_grant ... ok
test command_effect_is_journalled ... ok
test command_cancelled_midrun_is_killed ... ok
```

Determinism: the fixture uses fixed argv behaviors; timeouts are explicit and the
cancellation test bounds settlement with `tokio::time::timeout` so a broken kill
fails loudly rather than hanging.
