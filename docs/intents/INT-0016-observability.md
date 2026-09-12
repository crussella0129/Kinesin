# INT-0016 — Observability (OpenTelemetry traces & metrics)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0016
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Add production observability: OpenTelemetry-compatible traces and metrics for the
runtime — admission, scheduling, model calls, tool dispatch, compaction, run
lifecycle — so operators can see latency, throughput, saturation, and error rates
across concurrent runs. Spans and metrics MUST NOT contain prompts, model output,
tool arguments/results, or credentials (extending the existing "operational
metrics do not contain prompts" rule). This also finally measures the runtime
overhead adversarial-review.md flagged as unknown. Non-goals: shipping a metrics
backend (export to the operator's collector); logging content; a bespoke telemetry
format when OTel is the field standard.

## Acceptance criteria
- Disabled and enabled telemetry overhead are measured against a recorded baseline and an explicit acceptance bound; zero overhead is not inferred from configuration alone.
- The runtime emits OTel traces and metrics behind an operator-configured exporter
  (off by default); with no exporter configured, behavior is unchanged and overhead
  remains within the recorded acceptance bound.
- No span, metric label, or log line contains a prompt, completion, tool
  argument/result, or credential — proven by a test that scans emitted telemetry.
- Metrics cover the runtime's own overhead (queue wait, scheduler, tool dispatch,
  journal commit) separately from model inference time, so the "runtime overhead
  unknown" gap is closed with a recorded baseline.
- Traces correlate to a run by owner-scoped run id without leaking cross-owner data.

## Rationale
OpenTelemetry tracing is the 2026 standard target for agent runtimes, and a
production harness is operable only if its latency/saturation/error behavior is
visible. It also turns the adversarial-review "latency overhead unknown" into a
measured, regressible number.

## Alternatives
Rely on the SQLite journal + `inspect` alone (current: per-run forensics, not
fleet metrics, and no standard export). Prometheus-only metrics without traces
(loses causal spans across the async pipeline). A custom telemetry module
(rejected — OTel is the interoperable standard).

## Consequences
A tracing dependency and exporter wiring; discipline to keep content out of
telemetry (a redaction-by-construction API); a small always-present
instrumentation cost to bound; interacts with the shared-service future
(per-owner metric isolation).

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme C — operability).
- 2026-09-12: proposed acceptance clarified by the intent-first sprint 10 audit; implementation remains proposed.
