# Sprint 12 integration verification

After the real local workflow worked, Windows Rust 1.96.0 passed:
`cargo test --locked --test cli_inspect --test replay --test runner_tools --test managed_cli`.
Raw output: ignored `target/s12-live/integration-tests.log`.

| Suite | Result |
| --- | --- |
| cli_inspect | 17 passed |
| replay | 20 passed |
| runner_tools | 25 passed |
| managed_cli | 3 passed |

With 206 library tests, **271 tests passed**, zero ignored in these suites.
The initial CLI run caught two incorrect test expectations omitting preserved
input newlines. Corrected assertions without changing product behavior; the
whole CLI suite then passed.

| Task / EARS | Executed test | Observable contract |
| --- | --- | --- |
| T-110 replay | `session_capture_replays_frozen_context_and_rejects_tampering` | Actual runner requests replay with matching hashes; forged history/source and legacy stamp rejected |
| T-110 metadata | `metadata_session_capture_retains_provenance_without_history_content` | History content absent; source size/digest present |
| T-110 legacy | `legacy_capture_three_core_two_replays_without_session_context` | Previous format reproduces with/without prior answer |
| T-111 recent memory/reset | `human_session_remembers_user_requests_across_turns_and_new_clears_memory` | Three actual CLI requests retain first prompt; commands avoid model calls; /new clears |
| T-111 oversized/failed | `human_session_shortens_large_answers_and_preserves_completed_memory_after_failure` | >8 KiB Unicode reply cannot poison next request; failed entry preserves prior context |
| T-111 JSON | `explicit_json_session_keeps_stdout_as_structured_receipts` | Machine output stays parseable |
| T-111 file work | `a_freeform_run_writes_a_file_and_records_the_effect`, `a_freeform_run_edits_a_unique_passage_end_to_end` | Actual effects and permission checks |
| T-111 lifecycle | `normal_session_eof_settles_run_and_stops_managed_tree`, `managed_backend_death_exits_an_idle_session_without_waiting_for_stdin`, `managed_cli_ctrl_c_during_startup_stops_tree_before_any_admission` | Cleanup, child death and cancellation |

Independent review found and fixed premature eviction on failed requests and
clarified that /new clears live context, not durable records. Final source
review found no further production defects. Hosted CI was not run; no push or
merge is authorized by this request.
