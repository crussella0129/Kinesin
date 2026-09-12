# Local CLI

Start with [installation and first use](getting-started.md) for Windows/Linux
build tools, PATH, configuration, model startup and the fixture explanation.
`kinesin --help` (or `cargo run --locked -- --help` in a checkout) prints usage
without opening configuration, storage, a workspace or a model connection.

## The session

```text
kinesin
```

No arguments. It reads `kinesin.toml` from the working directory, then asks what
to do. Each entry becomes its own run that cites the previous one, so a follow-up
needs no run id and no flag:

```text
> what does project.txt say the language is?
> and where else is that recorded?
```

A session takes no workspace or model alias. With one of each configured there is
nothing to choose; with several, the operator names them in a single-run command
instead, because guessing would silently pick an authority. `kinesin --config
other.toml` opens a session against a different file. End it with Ctrl+C or by
closing the input.

Each entry is a separate immutable run in the journal, linked by the answer it
cites. Nothing rewrites an earlier run, so `inspect` and `export` work on any
entry in the thread exactly as they do on a single run. Only the previous
**answer** carries forward: metadata capture deliberately does not retain a
prompt, and a thread whose content depended on the capture mode would behave
differently for two owners running the same words.

The cited answer enters as reference data with its own entry in the input
inventory, marked as earlier model output. It is information, not instruction,
and grants no permission.

Run these commands from the repository root after the getting-started setup.
Single runs, sessions and `batch` require the verified model server. Configuration paths resolve
relative to the configuration file; CLI input-file paths resolve from your current
directory. Run execution uses the implicit local operator identity; `--owner`
is limited to provisioning credentials for a configured service owner.

The bundled examples expect a deliberately prepared `workspace/project.txt`;
its synthetic contents are in [the checked-task setup](configuration.md#add-an-explicit-checked-task).
They also expect the verified `kinesin-qwen25-coder-7b` model alias on loopback
port 8080. Follow [the recorded server startup](model-preflight.md#reproduce-the-baseline)
or edit your copied example to match the separately verified endpoint.

```text
cargo run -- --config examples/file-task.toml --task practice-fields --model local
cargo run -- --config examples/first-turn.toml --workspace practice --model local --prompt "Say hello" --allow-unchecked
cargo run -- batch --config examples/file-task.toml --input examples/batch.jsonl
```

Both commands use the same bounded controller, frozen authority, model/resource
pools, journal, and acceptance checker. The batch contains independent runs.
It does not authorize agents to spawn other agents or share conversation state.

## Batch input

Supply one JSON object per line. A checked task selects an operator-defined task
and model. A freeform submission selects a workspace, model, and prompt.

```json
{"submission":{"mode":"checked","task":"practice-fields","model":"local"}}
```

```json
{"submission":{"mode":"freeform","workspace":"practice","model":"local","prompt":"Summarize project.txt."},"allow_unchecked":true}
```

The optional `allow_unchecked` belongs to that row only and is rejected for a
checked task. Unknown and duplicate fields are errors. Checked submissions reject
prompts, workspace overrides, criteria, and expected answers. Ordinary permitted
`capture` and lower `limits` live inside `submission`, as defined in
[configuration](configuration.md).

The reader limits the file to 16 MiB, each line to 64 KiB including its newline,
and the number of rows to 1,024. It checks growing files incrementally as well as
checking the initial size. UTF-8 JSON and LF/CRLF are supported. Blank lines are
invalid rows. Invalid JSON or unauthorized rows receive their own error; other
bounded rows can continue. File-I/O or size-limit errors stop further input while
already accepted runs settle.

The producer reads one bounded line at a time and retains at most the configured
active-plus-queued number of completion handles. When admission is full it waits
for existing work, rather than submitting the rest of the file as waiting tasks.
One awaited blocking job reads input, and one awaited blocking job writes output;
neither creates an unbounded producer or result queue.

A single run using a streaming model profile also emits `kind: "text_delta"`
lines with `run_id`, `text`, and `provisional: true`. These are display progress,
not candidates accepted by the checker. Only the durable final `kind: "run"`
line publishes the final phase and receipt. At most 128 text frames of 2 KiB each
can wait for display; lag disconnects the observer and may emit
`kind: "display_closed"`. Model/tool execution and settlement continue under
their existing ownership. Batch output currently contains terminal results only,
including when its model adapter assembles a streamed response.

## Output and exits

Single and batch commands emit JSON Lines. A run line contains `kind: "run"`, its
one-based input `index`, `run_id`, both `phase` and `acceptance_status`, the
`receipt`, retained `result`, derived `task_accepted`, and per-item `exit_code`.
Batch lines are emitted as completions are drained; use the index/run ID rather
than assuming input order. An input/admission error has `kind: "error"`, an
optional index, and a bounded diagnostic.

Final candidates remain visible even when rejected. Treat the receipt's scope
and verdict as authoritative; merely seeing an answer does not establish task
acceptance. JSON escaping preserves candidate data while preventing raw terminal
escape/control sequences. Output is bounded to 8 MiB per encoded line and only
one output operation is retained outside the bounded completion set.

Exit codes follow [task acceptance](verification.md#cli-and-service-meaning):
passed checked completion 0; execution/setup failure 1; failed acceptance 2;
unchecked/inconclusive completion 3; cancellation 130. Explicit freeform opt-in
allows exit 0 for `completed + unchecked`, while `task_accepted` stays false.
The batch aggregate uses priority **130, 1, 2, 3, 0** and preserves individual
outcomes. An unchecked opt-in cannot excuse a failed checked task.

Ctrl+C stops new admission, requests cancellation for queued/active runs, and
joins the controller and storage writer. Results already committed keep their
verdicts. Running blocking filesystem or output operations may delay final
settlement; their capacity remains owned. A slow terminal therefore affects
display/overall batch progress without creating an unlimited output queue.

## Inspect, export, and replay

These commands do not require a running model. Inspection and export use the
implicit `local` owner; a run belonging to a service owner is not accessible by
passing its ID. They open the controller's private state through the same lock
and SQLite writer. Stop the other controller first. Opening state performs normal
startup recovery, so unfinished work from a previous process becomes interrupted.
The supplied current configuration must still validate, including its workspace
locations; it is not used to reassess a retained result.

```text
cargo run -- --config examples/file-task.toml --task practice-fields --model local --capture replay
cargo run -- inspect --config examples/file-task.toml --run RUN_ID
cargo run -- export --config examples/file-task.toml --run RUN_ID --output state/run.json
cargo run -- replay --input state/run.json
```

Substitute the actual run ID from the first command. The examples default to
metadata capture, so `--capture replay` must be selected on the original run.
Use a new export filename and a private state directory as provisioned in the
getting-started guide. Stop any other Kinesin session using this state first.
`inspect` prints the saved result,
receipt, both outcomes, derived `task_accepted`, and at most 256 event summaries.
Summaries contain sequence, kind, and elapsed time; they exclude raw replay inputs.
`events_complete: false` identifies a longer history.

`export` writes a versioned snapshot using bounded owner-filtered pages, with
limits of 256 events and 32 MiB. It never silently truncates a capture or overwrites
an existing file. A storage/write failure is an error; an I/O failure can leave an
incomplete new file, which must not be treated as a successful export. New files
inherit their destination directory's permissions; existing ACLs are not changed.
Replay exports include private instructions and observed evidence. Metadata
exports also contain private final results and receipts, and explicitly state
that exact replay and acceptance recomputation are unavailable.

The export summary's `capture_present` means replay inputs were requested; it
does not certify completeness or compatibility. `replay` reads only the supplied
bounded snapshot, without loading configuration, opening a workspace or database,
constructing a model client, or starting an async runtime. It rebuilds recorded
requests and decisions and recomputes compatible checked receipts from captured
evidence. Missing inputs, modified bindings, unsupported versions, and divergences
are errors. Current version 3 captures support completed runs, deterministic error/limit
endings, and recorded cancellation, deadline, model-queue, and journal-admission
stops. Dispatch and terminal decisions use their captured control observations;
late settlement cannot overwrite the recorded pre-inbox decision. Version 3 also
records bounded MCP startup, frozen schemas and cleanup. Older unsupported
capture/semantic versions, missing control inputs, recovery-interrupted runs, and otherwise
unavailable external-stop observations are explicit refusals. Imported data is
not authenticated and cannot create live authority or a new accepted result.

Inspection/export exit 0 means the operation succeeded. Replay exit 0 means its
captured inputs and recorded decisions are internally consistent, including when
the original task failed acceptance. Read `acceptance_status` separately.

## Retention and backups

These local operator commands use the same exclusive state lock and writer,
without creating model clients or workspace capabilities. They apply to the
operator's whole database, including service owners' records.

```text
cargo run -- retain --config examples/file-task.toml --limit 100
cargo run -- backup --config examples/file-task.toml --output private-backups/kinesin.sqlite
```

Retention removes at most the requested 1–100 eligible terminal runs per command.
It uses the configured policy and current time; the CLI cannot lower the minimum
retention window. Active runs and runs still inside the minimum window remain.
Deleting an eligible run also removes its idempotency association.

The `[storage.policy]` settings are `max_bytes` (default 4294967296), `max_runs`
(default 100000), and `minimum_hours` (default 24). `max_bytes` is operational
admission headroom: the controller compares database/WAL size plus reservations
for retained run data before creating a new run. It is not an exact physical disk
cap. SQLite pages, WAL/checkpoint behavior, filesystem allocation, backups, and
exports need additional free space. Removing records need not immediately shrink
the database file. Monitor actual disk space and keep backups outside this budget.

Create the private backup directory beforehand, outside the current state
directory. Backup uses SQLite's backup API, waits for completion and integrity
checking, and requires a new destination file. Existing files are never
overwritten. Restore into a separate private state directory and validate its
results before using it as the controller's state. The backup contains private
data for every retained owner.

## Provisioning and the private service

First add the owner/resource and service tables in
[configuration](configuration.md#shared-service-additions). Create the state and
verifier directories with restricted OS permissions, outside every tool workspace;
the commands validate these existing trees and do not repair their ACLs. Follow
the [native state audit](native-state-audit.md) for the separate deployment checks.

```toml
[service]
listen = "127.0.0.1:7070"
credential_verifiers = "state/credentials.json"
max_submission_bytes = 65536
max_page_size = 100
idempotency_retention_hours = 24
trusted_state_sids = []
```

`credential_verifiers` points to a JSON verifier file. `provision` creates it when
absent or adds a credential under the provisioning lock. On Windows,
`trusted_state_sids` can list at most eight additional operator-approved SIDs;
the current process identity, SYSTEM, and Administrators are already recognized.
This setting allows those principals in the audited ACL; it does not grant access
or change account membership. Unix configurations leave the list empty.

```text
cargo run -- provision --config kinesin.toml --owner alice --hours 24
cargo run -- serve --config kinesin.toml
```

Provisioning requires an existing configured owner and an integer lifetime of
1–8760 hours. Its stdout contains the complete bearer token once; this is the
deliberate secret-delivery command. Store the token privately. The verifier file
contains the token ID, owner, expiry, revocation flag, and secret hash. Do not put
plaintext tokens in examples, configuration, logs, or command arguments.

`serve` opens the private state, shared controller, pooled model clients and
workspace capabilities, then binds only the configured loopback address. It starts
unready and checks the credential file and each configured model's health,
identity, context size and slot capacity. A supervised monitor repeats the checks
after a five-second interval. Invalid verifier reloads disable authentication
until a valid replacement loads; expiry/revocation reject new requests. Existing
runs keep their admitted authority. An unavailable model closes readiness and new
admission while authorized retained results remain retrievable.

Clients supply a single `Authorization: Bearer TOKEN` header. Create requests
also require `Content-Type: application/json` and a bounded `Idempotency-Key`.
The body is a raw tagged submission, without the batch wrapper or an owner field.
The authenticated `GET /ready` returns only `ready` and uses 503 when unavailable.
The public `GET /health` returns only `{"alive":true}` while the process serves
HTTP, including when model readiness or credentials are unavailable. It performs
no storage query and remains under the listener's connection limits.
HTTP 202 means admission, while the persisted receipt and `task_accepted` describe
task correctness. The [API contract](loop-and-tools.md#shared-service-api-contract) defines
the run endpoints. Public historical SSE frames name their snapshot `current_run`
and omit a receipt; terminal frames contain `run` with the committed receipt.
Cancellation acknowledges an in-memory signal to the owning controller, not a
durable cancellation-intent record or instantaneous termination. A late request
returns the committed terminal result. If unfinished work has no live controller,
the endpoint returns unavailable with `cancellation_requested: false`; startup
recovery still classifies unfinished runs as interrupted.

Ctrl+C closes new admission and signals active work, then joins ingress,
monitoring, controller and storage ownership. A failed monitor also closes
admission. A blocking verifier read already in progress remains owned until it
finishes; stopping cancels cancellable network health waits. Keep remote access
behind verified TLS and complete the dedicated service-identity and network
checks before exposing the listener beyond its private deployment boundary.

Windows startup registers the Tokio console listener and then restores Ctrl+C
delivery for this process. A launcher can otherwise pass down an ignore flag
that registering a handler alone does not clear; see Microsoft's
[SetConsoleCtrlHandler contract](https://learn.microsoft.com/en-us/windows/console/setconsolectrlhandler).
The native regression test uses a separate hidden console, explicitly starts
with that flag set, sends an actual console event, and requires the child to exit.

## Configuration bounds that preserve settlement

`max_response_bytes` cannot exceed 1048576 (1 MiB). Both the HTTP adapter and
scripted client count encoded response data before returning a candidate, so
retained candidates fit the journal's fixed ceiling. Smaller limits are allowed.
`journal_queue_bytes` must be at least 2162688 (2 MiB + 64 KiB); its default remains
8388608. This minimum fits one maximum event/result, receipt copies, command
metadata, and the terminal arbitration reservation. Invalid combinations are
rejected while loading configuration, before opening state or dispatching effects.

## Validation scope

The local CLI proof runs six checked tasks through a synthetic loopback HTTP
provider, real capability file reads, independent field checking, and SQLite,
with one active and one queued slot. All six must persist passed receipts. This
tests the complete harness path and batch pacing; live-model goal quality is
measured separately. The reader, per-row authorization shape, exit aggregation,
and terminal escaping also have focused tests.

CLI integration tests inspect and export real journaled checked runs, then rename
the workspace, configuration, and database before launching the separate replay
process. They also reject altered candidates and metadata-only replay, deny
cross-owner inspection, preserve existing export/backup files, restore a checked
receipt from backup, and verify that recent runs survive retention. A listening
model fixture receives no requests from these operator commands.
