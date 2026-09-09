# Sprint 0 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | usage captured on the non-stream path | T-001 / WHEN decode_reply parses a usage object THEN return Some(Usage) | `usage_captured_from_nonstream_response` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | absence stays unknown, never zero | T-001 / WHEN a response omits usage THEN return None | `usage_absent_when_response_omits_it` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | usage captured on the streaming path | T-001 / WHEN the assembler observes its usage chunk THEN capture counts | `usage_captured_from_stream_usage_chunk` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | streamed usage is requested | T-002 / WHEN stream=true THEN include stream_options.include_usage | `streaming_request_asks_for_usage` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | non-stream request unchanged | T-002 / WHEN stream=false THEN no stream_options | `nonstream_request_omits_stream_options` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | tokens journalled per call | T-003 / WHEN a call reports usage THEN model_finished carries tokens | `model_finished_carries_tokens_when_reported` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | per-run totals surfaced | T-003 / WHEN a run finishes with reported usage THEN counters include summed tokens | `reported_usage_accumulates_into_terminal_counters` |
| [INT-0001](../../../intents/INT-0001-token-accounting.md) | honest absence at run level | T-003 / WHEN no call reported usage THEN counters omit token totals | `absent_usage_omits_token_totals` |

## Unit Tests
### T-001 unit tests (`src/model.rs`)
- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- `usage_captured_from_nonstream_response`: a `stop` response body carrying
  `{"prompt_tokens":37,"completion_tokens":6}` → `Some(Usage{37,6})`.
- `usage_absent_when_response_omits_it`: a response with no `usage` → `None`.
- `usage_captured_from_stream_usage_chunk`: an SSE sequence ending in a usage
  chunk → `Some(Usage)` with the chunk's counts.

### T-002 unit tests (`src/model.rs`)
- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- `streaming_request_asks_for_usage`: `prepare` with `stream=true` → body has
  `stream_options.include_usage == true`.
- `nonstream_request_omits_stream_options`: `prepare` with `stream=false` → body
  has no `stream_options` key.

## Integration Tests
### T-003 runner integration (`tests/runner_tools.rs` / `tests/runner_journal.rs`)
- **Intents:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- `model_finished_carries_tokens_when_reported`: a scripted step carrying usage →
  its `model_finished` event data includes `prompt_tokens`/`completion_tokens`.
- `reported_usage_accumulates_into_terminal_counters`: two scripted calls each
  reporting usage → terminal `counters` show the summed totals.
- `absent_usage_omits_token_totals`: an ordinary scripted run (no usage) →
  terminal `counters` omit token totals (no zero fields).

## End-to-End Tests
- **Status:** possible
- `test_cli_run_records_token_totals`: the CLI batch loopback fixture in
  `src/cli.rs` returns a `usage` object in its response; the stored run's terminal
  `counters`, read back through the run record, show the token totals.

Replay coverage: added event fields are additive and optional; existing replay
tests must still pass unchanged, proving replay does not require usage. The
streaming request fingerprint change (T-002) is covered by first confirming
whether any replay capture is a streaming request — replay compares the
recomputed prepared-request fingerprint to the recorded `model_planned`
`request_sha256`, so a pre-change streaming capture would mismatch — then
re-recording it or confirming none exists. Non-streaming replay is unaffected.
