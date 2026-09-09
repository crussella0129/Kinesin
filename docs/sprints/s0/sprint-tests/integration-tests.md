# Sprint 0 Integration Tests

- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- **Tested head:** `c66a23d0e1e03702480b7314e73fce3c6a69609d`
- **Runner:** `cargo test --locked --test runner_tools`
- **Result:** all green (see per-test confirmations below).

These exercise the runner journalling path end of Koil→runner, above the unit
layer: a scripted `ModelClient` carries `Usage` through a real admitted run and
into the immutable journal and terminal counters.

## Executed tests (`tests/runner_tools.rs`)

| Test | EARS clause (T-003) | Result |
|------|---------------------|--------|
| `model_finished_carries_tokens_when_reported` | WHEN a model call reports usage THEN the runner includes `prompt_tokens`/`completion_tokens` in that `model_finished` event | ok |
| `reported_usage_accumulates_into_terminal_counters` | WHEN a run finishes and at least one call reported usage THEN terminal `counters` include the summed totals | ok |
| `absent_usage_omits_token_totals` | WHEN no call reported usage THEN terminal `counters` omit token totals rather than record zero | ok |

## Assertions of note

- `model_finished_carries_tokens_when_reported`: one scripted call reporting
  `Usage{40, 8}` → its single `model_finished` event asserts
  `prompt_tokens == 40`, `completion_tokens == 8`.
- `reported_usage_accumulates_into_terminal_counters`: two calls reporting
  `Usage{40, 8}` and `Usage{55, 12}` → terminal `counters` assert
  `prompt_tokens == 95`, `completion_tokens == 20` (summed).
- `absent_usage_omits_token_totals`: a bare reply → asserts
  `counters.get("prompt_tokens").is_none()` and the same for
  `completion_tokens`, proving honest absence at the run level.

## Confirmation

```
test absent_usage_omits_token_totals ... ok
test model_finished_carries_tokens_when_reported ... ok
test reported_usage_accumulates_into_terminal_counters ... ok
```

Determinism: the scripted client uses `Duration::ZERO` delays and a
single-slot `RunResources`, so there is no timing, network, or shared-state
dependence.
