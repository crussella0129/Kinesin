# Sprint 10 unit verification

The locked plan uses verification-case names. The table maps those cases to
executed Rust tests or named document/policy checks; it does not invent a test
function when a case spans several layers. Full-suite counts and the final CI
commit are recorded in [test-report.md](test-report.md).

| Locked clause | Executed evidence | Outcome |
| --- | --- | --- |
| T-001/A, INT-0021 | All 20 original intent assessments in the four audit chapters; seven new chapters; installed Book validator | Coverage and explicit ownership retained; see research report |
| T-002/A–B, INT-0022 | `usage_partial_fields_remain_unknown`, model streaming/nonstreaming usage units, `mixed_usage_never_claims_complete_totals`, `inspect_preserves_only_completely_reported_token_dimensions` | Independently known fields retained; missing/error/overflow exchanges cannot produce complete totals |
| T-003/A, INT-0022 | Core compaction units; `freeform_reads_compact_and_replay`, `checked_run_evidence_survives_compaction` | Whole eligible groups compact; checked evidence stays protected |
| T-003/B, INT-0022 | `continuation_initial_context_is_bounded` | Actual prior-answer framing is counted before admission |
| T-003/C, INT-0022 | `compaction_replay_version_compatibility`, `continuation_replay_validates_the_recorded_reference_origin` | Unsupported semantics refused; current continuation origin/hash/bytes validated |
| T-003/D, INT-0022 | `actual_session_cache_request_contract` | Actual session-prepared requests and replay hashes agree; live observations reported separately |
| T-004/A, INT-0022 | `command_encoded_budget_covers_escaping_and_invalid_utf8`, configured-budget/truncation command tests | Full nested JSON fits the cap, including escaping and invalid UTF-8 replacement |
| T-004/B, INT-0022 | `leader_observation_retains_pid_until_group_cleanup`, `cancelled_observation_preserves_cleanup_and_signal_status` (Linux), native process integration tests | Direct child remains unreaped until group cleanup; observation cancellation preserves ownership |
| T-005/A–B, INT-0022 | Forced setup-failure and full-enforcement units; production BPF filter tests; `command_runtime_grants_cannot_cover_private_state`, `command_runtime_grants_cannot_cover_credential_verifiers` | Partial enforcement refuses execution; x32/legacy ABI bypass blocked; private paths excluded from read grants |
| T-006/A, INT-0022 | Four `mcp::transport` tests including byte/frame counts under fragmented and cancelled reads; discovery/config cap tests | Bounds apply before SDK decoding, including unterminated input and tiny-frame floods |
| T-006/B, T-007/A–C, INT-0022 | MCP fixture, service and replay tests in integration report | Process ownership, admission, durable schema freeze and both authorization gates verified across layers |
| T-008/A, INT-0022 | `remote_plaintext_is_rejected`, `origin_accepts_loopback_http`, destination/redirect protocol tests | Non-loopback HTTPS and public-destination opt-in remain independent |
| T-009/A–B, INT-0021/0022 | `cargo deny check`, `cargo audit`, isolated negative duplicate policy; full assurance mapping review | All gates pass; removing exact base64 exception fails with exit 2 |
| T-010/A, INT-0022 | `move_publishes_a_file_and_never_overwrites`, `move_collision_after_absence_check_preserves_destination_and_source`, `move_competition_publishes_exactly_one_destination_and_keeps_loser_source`, `move_source_cleanup_failure_preserves_published_destination_and_reports_partial_effect` | Destination publication never replaces another entry; partial source cleanup is explicit |
| T-011/A–B, INT-0021/0022 | Integrated platform suites, independent review, installed Book/phase/tracked validators and remote checkpoint | Final evidence recorded in test report and sprint metadata |

The integration review records additional regressions discovered while joining
the repairs, including actual red-to-green evidence for verifier placement and
the mandatory x32 filter fix. The earlier unmodified green suite is a baseline,
not proof that each new regression was separately executed against old code.
