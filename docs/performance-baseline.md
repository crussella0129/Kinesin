# Local measurements

The corrected release met the proposed 50 ms warm-run p95 target in both
1,000-sample repeats: **12.273 ms and 42.527 ms**. The earlier validated
checkpoint's ten-minute soak completed **10,152 runs** with bounded resources
and clean settlement. All measured tasks were synthetic and unchecked. Earlier
misses, remaining tail variation, and unmeasured service/load cases are retained
below rather than hidden behind these successful checkpoints.

The preserved first measurement below is a historical baseline. The measurement
driver now applies the configured retention/headroom policy, records executable,
configuration, lockfile, and SQLite versions, and joins its controller and storage
owners even when a measurement or output write fails. A new result must identify
its own executable; the first result does not validate later code.

Run the optimized driver from the repository root, choosing a fresh output
directory beneath the ignored `validation-output` directory:

```powershell
cargo build --release --example measure
.\target\release\examples\measure.exe validation-output/measure-warm --warm-only
.\target\release\examples\measure.exe validation-output/measure-full --soak-seconds 600
.\target\release\examples\measure.exe validation-output/measure-curves --curves-only
```

The default is 1,000 recorded warm trials after twenty warmups. `--warm-only`
skips cancellation, fairness, and soak. The full command also makes 100 bounded
HTTP cancellation observations, exercises twenty two-owner quota cycles, and
runs the explicitly requested soak. `--warm-trials` and `--cancel-trials` permit
smaller smoke runs; their allowed maxima are 1,000 and 100. Existing output
directories are rejected, preserving prior samples and avoiding old database
contents changing the workload.

`--curves-only` is a separate bounded experiment. With no controller queue and
two shared fake backend slots, it runs twenty batches at each active-run cap
1, 2, 4, and 8, always using the same 100 ms scripted delay. A second scenario
keeps four active slots and the same controller alive through ten batches each
at 100 ms, 400 ms, and 100 ms. It records all offered/admitted/rejected outcomes,
each terminal-result latency, actual peak active counts, and final settlement.
Failure to reach the intended concurrency fails the measurement. These are
420 unchecked synthetic runs; their latency includes injected positive delay,
and their small-sample percentiles are descriptive. They do not measure an
open-loop authenticated service or additional slots on a live model server.

These experiments call the library runner/controller directly with synthetic
inputs. They do not time configuration parsing, private-state startup checks,
authenticated service ingress, TLS, or a live model. The freeform receipt remains
`unchecked`; this latency distribution does not measure a file-evidence checker
or establish task correctness. Those behaviors have separate functional tests.
The scripted candidate is nineteen ASCII bytes; this is a small-payload
baseline, not a worst-case byte-cap measurement.
Process memory includes the driver's bounded buffer of at most 1,000 authority
snapshots retained for post-sample journal queries; it is not application-only
allocation accounting.

`samples.csv` includes total and durable admission latency. `breakdown.csv` is
collected from the real journal **after** all timed warm trials. Journal wait
includes event reservation through acknowledgement before the terminal event;
it excludes admission and the terminal write. Each event wait is truncated to
whole milliseconds before accumulation. Inbox wait and SQLite acknowledgement
are not yet separately instrumented, so this profile cannot distinguish them.
The terminal tail is an upper
estimate covering final commit, result retrieval, and up to one millisecond of
timestamp rounding. Model durations are integer milliseconds. These are nested
observations, not independent spans to add together or exclusive CPU costs.

The cancellation peer binds an ephemeral loopback port, accepts one complete
HTTP request, sends response headers, then withholds the response body. Each
sample records cancellation-call latency, peer EOF/reset latency, and latency to
the committed cancelled receipt. The driver checks that no additional model or
tool effect was planned. Peer closure measures this HTTP client's transport
cleanup; it cannot establish when a separate model server stops computing.
`unobserved` is retained when peer closure is not seen within five seconds.
Committed terminal latency is an upper bound on cancellation observation, so a
slow terminal write alone cannot diagnose slow cancellation observation.

The two-owner experiment fills Alice's two active and four queued slots, rejects
one excess Alice submission, and completes Bob's short run while Alice's runs
settle. The soak offers bursts of 25 and waits for each burst before continuing.
Both are closed-loop checks. The separate [fixed-arrival service experiment](service-load.md)
records actual HTTP arrivals, generator omissions, overload and settlement.
None of these small workloads measures fairness across a large population.

## Corrected zero-delay provider checkpoint

Checkpoint `17ecbc36fcdc9946c907d4807b2e9d8d19c933ed` makes the scripted
provider return immediately when its requested delay is zero. Positive scripted
delays and the real HTTP adapter retain their behavior; durable writes and
acceptance rules were not reduced. The new immutable release driver has SHA-256
`dcb52f14666dcaead6e5cf2b74894c0b694460431c82eac1d957d7144448a0ac`.
Two fresh-database repeats each excluded twenty warmups and recorded 1,000 runs:

| Corrected warm repeat | p50 | p95 | p99 | Maximum |
|---|---:|---:|---:|---:|
| A | 10.457 ms | 12.273 ms | 16.845 ms | 17.808 ms |
| B | 10.600 ms | 42.527 ms | 74.484 ms | 113.306 ms |

**Both measured p95 values meet the 50 ms target.** All 2,000 recorded runs
completed unchecked, and both processes returned storage reservations to zero.
The fake-model exchange duration rounded to zero milliseconds in every sample;
this means below the retained timestamp resolution, not zero work. Repeat B
still had admission p99 33.732 ms and terminal-tail p99 33.977 ms. Removing the
fake timer did not eliminate tail variation or explain every storage/scheduling
delay. Two runs on one workstation do not establish a deployment-wide latency
guarantee. Both complete distributions are preserved in
[repeat A](evidence/performance/checkpoint-17ecbc3/warm-a/summary.json) and
[repeat B](evidence/performance/checkpoint-17ecbc3/warm-b/summary.json).

The same executable then completed the separate fixed-batch concurrency curve.
All 300 offered runs were admitted and completed unchecked, no submission was
rejected, and each intended active cap was reached. The controller queue cap
was zero and the shared fake backend capacity stayed at two slots:

| Active cap | Batches / runs | Completion throughput | Per-run p95 including 100 ms delay |
|---|---:|---:|---:|
| 1 | 20 / 20 | 8.019 runs/s | 125.984 ms |
| 2 | 20 / 40 | 14.477 runs/s | 159.038 ms |
| 4 | 20 / 80 | 15.072 runs/s | 269.393 ms |
| 8 | 20 / 160 | 16.150 runs/s | 510.544 ms |

With two backend slots, increasing the active cap beyond two mostly adds
waiting in these batches. The modest throughput increase accompanies longer
per-run latency; the curve does not support an eightfold scaling claim.

The controlled slowdown kept four active slots and the same controller/resources
alive, offering forty runs per phase. Requested fake delay changed from
100 ms to 400 ms and back to 100 ms. All 120 runs completed unchecked without
rejection or error:

| Phase | Completion throughput | Per-run p95 |
|---|---:|---:|
| Initial 100 ms delay | 15.829 runs/s | 262.526 ms |
| Slower 400 ms delay | 4.613 runs/s | 875.347 ms |
| Restored 100 ms delay | 15.742 runs/s | 263.544 ms |

Every phase returned active/queued counts and queued bytes to zero, every
controller joined without runner errors, and the writer ended with 420 rows and
zero storage reservations. This approximately 35-second curve process peaked
at 11.996 MiB working set and 5.035 MiB private bytes. Its
[complete evidence](evidence/performance/checkpoint-17ecbc3/curves/summary.json)
records all offered/admitted/rejected outcomes and the descriptive timing
samples. Fixed-arrival authenticated-service load has its own
[bounded experiment](service-load.md); worst-case payload performance and a
live model capacity curve remain unmeasured. This is not completion of every
performance exercise in guide step 29.

No agent builds, tests, or model calls overlapped these measurements. Unrelated
applications remained. Full system-wide snapshots, process IDs and launch
timestamps are retained only in ignored local evidence; published samples keep
workload durations, memory counters and reproducible hashes. The ten-minute soak
below was not rerun after the zero-delay fake and separate SSE corrections:
it exercises the unchanged positive-100-ms fake, controller, and writer paths.
Its executable and checkpoint remain explicitly distinct.

## Measurements at the first validated implementation checkpoint

Checkpoint `e346207bd2ebedec061caa76fd07aaacc3c5c486` passed the clean-clone
checks. Its copied release measurement executable has SHA-256
`acef5c0d5372cedfab73340b3689f527a4c2a12daecb91cd810b781d848c0031`.
It used SQLite 3.53.2, WAL/FULL, metadata capture, and the configured 4 GiB,
100,000-run, minimum-24-hour retention policy. Two successive runs each recorded
1,000 warm samples after twenty warmups, on separate new databases:

| Warm measurement | p50 | p95 | p99 | 50 ms p95 target |
|---|---:|---:|---:|---|
| Warm-only process | 15.479 ms | 20.226 ms | 32.058 ms | Met in this sample |
| Warm segment before the full soak | 29.911 ms | 60.365 ms | 79.233 ms | Missed |

All 2,000 recorded runs completed with unchecked acceptance. The target was
therefore **not consistently met**. The second profile's p95 admission latency
was 12.605 ms, pre-terminal journal wait 37 ms, and terminal-tail upper estimate
8.837 ms. Its slowest complete run took 124.457 ms. These overlapping percentile
summaries cannot be added to reconstruct a percentile of total latency.

The scripted provider still awaited `sleep(Duration::ZERO)` in this executable.
Its recorded model-exchange p95 was 5 ms in the first process and 15 ms in the
second. Tokio documents millisecond timers and potentially coarser resolution
on Windows. [Tokio sleep](https://docs.rs/tokio/1.53.1/tokio/time/fn.sleep.html)
Removing a timer from the zero-delay fake is a separate measurement correction
to test; it does not establish the cause of journal/admission jitter, and it
must not erase these results or alter the actual HTTP provider.

The full process also settled all 100 cancellations. Cancellation-call p95 was
0.004 ms, socket-close p95 0.094 ms, and committed cancelled-receipt p95
36.081 ms. All peer closures were observed, all receipts remained unchecked,
and no additional model/tool effect was planned. Thus even the measured upper
bound on cancellation observation was below the 100 ms target in this sample.

Twenty owner-quota cycles admitted 120 Alice runs and twenty Bob runs while
rejecting twenty excess Alice submissions. Owner-rejection p95 was 0.003 ms;
Bob's completion p95 was 189.248 ms while sharing the two backend slots with
100 ms synthetic work. This records progress and quota enforcement for this
small workload, without a broader fairness or service-latency guarantee.

The 600.463-second overload soak offered 10,575 submissions in 423 bursts,
completed 10,152 runs, and immediately rejected the other 423. Overload-rejection
p95 was 0.003 ms. Peaks were eight active runs, sixteen queued runs, and 26,480
queued input bytes. Shutdown joined every runner and the writer, reported zero
runner errors, and returned active/queued counts, queued bytes, and storage
reservations to zero. This soak used 100 ms scripted delays, so its workload is
separate from the zero-delay measurement issue above.

Across 635 roughly one-second process samples, peak working set was 14.949 MiB
and peak private bytes 7.063 MiB; final working set was 14.770 MiB. The minute
bins show a plateau rather than accumulating retained work. The final database
held 11,412 rows, including warmups and the cancellation/fairness experiments;
its allocated live pages occupied 38,023,168 bytes and the WAL 4,165,352 bytes
at the final health query. These are observed storage figures, not a disk-size
guarantee under different payloads or retention intervals. The complete
[full-process evidence](evidence/performance/checkpoint-e346207/full/summary.json)
includes every sample, the ten-minute memory series, outcomes, and settlement
checks. Percentiles use nearest rank, `ceil(p * n)`, without interpolation.

All agent model work and local builds/tests were paused during these runs.
Before launch, aggregate CPU samples were 4.4–8.7%, available RAM about
19.2 GiB, and GPU utilization 13%. Unrelated user applications remained; this
is not an idle-machine claim. Raw samples, source hashes, and host context are
preserved in [checkpoint evidence](evidence/performance/checkpoint-e346207/warm/summary.json).

## First baseline, before retention/headroom integration

The first optimized synthetic run measured p95 **76.612 ms** for 1,000 warm
no-queue, one-turn freeform runs. This misses the guide's proposed 50 ms target.
Each sample includes admission, the actual SQLite journal and terminal receipt,
and a scripted answer with no injected delay. Capture is metadata; acceptance
is unchecked. Twenty warmups were excluded. p50 was 33.255 ms; p99 was 107.296 ms.

The following ten-minute overload soak completed 8,496 runs in 354 cycles.
Each cycle submitted 25 items against eight active and sixteen queued slots;
the fake model held each of two backend slots for 100 ms. All 354 excess
submissions were rejected, with p95 admission latency 0.003 ms. Observed maxima
were eight active, sixteen queued, and 24,736 queued input bytes. Shutdown
returned all reservations and reported zero runner errors.

Peak process working set was 17.461 MiB and peak private bytes 6.063 MiB.
These are process measurements sampled about once per second, not machine-wide
memory or GPU usage. Retained run payloads remain subject to separate disk policy.

This is a baseline with known background load: live GPU model evaluation and
other development/test compilation overlapped portions of the measurement.
Do not attribute the missed latency target to a particular mechanism without
a quieter repeat/profile. No production load or multi-host claim follows.

The driver is [examples/measure.rs](../examples/measure.rs). Raw samples and
memory observations are preserved in [evidence](evidence/performance/summary.json).
The initially measured executable SHA-256 was
`d6357f55a3149d1065bb08a06e0ec41797888f5175cf60a1b9795f68b0caf917`.
It ran on native Windows x86_64 with Rust 1.96.0 in release mode. This checkpoint
preceded the later retention/headroom and authenticated-service additions;
subsequent results must identify their own executable and settings.

The proposed target remains in force. Do not remove durable writes, lower
acceptance checks, discard failures, or relabel unchecked runs to improve it.

The current successful one-turn path commits admission, run start, model intent,
model observation, and terminal receipt separately. WAL with `FULL` synchronizes
each transaction, and automatic checkpoints can make occasional commits slower.
[SQLite's performance discussion](https://sqlite.org/wal.html#performance_considerations)
explains these costs. If future repetitions miss the target, first compare
admission, pre-terminal journal wait, and terminal tail. Returning the committed
record directly from the terminal command could remove a redundant query and
async round trip without changing durability. That is a proposed optimization,
not part of the measured driver. Group commit could amortize synchronization
across concurrent runs; it does not remove a single run's dependent commit
barriers and introduces its own batching delay.
