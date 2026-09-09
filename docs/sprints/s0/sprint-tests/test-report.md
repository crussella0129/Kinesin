# Sprint 0 Test Report

Verification provenance for INT-0001 (token accounting). All locked EARS
promises and both halves of the acceptance criteria are proved by named,
executed tests. One gap was found during this phase (`inspect` did not surface
counters) and closed; see `critique.md` C-001.

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | usage captured on the non-stream path | T-001 / `usage_captured_from_nonstream_response` | pass | Test evidence links this report |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | absence stays unknown, never zero (Koil) | T-001 / `usage_absent_when_response_omits_it` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | usage captured on the streaming path | T-001 / `usage_captured_from_stream_usage_chunk` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | streamed usage is requested | T-002 / `streaming_request_asks_for_usage` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | non-stream request unchanged | T-002 / `nonstream_request_omits_stream_options` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | tokens journalled per call | T-003 / `model_finished_carries_tokens_when_reported` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | per-run totals summed | T-003 / `reported_usage_accumulates_into_terminal_counters` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | honest absence at run level | T-003 / `absent_usage_omits_token_totals` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | **stored record** includes token totals when reported | E2E / `test_cli_run_records_token_totals` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | **`inspect` output** includes token totals when reported | E2E / `inspect_surfaces_token_totals_when_reported` | pass | " |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | `inspect` output honest absence | E2E / `inspect_omits_token_totals_when_unreported` | pass | " |

INT-0001 is verified. It is eligible for `realized` once completion evidence is
recorded (Loop Phase); this report is its Test evidence.

## Summary
- Unit tests: 124 passed / 0 failed / 124 total (crate `--lib`; includes the 5
  named model.rs usage units and the in-crate `test_cli_run_records_token_totals`
  E2E).
- Integration tests: 82 passed / 0 failed / 82 total (all `tests/*.rs` binaries;
  includes the 3 named `runner_tools.rs` accumulation tests and the 3
  `cli_inspect.rs` binary tests).
- E2E tests: executed (not N/A) — 3 named tests across `src/cli.rs` and
  `tests/cli_inspect.rs`, counted within the totals above.
- Whole suite: **206 passed / 0 failed**.
- CI status: green locally; hosted CI (`.github/workflows/ci.yml`) runs at the
  sprint checkpoint push (see below).

## CI Confirmation
- **Head SHA:** `c66a23d0e1e03702480b7314e73fce3c6a69609d`
- **CI run:** pending — the hosted `ci.yml` workflow runs on the human-approved
  `dev → main` checkpoint (remote profile `github/main<-dev/human-approve`); this
  head has not yet been pushed.
- **Conclusion:** local canonical runner: success.
- **Confirmations (local, tested head `c66a23d`):**
  - `cargo fmt --all -- --check` — clean.
  - `cargo clippy --locked --all-targets --all-features -- -D warnings` — clean.
  - `cargo test --locked` — 206 passed, 0 failed.

## Failures
None.

## Technical Debt Identified
- Streaming end-to-end usage is verified at the unit level
  (`usage_captured_from_stream_usage_chunk`, `streaming_request_asks_for_usage`);
  a full streamed run that carries a usage chunk through to the terminal counters
  is not separately E2E-tested. Low risk — the non-stream path is E2E-covered and
  the streaming assembler's capture/summation share the same runner path — but a
  streaming E2E would tighten it. Candidate for a follow-up intent, not a blocker.

## Coverage Observations
- Both directions of the honest-absence contract are covered at all three layers
  (unit, integration, E2E), so unreported usage is proven to stay unknown rather
  than be recorded as zero.
- The inspect gap (critique C-001) is a reminder that "surfaced" must be tested
  through the actual command output, not a store query that bypasses the render
  path. The added `cli_inspect.rs` tests assert against real `kinesin inspect`
  JSON.
