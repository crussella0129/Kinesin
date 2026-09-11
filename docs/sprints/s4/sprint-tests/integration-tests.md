# Sprint 4 Integration Tests

- **Intent:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- **Tested head:** `a8de1808ee9d74e62b6563c308c9568469441edd`
- **Runner:** `cargo test --locked --test runner_tools`
- **Result:** all green (23 tests; 2 new for T-002).

## Uniform transport (`tests/runner_tools.rs`)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `uniform_attach_prepares_identically_across_local_and_overlay_backends` | WHEN a run is configured with a backend at a private/overlay address THEN it prepares/dispatches through the same code path as loopback, prepared bytes identical modulo the origin | ok |
| `unreachable_backend_reports_not_ready` | WHEN a configured backend is unreachable THEN readiness reports not-ready with a defined outcome and no hang | ok |

The first test runs the same scripted checked run against a loopback `base_url`
and a CGNAT-overlay `base_url` (`http://100.100.20.30:8080`) and asserts the
captured prepared requests are byte-identical and acceptance is identical — the
backend address never enters the request body, and the overlay config parsing at
all is the T-001 admissibility win. The second points a real `ModelClient::http`
at a bound-then-dropped port and bounds `ready()` with a 5 s timeout, asserting a
`false` (no hang).

## Confirmation
```
test uniform_attach_prepares_identically_across_local_and_overlay_backends ... ok
test unreachable_backend_reports_not_ready ... ok
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Existing integration suites remain green (loopback origins are still accepted, so
no prior test regressed): full `cargo test --locked` reports 0 failed across all
binaries.
