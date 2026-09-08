# Local measurements

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
```

The default is 1,000 recorded warm trials after twenty warmups. `--warm-only`
skips cancellation, fairness, and soak. The full command also makes 100 bounded
HTTP cancellation observations, exercises twenty two-owner quota cycles, and
runs the explicitly requested soak. `--warm-trials` and `--cancel-trials` permit
smaller smoke runs; their allowed maxima are 1,000 and 100. Existing output
directories are rejected, preserving prior samples and avoiding old database
contents changing the workload.

These experiments call the library runner/controller directly with synthetic
inputs. They do not time configuration parsing, private-state startup checks,
authenticated service ingress, TLS, or a live model. The freeform receipt remains
`unchecked`; this latency distribution does not measure a file-evidence checker
or establish task correctness. Those behaviors have separate functional tests.
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
Both are closed-loop checks. Neither replaces fixed-arrival service-load testing
or measures fairness across a large population.

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
explains these costs. If the quiet repeat still misses the target, first compare
admission, pre-terminal journal wait, and terminal tail. Returning the committed
record directly from the terminal command could remove a redundant query and
async round trip without changing durability. That is a proposed optimization,
not part of the measured driver. Group commit could amortize synchronization
across concurrent runs; it does not remove a single run's dependent commit
barriers and introduces its own batching delay.
