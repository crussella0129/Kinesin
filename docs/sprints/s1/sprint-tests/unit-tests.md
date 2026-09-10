# Sprint 1 Unit Tests

- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Tested head:** `13ffdc3fcf1d7895d37629f3f46899780f7ce601`
- **Runners:** `cargo test --locked --lib config::tests`, `cargo test --locked --lib tools::tests`
- **Result:** all green.

## T-001 — config surface, allow-list, checked-run bar (`src/config.rs`)
| Test | EARS clause | Result |
|------|-------------|--------|
| `run_command_grant_requires_nonempty_allowlist` | grant + non-empty `commands` → accept | ok |
| `commands_without_grant_is_rejected` | grant with empty list, or list without grant → reject | ok |
| `command_name_with_separator_is_rejected` | a `commands` entry with a path separator / dot → reject | ok |
| `checked_workspace_cannot_grant_run_command` | a checked task's workspace grants run_command → authorization refuses | ok |

## T-002 — argv args, shape, allow-list validation (`src/tools.rs`)
| Test | EARS clause | Result |
|------|-------------|--------|
| `run_command_argv_accepted_when_allowlisted` | non-empty argv, allow-listed `argv[0]` → accept | ok |
| `run_command_rejects_empty_or_unlisted_or_pathy_argv` | empty / path-bearing / unlisted argv → deny, no execution | ok |
| `command_field_is_disjoint_from_file_tools` | `command` on a file tool, or a file field on run_command → reject | ok |

## Confirmation
```
test config::tests::run_command_grant_requires_nonempty_allowlist ... ok
test config::tests::checked_workspace_cannot_grant_run_command ... ok
test config::tests::commands_without_grant_is_rejected ... ok
test config::tests::command_name_with_separator_is_rejected ... ok
test tools::tests::command_field_is_disjoint_from_file_tools ... ok
test tools::tests::run_command_argv_accepted_when_allowlisted ... ok
test tools::tests::run_command_rejects_empty_or_unlisted_or_pathy_argv ... ok
```

The checked-run bar reuses `ToolName::is_mutating`, so `run_command` is barred from
checked runs by the same authorization rule that bars file writes — no new bar.
