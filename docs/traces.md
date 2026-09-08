# Journal, results, and replay

## Use a transactional store

Use SQLite through `rusqlite` for the first persistent journal. This replaces
per-event files and a hand-maintained lookup/index. A database earns its dependency
by keeping an event and its run-status update atomic, supporting owner-scoped
queries, and giving the shared-service phase a usable storage boundary.

One dedicated OS thread owns the connection. The initial synchronous store can
be tested directly; asynchronous runners access it through a bounded command
channel and a one-shot acknowledgement. Never call blocking SQLite operations
on a Tokio runtime worker.

Use a private state directory containing `controller.lock`, `kinesin.sqlite`,
and any SQLite WAL/shared-memory sidecars. Before recovery, acquire and retain an
exclusive lock on the lock file. A second controller using the same state
directory must fail clearly. The lock file's existence alone proves nothing;
retain the locked handle for the controller lifetime. Current Rust has
`File::try_lock` (stable since 1.89).
[Rust file locking](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock)

## Proposed schema v2

This is the proposed second revision of the design, not a migration for an
existing implementation or database. Use parameterized statements. Enable foreign
keys on every connection. Set/check schema version at startup; future migrations
must be explicit transactions before accepting work. Execution and acceptance
have separate meanings defined in [verification](verification.md).

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    model_profile_id TEXT NOT NULL,
    task_mode TEXT NOT NULL CHECK (task_mode IN ('freeform', 'checked')),
    task_profile_id TEXT,
    task_profile_version TEXT,
    task_spec_sha256 TEXT,
    checker_id TEXT,
    checker_version TEXT,
    capture TEXT NOT NULL CHECK (capture IN ('metadata', 'replay')),
    phase TEXT NOT NULL CHECK (phase IN (
        'queued', 'running', 'cancelling', 'completed',
        'stopped', 'failed', 'cancelled', 'interrupted'
    )),
    acceptance_status TEXT NOT NULL CHECK (acceptance_status IN (
        'unchecked', 'pending', 'passed', 'failed', 'inconclusive'
    )),
    created_unix_ms INTEGER NOT NULL,
    policy_version TEXT NOT NULL,
    submission_sha256 TEXT NOT NULL,
    idempotency_key TEXT,
    terminal_reason TEXT,
    result_json TEXT,
    result_sha256 TEXT,
    acceptance_json TEXT,
    CHECK (
        (task_mode = 'freeform' AND acceptance_status = 'unchecked'
            AND task_profile_id IS NULL AND task_profile_version IS NULL
            AND task_spec_sha256 IS NULL
            AND checker_id IS NULL AND checker_version IS NULL)
        OR
        (task_mode = 'checked' AND acceptance_status != 'unchecked'
            AND task_profile_id IS NOT NULL AND task_profile_version IS NOT NULL
            AND task_spec_sha256 IS NOT NULL
            AND checker_id IS NOT NULL AND checker_version IS NOT NULL)
    ),
    CHECK (task_spec_sha256 IS NULL OR length(task_spec_sha256) = 64),
    CHECK (result_sha256 IS NULL OR length(result_sha256) = 64),
    CHECK (
        (result_json IS NULL AND result_sha256 IS NULL)
        OR (result_json IS NOT NULL AND result_sha256 IS NOT NULL)
    ),
    CHECK (
        phase IN ('queued', 'running', 'cancelling')
        OR (acceptance_status != 'pending' AND acceptance_json IS NOT NULL)
    ),
    CHECK (
        phase NOT IN ('queued', 'running', 'cancelling')
        OR acceptance_status IN ('unchecked', 'pending')
    ),
    CHECK (acceptance_status != 'passed' OR phase = 'completed'),
    CHECK (
        phase != 'completed'
        OR (result_json IS NOT NULL AND result_sha256 IS NOT NULL
            AND acceptance_json IS NOT NULL)
    ),
    UNIQUE (owner_id, idempotency_key)
);

CREATE TABLE events (
    run_id TEXT NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    seq INTEGER NOT NULL CHECK (seq >= 0),
    schema_version INTEGER NOT NULL,
    kind TEXT NOT NULL,
    elapsed_ms INTEGER NOT NULL CHECK (elapsed_ms >= 0),
    data_json TEXT NOT NULL,
    PRIMARY KEY (run_id, seq)
);

CREATE INDEX runs_owner_created
    ON runs(owner_id, created_unix_ms, run_id);

PRAGMA user_version = 2;
```

`runs` is the current projection: phase, ownership, frozen task identity,
acceptance status/receipt, and final result. `completed` replaces the earlier
name `succeeded`; it does not mean a task passed. Derive `task_accepted` only from
`phase = completed AND acceptance_status = passed`, never from a stored
client/model assertion. A checked run begins `pending`; freeform begins and
remains `unchecked`. All terminal phases require a receipt and prohibit pending
acceptance. A completed run additionally requires its candidate/result and digest;
a run stopped without a candidate may have a null result.

`events` is ordered history. Sequence starts at zero separately for every run.
The v2 typed event/receipt format carries the same outcome distinction. The
application enforces legal transitions, event order, receipt/projection agreement,
canonical digest encoding, and all serialized byte bounds; SQL constraints are
an additional check, not a verifier. UUID run IDs are opaque identifiers, not
access tokens.

For a checked task, freeze profile ID/version, checker ID/version, specification
digest, generated instruction, and criteria before admission. The digest covers
the versioned effective specification, including source-parser and output
semantics. Keep its identity in the projection and acceptance event; replay
capture additionally retains the exact frozen specification. Do not resolve the
task alias again to reinterpret an existing result.

A model turn counts generation attempts, not database events. A request can have
many events. Give every proposed external effect an `effect_id`, for example
`model-0` or `model-0/tool-0`. Preserve the provider's `call_id` separately;
correlate it with its originating model effect. Never use provider IDs as paths.

## Event vocabulary

| Kind | Purpose and required information |
|------|----------------------------------|
| `run_accepted` | Principal-derived owner, approved aliases, submission mode, frozen task/checker identities, required criterion IDs and specification digest, initial acceptance status, effective policy/limits, capture mode, versions, submission identity; initial inputs and exact frozen specification additionally in replay mode |
| `run_started` | Scheduler admission, queue duration, start observation |
| `model_planned` | Effect/turn ID, prepared-request fingerprint, request byte count, profile identity, input event references, effective sampling |
| `model_finished` | Same effect ID, sent/unsent status, normalized reply/error classification, counts/timings; actual normalized input additionally in replay mode |
| `tool_planned` | Effect/model/call IDs, tool name, validation/authorization decision; full arguments additionally in replay mode |
| `tool_finished` | Same IDs, executed/denied/unsent status, duration, bounded result classification; eligible evidence ID/resource identity/completeness/digest bound to this observation sequence; exact observation additionally in replay mode |
| `cancel_requested` | Authorized caller/reason and runner-visible cancellation observation |
| `run_finished` | Execution phase/reason, acceptance status and bounded receipt, counters, elapsed time, exact candidate digest when present; result and acceptance projection saved in the same transaction |
| `recovery_interrupted` | Startup marks unfinished work interrupted with an inconclusive checked receipt or unchecked freeform receipt; identify any pending effect as outcome unknown |

Within one run, only its runner assigns the next event sequence after acceptance.
Admission creates sequence zero; recovery takes over only while holding the
controller lock after the previous process is gone. Active-run events are never
appended independently by tool threads, model clients, or stream subscribers.

The cancellation API signals the owning runner, which records and handles the
request. It does not race a second writer against that runner's sequence. Queued
runs are owned by the scheduler and can terminate without starting a runner.
Neither cancellation nor an observer creates a separate acceptance verdict.

## Capture policy and final results

**Metadata** is the default. Keep identifiers, aliases, classifications, byte
counts, timing, policy versions, and outcomes. Do not retain prompts, intermediate
model messages, tool arguments/results, or raw provider bodies. Fingerprints and
filenames/identifiers can still disclose information; metadata is private too.

**Replay** additionally stores the exact effective instruction text, initial
prompt, tool definitions/versions, and normalized model/tool/error inputs that
changed the core. Record each conversation input once. Include any provider
continuation data needed by the supported adapter, under the same bounds.
For checked tasks include the frozen task specification and the actual evidence
observations needed by its checker. Evidence references reuse those observations;
do not add another durable copy of each body.

A final answer candidate is a bounded, owner-scoped result in `runs.result_json`,
committed with its acceptance receipt and terminal event. Retain rejected and
unchecked candidates as well as passed ones so an asynchronous client can inspect
the outcome after restart. The result envelope identifies the exact candidate
and, for a passed checked task, the harness-rendered verified fields. A terminal
run without a candidate still has a receipt explaining its outcome.

`result_sha256` and the receipt's candidate digest hash the exact candidate UTF-8
bytes after provider decoding, before trimming, parsing, JSON reserialization,
or rendering. They do not hash the serialized result envelope. Keep original
candidate bytes unchanged. A checked candidate above 8 KiB fails the output
contract before parsing; retained candidates remain under the existing bounded
model-response/result limits. Evidence digests cover exact observed tool-body
bytes, and a full-content claim requires a complete observation.

An acceptance receipt is bounded to 8 KiB and binds owner/run, status/reason,
contract/checker IDs and versions, specification digest, candidate digest when
present, required criterion outcomes, evidence IDs/sequences/digests, scope, and
assessment duration. Generate it through one typed path and validate agreement
with the projection before the terminal transaction. Missing candidate/evidence
is explicit, never replaced by a fabricated digest or a default pass.

Results and receipts may disclose sensitive file contents, resource identities,
or rejected output even in metadata mode. Apply the same authorization and
retention to every verdict. Public diagnostics use stable codes and sanitized
bounded messages, never hidden expected values, unrelated paths, raw exceptions,
or a backend error body. Metadata capture does not retain intermediate evidence
bodies; its receipt is a historical verdict, not enough data to recompute one.

Operational `tracing` output is a separate redacted channel. It contains no
prompt/result text, credential headers, raw environment, or unbounded metric
labels. SQLite does not encrypt records or implement application authorization;
the service and operating-system permissions do that work.

## Recording a prepared request

Keep source provenance with the recorded inputs: stable source ID, purpose,
origin/trust class, known revision or content fingerprint, and byte count in a
deterministic order. Initially this covers configured instructions and user input;
tool observations reference their originating call and the bytes actually seen.
Store these bounded fields in the existing event payloads; no additional database
table or copy of the conversation is required.

Metadata capture retains only permitted source descriptors/hashes/counts; these
remain private. Replay capture retains the exact selected input bytes once.
Do not promote a workspace file to trusted instructions because of its name or
location. Do not reread today's source files to recreate yesterday's request.
Provenance means a source was supplied; it does not prove the model relied on it
or that it caused a particular sentence.

A human-edited export may be explicitly submitted as input to a new authorized
run. It does not alter the original events, inherit another owner's authority,
or resume an interrupted effect. Apply normal capture and input-size limits.

Prepare provider bytes without I/O. Serialize the typed request deterministically
for the adapter version and compute SHA-256 with a library. The journal records
that fingerprint, byte count, profile/version, and references to the inputs used.
The HTTP client sends those same prepared bytes after the intent commits.

In replay mode, initial inputs plus recorded deltas allow the adapter to rebuild
the body and compare its fingerprint. Metadata mode cannot do exact replay.
Do not reconstruct from today's files or quietly substitute missing text.

Do not store each growing full request as another snapshot. Across many turns,
that repeatedly writes the same history. Do not store every streaming delta
durably either: aggregate bounded deltas into the final normalized input.
A failed/incomplete stream records its classification and counts. Actual bounded
partial content may be retained only under replay capture. Displaying provisional
text never upgrades metadata capture or authorizes retaining that text.

Display deltas are provisional and are not replayed as tool actions. Exact UI
timing replay is outside the initial format. Core replay concerns decision inputs
and effects.

## Commit and acknowledgement rules

Configure/check `journal_mode=WAL` and `synchronous=FULL`. Pin a verified SQLite
engine including the WAL-reset fix in 3.51.3 or a documented fixed backport.
Keep WAL files on a local supported filesystem, not a network share.
[SQLite WAL and version caveat](https://www.sqlite.org/wal.html)

An acknowledgement means the transaction committed under that durability
configuration, subject to SQLite's filesystem/storage guarantees. It does not mean
the external action happened. A transaction is also not tamper evidence.
[SQLite synchronous semantics](https://www.sqlite.org/pragma.html#pragma_synchronous)

For each lifecycle change:

1. Bound/serialize the event before queueing. Use a 2 MiB maximum serialized
   event as an initial implementation bound. Reject overflow; do not silently
   truncate a replay input and call it exact.
2. Reserve command-count and byte capacity within the run deadline and journal
   admission timeout. Accepted runners/ingress are themselves bounded, so
   arbitrary blocked senders cannot accumulate elsewhere.
3. The storage owner begins a transaction, appends the event, and updates the
   relevant projection/result/acceptance/idempotency record together.
4. It commits, then acknowledges. Count and byte reservations include the
   executing command and are released only on completion or definitive rejection.
5. The runner rechecks time/cancellation after acknowledgement before an external
   effect begins.

Ordinary journal admission uses the remaining execution budget. After a stop,
use the smaller of the journal admission timeout and the remaining one-time
5-second settlement grace from [performance](performance.md) for outcome and
terminal bookkeeping. This avoids a zero remaining run budget preventing the
timeout itself from being recorded. Grace expiry reports unresolved settlement;
it cannot abandon an in-progress commit or release capacity early. The controller
keeps supervising late completion and may finish its bounded bookkeeping later;
no model/tool effect, new/repeated checker, or automatic retry is permitted during
settlement.

Finalization follows [verification](verification.md): acknowledge the complete
model observation, release the model permit, and retain active-run ownership.
Assess within the remaining execution budget using the bounded pure checker,
then build the candidate/result and receipt. Reserve storage-inbox count/byte
capacity before the final time/cancellation check. After that wait, an observed
cancel/expiry changes the proposed execution outcome and makes checked acceptance
inconclusive. Transfer the command synchronously through the reservation, with no
unchecked await between arbitration and submission.

One terminal transaction persists the candidate/result and exact digest, the
acceptance receipt and both outcome projections, and the terminal event. It
cannot publish a completed phase first and fill in the verdict afterward.
There is no checker queue, post-terminal reassessment, or automatic repair turn.
Observers and command callers receive a final outcome only after this combined
commit; a persistence error never becomes a durable pass.

**Acceptance into the storage inbox is the terminal cancellation boundary.**
Before terminal-command submission, observed cancellation/expiry wins. After
submission the controller settles that command: a later cancel/deadline cannot
recall it or overwrite its committed outcome, including a passed receipt. A later
cancel request returns the settled state. Retain active-run ownership and all
still-running work until actual completion, even if an acknowledgement wait or
the settlement grace expires.

Start with individual transactions. If measured storage overhead matters, group
a bounded number of ready commands into one transaction while preserving each
run's order. A failed batch fails all its commands; none are acknowledged early.
Do not change to weaker synchronization without a separately named decision and
updated crash contract.

The writer's inbox bounds are in [performance.md](performance.md). A storage
failure closes admission and prevents further unrecorded effects. If an effect
already completed, preserve that uncertainty; do not repeat it to make the log
look complete. Telemetry/stderr can report persistence failure even when the
journal cannot record its own failure.

A task cancelled after a database command entered the inbox must still settle
the acknowledgement, including when the command has not started executing. Do
not release resources and assume rollback merely because its receiver stopped
waiting. Nonterminal acknowledgements still require a fresh time/cancellation
check before any subsequent effect or terminal-command submission.

## Submission idempotency

The API bounds the `Idempotency-Key` header and scopes it to authenticated
`owner_id`. Hash a versioned serialization of the validated submission:
the requested mode and model alias, requested lower limits, capture choice, and
the mode-specific fields. Freeform includes workspace alias and exact prompt;
checked includes task alias and rejects extra prompt/instructions, criteria,
expected answers, or a workspace override. Any future typed task parameters must
also enter this identity. Field order in incoming JSON does not change it.
Keep the requested-submission hash separate from the frozen effective
specification digest/version.

After owner/alias authorization, perform a bounded existing-key lookup. A matching
submission returns its existing run without acquiring new run capacity, even when
the run queue is full or an operator has since edited the task profile. Return
that run's original frozen identity, result, and verdict; do not resolve today's
profile to rerun or reinterpret it. Recheck current owner access to the original
run/resources. A different submission, including a changed mode or task alias,
is a conflict. Reads still use bounded service/storage resources; unavailable
storage can prevent a lookup. A new evaluation requires a new run/key.

For an absent key, reserve scheduler capacity, then atomically insert the run,
`run_accepted`, and the idempotency association. Recheck the key in this transaction
to handle concurrent first submissions. Release unused reservation on a duplicate
or definitive failure. Do not acknowledge new acceptance before commit.

A controller-owned submission operation settles commit and scheduling independently
of the HTTP receiver. Transfer that ownership before acceptance can commit. A
disconnect between commit and dispatch/202 must not abandon the accepted run.
An uncertain acknowledgement is settled by the controller, not assumed rolled back.

Require a key for service submissions. CLI runs may use null. Keep the key at
least for the configured minimum retry period (initially 24 hours from acceptance).
Do not delete its run row earlier to satisfy count/byte pressure; close admission
if necessary. After its record is eligible for purge and deleted, the service can
no longer recognize that retry. This deduplicates run creation,
not model requests or external tool effects.

## Querying and exporting

Every service query joins ownership, for example selecting events only through
a run whose owner matches the verified principal. Use parameterized SQL and
bounded pages; do not fetch every run/event into memory and filter afterward.

Reserve the global/per-run observer capacity before subscribing. Subscribe to
bounded notifications before the first catch-up query, then read bounded pages
after the last durable sequence cursor. Notifications are wakeups to query the
journal, not the authoritative event payload. Requery after wakeups; deduplicate
by sequence and include a bounded periodic catch-up so a coalesced/lost wakeup
cannot strand a terminal update. Never hold a database read transaction while
waiting on or writing to a subscriber. A terminal run's stream ends after catch-up.
The terminal projection includes the receipt from the same committed transaction;
never emit a completed/accepted notification while acceptance is still pending.

The `/events` API emits only a whitelisted public lifecycle/status projection,
never raw `events.data_json` or private replay inputs. Optional transient display
text is provisional and has no durable cursor/reconnect guarantee. Reconnection
retrieves durable status after its authorized cursor; GET returns the final result.
GET, list, CLI, and export expose both execution and acceptance outcomes and the
derived `task_accepted`; successful retrieval or HTTP 202 admission is not a pass.
Keep slow subscriber buffers separate from the journal queue and disconnect a
subscriber at its bounds. Hold observer reservations until the stream ends;
[performance](performance.md) defines both global and per-run caps.

An export reconstructs a versioned readable JSON/JSONL document through a
bounded stream. Export is a view, not the authoritative storage format. Private
replay exports require owner permission. A metadata export must clearly state
that exact replay, including recomputing acceptance, is unavailable. Rejected
candidates receive the same private-result handling as passed candidates.

## Recovery, retention, and backup

Acquire the controller lock before opening for recovery. Let SQLite recover its
transaction log. In one explicit recovery pass, mark queued, running, or
cancelling runs interrupted and append `recovery_interrupted`. No automatic
requeue, model regeneration, tool re-execution, or verification occurs. In the
same transaction set pending checked acceptance to inconclusive and write a
bounded interruption receipt using the frozen identities/criterion IDs. Freeform
remains unchecked and also gets an interruption receipt. Any already committed
terminal candidate, receipt, and event remain unchanged; an acknowledgement lost
before the crash does not justify reassessment or a replacement verdict.

A pre-terminal crash cannot leave a completed row without its receipt because
they share one transaction. Recovery does not infer a pass from a complete model
observation or reconstruct missing evidence from today's files. A missing receipt
in an unsupported imported/legacy format is not acceptance; reject incompatible
data until an explicit migration/import policy handles it.

A committed planned effect without its finished event is outcome unknown.
A crash can happen after an effect takes place but before its result commits.
SQLite transactions cannot close that external-system gap.

Retain completed/interrupted runs under an operator policy with count/byte/age
budgets. Delete complete runs, not random event rows, only after their minimum
idempotency window has elapsed. Retain active runs regardless of age. If space
cannot satisfy these guarantees, reject new work; do not silently shorten the
promised retry window. Document when eligible deletion loses deduplication memory.
Keep active-run data and storage-headroom checks
separate. Monitor database/WAL size and checkpoint behavior.

Use SQLite's backup API or a verified stopped/checkpointed backup procedure.
Do not copy only a live main database while ignoring its WAL. Restore into a
separate private directory, validate schema/integrity/results, and rehearse the
recovery policy before relying on a backup.
[SQLite backup API](https://www.sqlite.org/backup.html)

## Replay contract

Replay loads a compatible versioned replay capture, initializes the pure core,
supplies recorded model/tool/error/time/cancellation observations, and compares
proposed effects plus prepared-request fingerprints. It never opens tool-target
files, calls a model, or asks for new authority.

Capture semantics version 2 records `control_dispatch` on each finished model
or tool effect. This is the actual check after capacity/intent waits and before
dispatch: elapsed microseconds from the run's accepted monotonic origin,
whether cancellation was observed, and whether journal admission had already
stopped the run. A model intent that commits after a stop gets a finished
`unsent` observation and does not increment the model-attempt counter. Denied
tools likewise do not become executed effects.

The terminal event records `control_terminal`, sampled at the final check before
the synchronous transfer into the storage inbox. Replay applies the same
cancellation, execution-deadline, then journal-stop precedence to that observation.
Later settlement time cannot replace this decision. A model queue stop also
records its start, deadline, finish, and whether it timed out or capacity became
unavailable. Replay checks those budgets and does not infer a queue timeout from
today's clock. These bounded control fields contain no prompt or tool payload and
can remain in metadata capture.

Replay verifies control/event ordering, forbids dispatch after a recorded stop,
and reconstructs cancelled/deadline/queue/journal-stop receipts as inconclusive
for checked tasks. It never runs the checker to turn a stopped task into a pass.
Missing control observations and version 1 captures are explicitly unavailable;
retaining an old database row does not upgrade its capture. Recovery-interrupted
runs also remain unavailable because outstanding external effects may be unknown.
The same saved snapshot is never resumed or repaired by executing those effects.

Compatibility checks cover core/adapter/tool versions, schemas, and inputs.
Reject missing/redacted data, ordering errors, or a fingerprint mismatch at the
first divergence. A fixed live sampling seed is not replay.

Verification replay additionally requires the exact frozen task specification,
compatible checker/source-parser/output semantics, the exact candidate bytes,
and the actual observed evidence bodies bound to their owner/run/effect/sequence.
Rerun only the bounded pure checker against captured inputs and compare its
criterion outcomes/receipt identities. A digest or stored verdict alone is not
an evidence body. Missing inputs mean verification replay is unavailable; never
substitute the current task profile or workspace contents. Freeform remains
unchecked during replay.
Compare semantic verdicts and bindings, not equality of a newly measured checker
duration with the original recorded duration.

Imported receipts are untrusted source data, not live accepted results. Replay
can establish internal consistency against its captured inputs; it does not
authenticate their origin or create a cryptographically signed attestation.

Branching or resuming is a future execution feature, not a side effect of reading
events. It must define changed policy/configuration, world-state changes,
pending-effect reconciliation, and a new run identity.
