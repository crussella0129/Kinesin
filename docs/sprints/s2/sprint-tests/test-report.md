# Sprint 2 Test Report

Verification provenance for INT-0002 (context compaction). Every locked EARS clause
and all acceptance criteria are proved by named, executed tests. The final critique
is `proceed-with-caveats`; see `critique.md`.

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | a run at the limit continues under a bounded policy | T-002 / `run_continues_past_history_limit_by_compacting`, `test_cli_run_compacts_past_history_limit` | pass | Test evidence links this report |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | bounded — stops when nothing droppable | T-002 / `compaction_stops_when_nothing_droppable`; T-001 / `drop_oldest_returns_false_when_only_protected_remain` | pass | " |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | complete tool-call/result groups preserved; no partial group | T-001 / `drop_oldest_removes_whole_group` | pass | " |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | evidence bytes never dropped; checked acceptance unaffected | T-001 / `drop_oldest_preserves_evidence_bearing_result`; T-002 / `checked_run_evidence_survives_compaction` | pass | " |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | protects system + recent floor | T-001 / `drop_oldest_preserves_system_and_recent_floor` | pass | " |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | immutability and replay contracts hold | T-002 / `replay_reproduces_a_compacted_run` | pass | " |
| [INT-0002](../../../intents/INT-0002-context-compaction.md) | explicit policy with a configured floor | T-001 / `compaction_policy_validates_floor` | pass | " |

INT-0002 is verified for realization at the Loop Phase once completion evidence is
recorded; this report is its Test evidence.

## Summary
- Unit tests: 136 passed / 0 failed / 136 total (crate `--lib`; includes the 4 core drop tests and the config policy test).
- Integration tests: 98 passed / 0 failed / 98 total (all `tests/*.rs`; includes the 3 `runner_tools.rs` compaction tests, the `replay.rs` reproduction test, and the `cli_inspect.rs` E2E).
- E2E tests: executed — `test_cli_run_compacts_past_history_limit` (real `kinesin run` + `kinesin inspect`), counted in the totals above.
- Whole suite: **234 passed / 0 failed**.
- CI status: green locally; hosted Windows + Linux matrix runs at the sprint checkpoint.

## CI Confirmation
- **Head SHA:** `b22990b94966371a73f27f52836a00b2dec34dcf`
- **CI run:** pending — the hosted matrix workflow runs on the human-approved `dev → main` checkpoint (remote profile `github/main<-dev/human-approve`); this head has not yet been pushed.
- **Conclusion:** local canonical runner: success.
- **Confirmations (local, tested head `b22990b`):**
  - `cargo fmt --all -- --check` — clean.
  - `cargo clippy --locked --all-targets --all-features -- -D warnings` — clean.
  - `cargo test --locked` — 234 passed, 0 failed.

## Failures
None.

## Technical Debt Identified
- Compaction is drop-oldest only. A summary-of-older-turns strategy (which would add
  a model call and non-deterministic text to capture/replay) remains a possible
  future intent, as recorded in INT-0002's Alternatives.
- The boundary tests are byte-tuned (bulky content vs. a small `max_history_bytes`);
  they are deterministic but brittle to a future change in conversation serialization
  overhead. A helper that computes the threshold from live sizes would harden them.

## Coverage Observations
- The evidence contract is covered at two layers: the unit drop test proves an
  evidence group is skipped, and the checked-run integration test proves acceptance
  is unaffected end to end.
- The replay reproduction test is the key guarantee that compaction did not disturb
  the immutability/replay contract — it would fail if replay did not apply the same
  deterministic drops.
