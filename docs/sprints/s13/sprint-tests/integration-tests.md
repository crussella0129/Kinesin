# Sprint 13 focused integration verification

All official checks below followed the [live confidence gate](e2e-tests.md#live-confidence-gate-passed) on 2026-09-20. Results were supplied by the implementation operator; the independent reviewer inspected the final source and assertions without rerunning them. [tested-source.json](tested-source.json) identifies the changed files against baseline `6bb16fb17dec84e7d269d88f666a4af9ee4ebc76`; all eleven hashes match T-113 implementation commit `d9547f8865b0703919f3ba3aea1f75abd7432932`.

| Command / suite | Final result |
| --- | --- |
| `cargo test --locked --lib --test runner_tools --test runner_journal --test managed_cli --test replay` — `runner_tools` | 25 passed. |
| Same command — `runner_journal` | 9 passed. |
| Same command — `managed_cli` | 3 passed. |
| Same command — `replay` | 22 passed. |
| `cargo test --locked --test preview -- --nocapture` | 6 passed; repeated once after the final test-file adjustment, again 6 passed. |

Together with the 206 library cases, this is 271 distinct passing cases across the selected targets. The repeated six preview cases are not counted twice. Formatting and Clippy passed as recorded in [unit-tests.md](unit-tests.md).

## Acceptance mapping

| Plan verification / intent | Executed evidence and assertions |
| --- | --- |
| `repaired_boundary_units`, `preview_service_config_rejected`; INT-0031 AC3/AC4, T-113 clauses 3/5 | `preview_grants_reject_service_configuration_and_checked_authority` rejects a service section with preview authority and rejects a checked workspace with the effectful grant. Existing library/runner targets exercise surrounding authority and journal behavior. |
| `preview_denies_outside_paths`; AC3/AC4, T-113 clause 4 | `real_http_enforces_selected_subtree_origin_methods_and_asset_limits` sends literal/encoded traversal through raw HTTP, checks denial and absence of outside marker bytes, and covers hidden/unsupported/oversized assets, directory listing, missing prefix, wrong Host/Origin and POST. `native_symlink_cannot_expose_an_outside_asset` checks a real file symlink and denied target bytes. |
| `preview_capability_integration`; AC3, T-113 clauses 1/4 | `actual_runner_denies_an_ungranted_preview_call` observes the real runner's `denied / tool_denied` event. `real_http_reloads_assets_and_keeps_one_preview_until_shutdown` asserts body bytes, security headers, HEAD behavior, URL reuse, refusal to change roots, updated bytes after file edits, survival of a completed turn's cancellation token, and listener closure. |
| `repaired_server_integration`; AC3/AC4, T-113 clauses 1/3 | `cancellation_deadline_and_drop_do_not_leave_a_listener` covers cancelled/expired startup, invalid paths, an actual live response and Drop cleanup. The real CLI/browser lifecycle and follow-up operation are independently recorded in E2E evidence. |
| Replay compatibility of the new effect; AC4 | `preview_capture_replays_after_listener_and_workspace_are_gone` starts preview through the runner, reads actual HTML, shuts down and removes the fixture, then confirms replay neither reopens the port nor recreates files. `preview_tool_version_preserves_legacy_unknown_tool_denials` checks old tools-2 denial semantics and rejects mismatched version/observation combinations. |
| `deferred_verification_record`; AC4, T-114 | The recorded live gate precedes these commands. The final critic assesses source, assertions, scoped platform evidence and retained model failures. |

The Windows symlink fixture actually executed: both `--nocapture` preview runs passed without the `UNAVAILABLE` privilege message. This is not a silently skipped negative path. Requests use real loopback sockets and bounded watchdogs, and temporary fixture paths are unique. Scripted model replies isolate runner/replay dispatch; the separate live exercise uses the actual local model.

Only native Windows was executed. No Linux or remote CI run was requested or reported, and no general autonomous model-competence claim follows from these passing harness checks.
