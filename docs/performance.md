# Performance and scaling

Kinesin's first complete release should run several bounded sessions, stream
model output, and remain responsive when an operation is slow. Its next release
can expose those sessions through a shared service. The implementation sequence
is in the [build guide](build-guide.md); this document defines the resource and
measurement contract.

**Every number below is a proposed starting point, not a measured result.**
There is no implementation or benchmark yet. Minimality means a small system
whose resource use and failure behavior you can explain.

## One owner per session, bounded work across sessions

Keep one synchronous state machine per run. Its async runner owns history,
pending calls, limits, and event order. Model calls and tools execute serially
within that run. Different runs can wait on I/O concurrently in one Tokio
runtime. A controller owns the admitted runners and joins their tasks.

Use these shared limits as the canonical initial settings. Per-run and HTTP
limits are defined in [configuration](configuration.md).

| Setting | Proposed value | What it bounds |
|---------|----------------|----------------|
| `max_active_runs` | 8 | Admitted runners, including model/tool waits, acceptance checking, and finalization |
| `max_queued_runs` | 16 | Accepted inputs waiting for a runner; use 0 in the first admission exercise |
| `max_queued_input_bytes` | 1,048,576 (1 MiB) | Combined owned input bytes in that queue |
| `max_inflight_model_requests` | 2 | Local model exchanges; cap at independently verified server capacity |
| `max_blocking_tools` | 4 | Filesystem tool jobs, including jobs still finishing after cancellation |
| `journal_queue_events` | 64 | Outstanding event commands, including the command executing on the database writer |
| `journal_queue_bytes` | 8,388,608 (8 MiB) | Combined serialized bytes of outstanding event commands, including the executing command |
| `journal_admission_timeout_s` | 5 | Maximum wait to enter the writer inbox, constrained by the applicable execution or settlement budget |
| `settlement_grace_s` | 5 | One additional bounded wait for stop-related bookkeeping and owned-work settlement; never additional execution time |
| `observer_queue_events` | 128 | Pending display events per observer, once event streaming exists |
| `observer_queue_bytes` | 262,144 (256 KiB) | Pending display bytes per observer |
| `max_observers_per_run` | 2 | Concurrent display subscriptions to a run |
| `max_observers_global` | 16 | Concurrent display subscriptions across active and retained runs |
| `per_owner_active_runs` | 2 | Active runs for one authenticated owner in shared-service mode |
| `per_owner_queued_runs` | 4 | Queued runs for that owner, still inside the global queue limits |

Queue limits count retained payloads, not merely channel slots. An event command
contains one event and the corresponding projection change; both byte sizes
count. A command larger than the inbox byte cap is rejected before waiting.
Bound data before it enters a queue. For journal commands, hold both count and
byte reservations through execution until completion or definitive rejection;
dequeue and caller cancellation do not release them. Input-queue reservations
instead transfer to the admitted runner's bounded ownership at dequeue. Neither
path may leave a retained payload outside its applicable resource accounting.

The first controller rejects excess runs immediately. Add the bounded queue only
after admission and release behavior work. Do not load an arbitrarily large job
file before checking its size, spawn a task for every submitted job, or collect
all completed results indefinitely. Consume a bounded input batch and drain
completed task handles as runs finish. An overflow has an explicit outcome;
it never silently disappears.

Tokio's bounded channels provide backpressure, but an unlimited number of blocked
senders can still retain unlimited inputs. Likewise, a semaphore bounds permit
holders, not arbitrary tasks waiting for permits. Admit before spawning. With
eight active runners and at most one pending model request per runner, the
internal model-wait population is also bounded.
[Tokio channels](https://tokio.rs/tokio/tutorial/channels),
[Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html),
[JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html)

## Keep permits attached to the actual work

Acquire a model permit for an entire exchange, including response-body
consumption. Keep it through protocol validation and settlement of the corresponding
journal outcome so a saturated writer cannot turn completed responses into a
second unbounded queue. Release it before tools or the task-acceptance checker
run. Keep active-run ownership until finalization settles. HTTP response headers
alone do not mean a streamed model request has completed.

Acquire blocking-tool capacity before submitting filesystem work. Move that
permit into the blocking job, so abandoning its async waiter cannot falsely
return capacity while the job still runs. No synchronous filesystem operation or
SQLite call belongs on a Tokio worker. Use bounded blocking jobs for short
filesystem work and a dedicated thread for the long-lived database writer.
Tokio documents both blocking-pool limits and the fact that started
`spawn_blocking` jobs cannot be aborted.
[Tokio blocking work](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

The controller owns its task set. Normal shutdown stops admission, cancels
runners cooperatively, lets them settle outstanding work, and joins them. A
forced abort can leave an incomplete run; it is never reported as a successful
terminal event. Aborting tasks also requires draining their join results.
[Graceful shutdown](https://tokio.rs/tokio/topics/shutdown),
[JoinSet cancellation](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html#method.abort_all)

## Bound task acceptance inside the run

[Verification](verification.md) defines the authoritative checker and its fixed
bounds: 1–4 criteria, an 8 KiB serialized specification, an 8 KiB checked final
candidate, 64 KiB retained evidence per active run, and an 8 KiB acceptance
receipt. Evidence-record count also stays within the run's admitted tool-call
limit. These bounds supplement the existing history, tool-result, request, and
journal limits; they do not replace them.

Count evidence memory for as long as the checker retains it, even if conversation
history releases the corresponding entry. Immutable bytes may be shared, but
moving a reference between inventories must not evade accounting. Evidence
overflow stops the run with `verification_evidence_limit`; do not discard
required observations to manufacture a later pass.

The first checker is a pure inline operation over the frozen specification,
candidate, and runner-owned observations. It scans each distinct bounded source
once and performs at most the configured field checks. It has no queue, model
call, new file read, arbitrary regex, or spawned worker. Active-run admission
therefore bounds simultaneous retained checker work. Measure its duration;
wrapping synchronous code in a timeout cannot preempt it. An expensive future
checker needs a new resource design.

Release the model permit after its observation acknowledgement and before
checking; retain the active-run reservation through receipt finalization.
Assessment consumes remaining execution time, with cancellation/deadline checks
before and after it. The settlement grace can record a known outcome, but cannot
launch or repeat a checker. Failed or inconclusive acceptance triggers no
automatic repair generation or verification retry.

## Deadline and cancellation behavior

Start one monotonic deadline when a bounded submission is accepted. Include
admission queueing, model-permit waits, tools, HTTP bodies, acceptance checking,
and journal waits.
Never reset the run clock for each model turn. The model queue additionally has
the shorter `model_queue_timeout_s` bound. After obtaining capacity or a journal
acknowledgement, recheck cancellation and remaining time before dispatch.

The execution deadline limits permission to perform work; it must not prevent
recording why work stopped. On the first cancellation, deadline expiry, or other
stop decision, start one `settlement_grace_s` interval. During that interval,
allow only bounded bookkeeping: settle an already submitted command or operation,
record its outcome or a known-unsent effect, and commit the terminal state once
owned work is settled. No model/tool dispatch, checker launch, or retry is
authorized by this grace. Do not restart the interval for each event or cleanup
attempt.

Ordinary journal admission uses the smaller of its admission timeout and the
remaining execution budget. Stop-related journal admission instead uses the
smaller of its admission timeout and the remaining settlement grace. Preserve
sequence and acknowledgement order in both cases; never submit a replacement
for a database command whose commit outcome is still unknown.

Acceptance of the terminal command into the storage inbox is the cancellation
arbitration boundary. Before that point, apply observed cancellation/expiry to
the terminal decision and a checked task's inconclusive receipt. After that
point, settle the submitted command; a late cancellation/deadline cannot recall
it or overwrite its committed result. The settlement grace does not reopen this
decision. [Verification](verification.md) defines the single finalization path.

The grace bounds how long the stopping path waits, not how long a kernel call
or SQLite commit can run. If it expires, report unresolved settlement rather
than a committed terminal outcome. The controller retains the run, outstanding
handles, and their resource reservations until actual completion, and may then
finish bookkeeping. Retained unresolved runs still count toward active capacity;
do not accept replacements that bypass that bound. Controller shutdown may
report incomplete cleanup and leave recovery to the next start. Neither a
timeout nor process exit proves that remote inference stopped.

Consequently, task latency can exceed the execution budget during settlement.
Measure settlement time separately. A hard upper bound on termination is not
promised for in-process blocking work or unresponsive storage.

For HTTP, constrain the whole exchange by the smaller of the configured request
timeout and remaining run time. A read-stall timer catches a stalled connection;
a total deadline catches a peer that sends tiny chunks indefinitely. Streaming
does not remove either deadline.
[reqwest timeouts](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.timeout)

Cancellation stops new effects and abandons cancellable network waits. It is
not evidence that remote inference has stopped. Nor can it instantly stop a
filesystem operation or an already submitted SQLite transaction. Retain and
settle their acknowledgements according to [the journal contract](traces.md).
Never retry a journal write merely because its waiter timed out; commit may have
succeeded. Record uncertainty if clean settlement becomes impossible.

Give each runner a child cancellation token so cancelling one run does not cancel
its siblings. Audit every `select!` boundary for what happens when a losing
future is dropped. In particular, do not select away a partially completed
operation and restart it as though nothing happened.
[CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html),
[select cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety)

## Streaming without accumulating work

The adapter owns bounded incremental parsing. The runner receives a complete
normalized model observation before any tool is dispatched. Text deltas may
reach an observer earlier, but are provisional and cannot authorize effects.
Protocol details and malformed-stream tests are in [integration](integration.md).

Reserve global and per-run observer capacity with nonblocking admission before
attaching a subscriber. Hold both reservations for the stream's full lifetime,
including catch-up on a retained run; release them on every close/error path.
The active-run limit cannot bound subscriptions to completed runs. A terminal
event contains both execution outcome and acceptance, following their atomic
result/receipt commit. An event stream closes only after delivering that
authorized terminal catch-up, not when model generation or checking finishes.

Observers have their own bounded event/byte queues. Disconnect a lagging observer
with an explicit lag indication; do not hold inference open indefinitely while a
terminal or network subscriber drains. An observer is optional. The journal is
mandatory and its acknowledgement remains on the effect path. In service mode,
reconnection can retrieve owner-authorized run status and available journal
events; it must not claim to recover unsaved provisional text.

Separate the model exchange permit from an observer's connection. A subscriber
disconnect does not automatically mean a background run was cancelled. Define
that API behavior once in [architecture](architecture.md), then test it.

## Storage cost is part of latency

The initial journal uses one `rusqlite` connection on a dedicated writer thread,
SQLite WAL mode, and `synchronous=FULL`. An event append and its run projection
update commit in one transaction. Only a successful commit acknowledges the
command. Query the effective settings; a misspelled SQLite pragma can be ignored.
WAL allows readers alongside a writer, but retains a single-writer constraint.
[SQLite WAL](https://www.sqlite.org/wal.html),
[SQLite synchronous settings](https://www.sqlite.org/pragma.html#pragma_synchronous)

Measure inbox wait and transaction acknowledgement separately. Use metadata
recording by default. Opt-in replay recording stores normalized conversation
deltas once, rather than copying the complete growing conversation on every
turn. The exact outgoing HTTP body still exists as one bounded prepared value;
its transmission does not require retaining another full copy in storage.

Finalization writes the bounded candidate/result, fingerprint, acceptance
receipt/projection, and terminal event together. Count all of those bytes in the
outstanding journal command. Metadata capture retains the sensitive final
candidate and receipt, including rejected or unchecked candidates; it does not
add durable copies of every evidence body. There is no later unbounded queue of
verdicts waiting to amend already terminal results.

Start with one transaction per event command. If measurements identify commit
latency as a bottleneck, experiment with bounded group commit: a small batch of
independent commands in one transaction, preserving each run's sequence. Every
acknowledgement still follows successful commit with the same durability
settings. Include the batching delay in the measured latency. Do not silently
change `FULL` to `NORMAL` or acknowledge on enqueue to reach a target.

Bound retention and monitor database/WAL size in addition to RAM. Long read
transactions and checkpoints can affect storage behavior and latency; avoid a
live subscriber holding a database transaction open. Keep query pages bounded.
Storage failure stops further effects. [Traces](traces.md) owns the persistence,
retention, crash, and replay details.

## Measure the experience and its causes

Use monotonic timestamps for durations and wall-clock timestamps for human
correlation. Keep run identifiers in logs and traces, not high-cardinality metric
labels. A timing recorded at the HTTP client cannot separate network time from
server queueing without compatible server measurements.

| Metric | Definition |
|--------|------------|
| Admission wait | Accepted submission to runner admission |
| Model queue wait | Model permit requested to granted |
| First response byte | HTTP dispatch to first body byte |
| First useful text | HTTP dispatch to first nonempty displayable text delta; absent for a tools-only reply |
| Tool readiness | HTTP dispatch to complete, validated tool-call batch |
| Model exchange | HTTP dispatch to complete normalized reply or classified error |
| Journal inbox wait | Event ready to accepted by the writer inbox |
| Journal acknowledgement | Inbox acceptance to committed acknowledgement |
| Tool duration | Handler start to actual handler completion |
| Assessment duration | Pure checker start to completion, including parsing and criterion checks |
| Task latency | Accepted submission to committed terminal outcome and acceptance receipt |
| Cancellation response | Cancellation signal to runner observing it and stopping new dispatch |
| Execution-completion throughput | Runs reaching `completed` per elapsed minute, regardless of acceptance |
| Contract-pass throughput | Runs reaching `completed` with acceptance `passed` per elapsed minute |
| Live-evaluation goal success | Independently judged goal successes over evaluated tasks, with grader/version and denominator |

Report p50 and p95 with sample counts, offered/accepted/rejected work, outcomes,
invalid calls, repeat stops, model turns, token usage, active counts, queue bytes,
peak resident memory, and database/WAL growth. Missing token usage is unknown,
not zero. An HTTP success is not a task success. A streamed chunk may contain
several tokens, no tokens, or a partial tool argument; do not label chunk timing
as precise token timing.

Keep all three outcome measures separate. `unchecked`, `failed`, `pending`, and
`inconclusive` acceptance never enter the contract-pass numerator. An external
evaluation can assess broader goal quality, but cannot silently upgrade the
stored receipt. Record contract identity/version and its scope alongside passed
counts; passing a narrow extraction contract is not a general correctness claim.

Do not add durations from different concurrent runs and call the sum wall-clock
latency. Within a run, use nonoverlapping spans or show their overlap. Keep
latency distributions for completed runs, contract-passed runs, and failed or
inconclusive outcomes separate. Report every outcome's count so dropping slow
failures cannot make the result look better.

## Initial targets and benchmark procedure

These targets express desired behavior for the implementation. They are not
promises about a model, machine, or filesystem.

| Experiment | Proposed acceptance target |
|------------|----------------------------|
| Warm, one-turn fake provider; at most 64 KiB payload; 1,000 trials | p95 at most 50 ms for the complete no-queue run, including selected checker and journal commits; record contract/capture profiles and exclude intentionally injected fake delay |
| Cancel while waiting on async HTTP | p95 at most 100 ms to observe cancellation and stop new dispatch; measure actual cleanup separately |
| Submit bounded valid input while admission is full | p95 at most 50 ms to return the documented overload result |
| Ten-minute repeated-overload soak | Active/queue limits never exceeded; retained work and RSS reach a plateau; report peak memory and investigate upward trends |
| Real model | Establish a fixed baseline before choosing first-text, tool-readiness, or task-latency targets |

Run a release build on a named machine with background load recorded. Warm up
separately, preserve raw samples, and compare repeated runs. Treat a missed target
as a profiling question, not permission to weaken policy or hide journal costs.

1. Verify limits with scripted providers and controlled slow tools/writer jobs.
   Prove two independent runs overlap without letting either exceed its limits.
2. Measure one, two, four, then eight active runs with a fixed fake workload.
   Keep model-inflight capacity explicit. Introduce cancellation, rejection,
   malformed replies, failed/inconclusive checks, and slow observers while other
   runs continue. Saturate evidence bounds and verify that checking holds no
   model permit and creates no hidden worker or retry queue.
3. Use the live [evaluation tasks](testing.md) with one verified model/server
   profile. Compare model-inflight limits 1 and 2, then 4 only if server capacity
   and memory have been verified. Include a mixture of short and long prompts.
4. Compare execution-completion throughput, contract-pass throughput, independent
   goal evaluation, and p95 latency separately. More batching may improve raw
   throughput while delaying a short task behind a long one.
   Preserve model, GGUF checksum, template, sampling, context, server commit,
   launch flags, hardware, and cache state with each result.
5. For the service, drive fixed arrival rates as well as fixed concurrency.
   A test that waits for each response before submitting another request reduces
   offered load when the system slows down, potentially hiding overload.
   Report dropped load-generator iterations too.
   [Open and closed load models](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/open-vs-closed/)

## Experiments after the baseline

Before introducing another execution mechanism, use the experiments in the
[paper review](paper-review.md#optional-optimization-experiments). Context
selection, overlapping independent tool operations, and speculative decoding
optimize different parts of the critical path. A reported decoding speedup is
not an end-to-end agent-task or shared-service speedup.

Keep serial tools as the baseline. Consider bounded parallel calls only when
measured tool latency warrants it and the complete authorized batch contains
independent operations. Recheck call/result ordering, cancellation, journal
ownership, and shared limits. No DAG framework or partial-call execution is
required by the current release.

Backend speculation is conditional on a verified implementation supporting the
chosen target. Compare complete tool-readiness and task latency under the actual
concurrent load, as well as goal correctness. Include every GPU, sidecar, queue,
and cache in capacity/cost accounting. Measure overlapping paths without adding
their durations, and distinguish target completion from total resource settlement.
Greedy batch-one results on split H100s do not predict a local quantized backend's
p95 under load. See [OoO-Spec v1](https://arxiv.org/abs/2608.00814v1).

## Scaling to a shared service

The first service has one controller and a local SQLite database. Add Axum only
after the concurrent CLI passes its gates. Require operator-provisioned bearer
credentials over TLS, owner-scoped run access, bounded HTTP ingress, and the
per-owner limits above. Authentication and tool capability rules are specified
in [security](security.md).

Use bounded queues per authenticated owner, serviced round-robin at both run
admission and model dispatch. Each owner stays inside the global count/byte
limits. Expire queued work before dispatch. This gives each ready owner a turn;
it does not promise equal token throughput or preempt a long model request.
Keep individual request token/time budgets to limit that imbalance.

A FIFO semaphore is useful in the personal CLI. It is not a complete multi-user
scheduler: its fairness is request order, and a large `acquire_many` request can
block smaller ones behind it. Avoid using weighted permit acquisition as a
shortcut for token-fair scheduling.
[Semaphore fairness](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html)

Tower supplies useful service limits and load shedding, but middleware order is
observable behavior. Its documented `buffer(100)` outside
`concurrency_limit(10)` permits 110 outstanding requests. Test the exact stack,
including body limits and error mapping. A middleware future that returns a
streaming response can finish before that response body; application run/model
permits must still cover their full lifetimes.
[Tower layer order](https://docs.rs/tower/latest/tower/struct.ServiceBuilder.html#order),
[Tower load shedding](https://docs.rs/tower/latest/tower/load_shed/index.html)

Several model endpoints can eventually sit behind this single controller, each
with its own capacity and compatibility profile. Running several harness
controllers against the same work requires a later ownership design with leases,
fencing, recovery, and effect reconciliation. Do not share a WAL database over a
network filesystem or infer safe distributed execution from SQLite transactions.
[SQLite WAL deployment restrictions](https://www.sqlite.org/wal.html)

General-purpose behavior comes from stable run, provider, and capability
contracts. It does not require model-spawned worker trees, a custom scheduler
framework, or a network hop between every named module.
