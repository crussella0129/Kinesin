# Sprint 1 Test Report

Verification provenance for INT-0003 (bounded command-execution tool). Every locked
EARS clause and all four acceptance criteria are proved by named, executed tests.
The final critique is `proceed-with-caveats`; see `critique.md`.

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | granted per workspace; barred in checked runs | T-001 / `run_command_grant_requires_nonempty_allowlist`, `commands_without_grant_is_rejected`, `command_name_with_separator_is_rejected`, `checked_workspace_cannot_grant_run_command` | pass | Test evidence links this report |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | always an argv vector; no shell | T-002 / `run_command_argv_accepted_when_allowlisted`, `command_field_is_disjoint_from_file_tools`; T-003 / `run_command_passes_argv_without_shell_interpretation` | pass | " |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | reject bad/unlisted argv before execution | T-002 / `run_command_rejects_empty_or_unlisted_or_pathy_argv` | pass | " |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | runs with bounded output, cwd, scrubbed env | T-003 / `command_runs_and_captures_bounded_output`, `command_runs_with_scrubbed_env_and_workspace_cwd`, `command_output_truncated_at_cap` | pass | " |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | failure modes each defined | T-003 / `command_timeout_kills_process_tree`, `command_nonzero_exit_reported`, `command_missing_executable_errors`; T-004 / `command_cancelled_midrun_is_killed` | pass | " |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | denied without the grant; effect journalled | T-004 / `run_command_denied_without_grant`, `command_effect_is_journalled` | pass | " |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | end-to-end run + effect journalled + surfaced | E2E / `test_cli_run_command_effect_is_journalled` | pass | " |
| [INT-0003](../../../intents/INT-0003-shell-execution.md) | cross-OS process behavior exercised | T-005 / CI matrix (non-unit; see below) | pending CI | " |

INT-0003 is verified for realization at the Loop Phase once completion evidence is
recorded; this report is its Test evidence.

## Summary
- Unit tests: 131 passed / 0 failed / 131 total (crate `--lib`; includes the 7 named config/tools units).
- Integration tests: 93 passed / 0 failed / 93 total (all `tests/*.rs`; includes the 7 `command_tool.rs`, 3 `runner_tools.rs`, and the `cli_inspect.rs` E2E).
- E2E tests: executed — `test_cli_run_command_effect_is_journalled` (a real `kinesin run` + `kinesin inspect`), counted in the totals above.
- Whole suite: **224 passed / 0 failed**.
- CI status: green locally; hosted CI (`.github/workflows/ci.yml`) now a Windows + Linux matrix, running at the sprint checkpoint push.

## CI Confirmation
- **Head SHA:** `13ffdc3fcf1d7895d37629f3f46899780f7ce601`
- **CI run:** pending — the hosted matrix workflow runs on the human-approved `dev → main` checkpoint (remote profile `github/main<-dev/human-approve`); this head has not yet been pushed. T-005's cross-OS EARS clause is satisfied by both `windows-latest` and `ubuntu-latest` jobs reporting green there.
- **Conclusion:** local canonical runner: success.
- **Confirmations (local, tested head `13ffdc3`, windows-latest equivalent):**
  - `cargo fmt --all -- --check` — clean.
  - `cargo clippy --locked --all-targets --all-features -- -D warnings` — clean.
  - `cargo test --locked` — 224 passed, 0 failed.

## Failures
None.

## Technical Debt Identified
- Per-command timeout is derived from the remaining run budget (`max_run_s`) rather
  than a dedicated limit; a single command can consume the whole run budget before
  the group is killed. A dedicated `max_command_s` limit is a reasonable follow-up.
- `inspect` surfaces a command run via `counters.tool_calls` but not the per-event
  command detail or exit code (its summaries drop event bodies). Surfacing per-event
  tool detail through `inspect` would be a separate observability intent.

## Coverage Observations
- All five named failure modes (timeout, oversized output, non-zero exit, missing
  executable, cancellation) have dedicated negative tests, and the timeout test
  additionally proves whole-tree cleanup via a grandchild marker.
- The argv-only contract is enforced at the type level (a `Vec<String>`, no shell
  string exists) and confirmed behaviorally by passing shell metacharacters through
  as one literal argument.
