# Test Critique — Sprint 0

Adversarial read-only screen of the token-accounting evidence against the locked
`build-plan.md` / `test-plan.md` and INT-0001's acceptance criteria. One concern
was raised on the first pass and closed before this final pass; it is recorded
here with its resolution for provenance.

## Concerns

### C-001: `inspect` output did not include token totals (RESOLVED)
- **Where:** `INT-0001` Acceptance criteria / `e2e-tests.md` / `src/cli.rs` inspect handler.
- **Quote:** INT-0001 — "A completed run's stored record **and `inspect` output** include prompt and completion token totals when the server reports them." Build plan T-003 — "`inspect` already reads events and the terminal record, so no new command surface."
- **Failure mode:** intent-coverage / e2e-cop-out.
- **Why it matters:** The build plan assumed `inspect` surfaces the totals for free. It does not: `inspect`'s `EventSummary` carries only `seq`/`kind`/`elapsed_ms` and drops the event body, and the counters ride in the `run_finished` event `data`, not the `RunRecord`. So `inspect` never surfaced counters at all. The original E2E asserted the totals via a raw `Command::Events` query, which read the stored event directly and masked the gap — the acceptance criterion's "`inspect` output" half was unproved and, in fact, unmet.
- **Suggested response:** add-test **and** fix implementation (the smallest change that meets the criterion; not re-architecture).
- **Resolution:** `inspect` now lifts the `run_finished` counters into a `counters` field on the inspect output line during its existing event scan (commit `c66a23d`). Two real-binary tests were added in `tests/cli_inspect.rs`: `inspect_surfaces_token_totals_when_reported` (asserts summed `prompt_tokens`/`completion_tokens` in `kinesin inspect` JSON) and `inspect_omits_token_totals_when_unreported` (asserts honest absence). The intent was not amended — the desired outcome did not change.

## Screen of the remaining failure modes (final pass)

- **Intent/EARS trace gap:** none. Every EARS clause maps to a named executed test (`test-plan.md` traceability table), and both halves of the "stored record and inspect output" acceptance criterion now have executed tests.
- **Assertion weakness:** none. Tests assert exact counts (40/8, 95/20, 17/4, 39/15) and `is_none()` for absence, not mere presence.
- **Stub leakage:** none. The scripted `ModelClient` carries `Usage` as contract data; the E2E paths use real HTTP/SQLite and the real binary.
- **Integration drift:** none. Integration covers the runner journal/accumulation boundary (T-003), distinct from the Koil parse unit layer.
- **Negative-path absence:** covered. Honest-absence has `usage_absent_when_response_omits_it` (unit), `absent_usage_omits_token_totals` (integration), and `inspect_omits_token_totals_when_unreported` (E2E).
- **Flake risk:** low. Scripted tests use `Duration::ZERO` and single-slot resources; binary/loopback tests use OS-assigned ports, bounded deadlines, and `no_model_calls()` guards.
- **Evidence drift:** none. All result artifacts name tested head `c66a23d0e1e03702480b7314e73fce3c6a69609d` and their runners; the report attaches as Test evidence to INT-0001.

## Confidence
clean
