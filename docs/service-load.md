# Fixed-arrival service load experiment

This is a bounded synthetic experiment for build-guide step 29. It drives the
real authenticated HTTP ingress, controller, and SQLite writer through an
ephemeral loopback listener. A scripted model delays each answer by 100 ms;
one model slot serves two active runs and a queue of four. It does not measure
live inference quality, a dedicated service identity, TLS, or remote exposure.

Run it explicitly in a quiet window, with no other timing experiment or model
evaluation running. Normal `cargo test` leaves the experiment ignored. It uses
existing dependencies and creates no external service or long-lived process.

```powershell
$env:KINESIN_SERVICE_LOAD_OUTPUT = "validation-output/service-load.json"
cargo test --locked --test service_load -- --ignored --nocapture
Remove-Item Env:KINESIN_SERVICE_LOAD_OUTPUT
```

The output file must be new; existing evidence is never overwritten. If the
variable is omitted, the test chooses a unique filename under `validation-output`.
Record the tested commit alongside the JSON when retaining a baseline. The
report contains no token, prompt, or model-output content.

The two fresh scenarios schedule 40 arrivals each, at 5 and 100 requests per
second. The low rate is below the fake model's nominal ten answers per second;
the high rate is intended to saturate admission. These are configured arrival
rates, not claims that the generator successfully sent every scheduled request.

Each tick has an absolute deadline. The generator tolerates lateness shorter
than one arrival interval: 200 ms at 5/s and 10 ms at 100/s. If an entire
interval has elapsed, that tick is counted as `skipped_late_ticks`. If eight
HTTP requests already occupy the generator's owned task set, the tick counts
as `skipped_generator_capacity`. There is no waiting population beyond those
eight requests and no catch-up burst. All tick lags and actual send lags are
retained in microseconds, including timing jitter tolerated by the policy.

`offered_ticks` means scheduled arrivals. The report enforces:

```text
offered = dispatched + skipped_late + skipped_generator_capacity
dispatched = acknowledged_admissions + server_rejections
acknowledged_admissions = durable_admissions = completed_runs = model_requests
```

HTTP status counts distinguish server rejection from generator omissions. The
experiment records transport, body, and response-validation errors separately
and fails its proof if any occur. A timed-out request could already be durably
admitted; comparing HTTP acknowledgements with actual journal rows catches
that uncertainty instead of assuming the request did nothing.

Raw per-request latencies measure admission responses, including rejection;
they are not end-to-end task latencies. Separate elapsed times cover the request
window and the subsequent run drain. Active-run, queued-run, queued-byte, and
ingress peaks demonstrate bounded capacity. After all work settles, the test
joins ingress, controller, and writer and requires zero remaining run, queue,
observer, connection, and journal ownership. Completed freeform tasks remain
unchecked, so `passed_tasks` is always zero.

Compare repeated quiet runs before drawing latency conclusions. Small samples
and local timer resolution limit precision; the raw generator omissions and
lag samples are part of the result, not work to silently exclude.

## Recorded local run

The 2026-09-08 quiet run used the already-built test executable and completed in
9.07 seconds. [Raw JSON](evidence/performance/service-fixed-arrival-20260908.json)
contains all request samples, generator lag, counters, and cleanup outcomes.

Production source was `17ecbc36fcdc9946c907d4807b2e9d8d19c933ed` on `answer-key`.
The measurement test was new and uncommitted when compiled. Its precise
provenance is:

| Artifact | SHA-256 |
|----------|---------|
| `tests/service_load.rs` | `9E41E727051A629EA3028FDCAFE8290D6CFF2BB9E01B98FD3599D25D1F086592` |
| `target/debug/deps/service_load-7389078c50ee75bc.exe` | `A60A25C240DD0BAE7B4876B39535E8277D600330DA80C214D440615FED26FC4A` |

The Windows x86-64 executable used Cargo's test profile, unoptimized with debug
information. It was built by `cargo test --locked --test service_load` before the
quiet window; the experiment then invoked that executable with
`--ignored --nocapture`, so no compilation overlapped measurement. Formatting,
the ordinary ignored test invocation, and targeted Clippy passed beforehand.

| Scheduled rate | Offered | Sent | Late ticks skipped | Admitted and completed | Server rejected | Peak active / queued |
|----------------|---------|------|--------------------|------------------------|-----------------|----------------------|
| 5/s | 40 | 40 | 0 | 40 | 0 | 1 / 0 |
| 100/s | 40 | 30 | 10 | 9 | 21 | 2 / 4 |

Neither scenario dropped a tick because the generator's request capacity was
full. Both had zero response errors. The high-rate case retained at most 6,875
queued input bytes and reached the configured active and queued limits. All
admitted runs completed unchecked; the test makes no task-quality claim.

The low-rate request window ended at 7,805 ms and all runs settled by 7,929 ms.
The high-rate window ended at 398 ms and all runs settled by 1,017 ms. Both
scenarios finished with zero active runs, queued runs, queued input bytes,
connections, observers, and outstanding journal commands/bytes; ingress,
controller, and writer joins all succeeded.

Admission-response latency, using nearest-rank percentiles over the raw samples:

| Scheduled rate | HTTP outcome | Samples | p50 (ms) | p95 (ms) |
|----------------|--------------|---------|----------|----------|
| 5/s | 202 admission | 40 | 4.216 | 5.831 |
| 100/s | 202 admission | 9 | 6.452 | 12.249 |
| 100/s | 429 owner quota rejection | 21 | 0.840 | 1.058 |

These values cover HTTP creation responses, not model completion or accepted-task
throughput. With only nine high-rate admissions, its p95 is the maximum sample.

The 100/s case is a schedule the generator attempted. Its ten missed ticks are
part of the result, so this run does not establish delivery at 100 requests per
second or a sustained throughput capacity. It demonstrates explicit overload
accounting under a fixed schedule with bounded generator work.
