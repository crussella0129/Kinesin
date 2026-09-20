# Sprint 12 unit verification

Windows Rust 1.96.0: the targeted command's library suite passed all **206**
unit tests after implementation and the real local walkthrough. Raw output:
ignored `target/s12-live/targeted-tests.log`. Final source commits are linked
from the report; testing used their working-tree content.

| Task / EARS | Executed test | Assertion |
| --- | --- | --- |
| T-110 recent turns | `session_context_bounds_use_encoded_bytes_and_preserve_order` | Order and encoded size/count |
| T-110 invalid context | `session_context_rejects_empty_text_and_ambiguous_origins` | Empty, invalid or duplicate origins rejected |
| T-110 fresh authority | `session_context_declares_provenance_and_identity_without_granting_tools` | History changes identity, not grants |
| T-110 invalid combinations | `session_authority_rejects_checked_legacy_and_invalid_context` | Checked/legacy combinations and oversized initial history rejected |
| T-111 long answer | `memory_preserves_unicode_ends_when_shortening_a_turn` | Bounded Unicode-safe head/tail and clipping notice |
| T-111 encoded bounds | `escaped_text_fits_the_encoded_budget_even_for_tiny_windows` | JSON escapes count against byte allowance |
| T-111 bounded/reset memory | `memory_keeps_the_newest_turns_and_clear_resets_context_and_counts` | Oldest eviction, newest retention, reset |
| T-111 request admission | `session_admission_fits_both_history_and_the_compiled_tool_request` | Initial history and compiled schema/envelope limits |
| T-111 failed requests | `rejected_session_requests_preserve_completed_context` | Invalid request does not mutate memory |

`cargo fmt --all -- --check`, `git diff --check` and
`cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
No dependencies changed. No full-repository or Linux test result is inferred.
