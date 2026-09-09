# Sprint 0 Unit Tests

- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- **Tested head:** `c66a23d0e1e03702480b7314e73fce3c6a69609d`
- **Runner:** `cargo test --locked --lib model::tests`
- **Result:** all green (see per-test confirmations below).

## Executed tests (`src/model.rs`)

| Test | EARS clause (task) | Result |
|------|--------------------|--------|
| `usage_captured_from_nonstream_response` | T-001 · WHEN decode_reply parses a `usage` object THEN return `Some(Usage)` with prompt/completion counts | ok |
| `usage_absent_when_response_omits_it` | T-001 · WHEN a response omits `usage` THEN return `None` (never zero) | ok |
| `usage_captured_from_stream_usage_chunk` | T-001 · WHEN the streaming assembler observes its usage chunk THEN capture the counts | ok |
| `streaming_request_asks_for_usage` | T-002 · WHEN `prepare` builds `stream = true` THEN body includes `stream_options.include_usage = true` | ok |
| `nonstream_request_omits_stream_options` | T-002 · WHEN `prepare` builds `stream = false` THEN body omits `stream_options` | ok |

## Confirmation

```
test model::tests::usage_absent_when_response_omits_it ... ok
test model::tests::usage_captured_from_nonstream_response ... ok
test model::tests::nonstream_request_omits_stream_options ... ok
test model::tests::streaming_request_asks_for_usage ... ok
test model::tests::usage_captured_from_stream_usage_chunk ... ok
```

Both positive (`Some(Usage)`) and honest-absence (`None`) paths are asserted, so
unreported usage stays unknown rather than being recorded as zero.
