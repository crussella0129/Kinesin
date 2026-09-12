# Runtime and tool contracts

This document specifies behavior. The [build guide](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md) supplies the
implementation order; [security](security.md), [performance](performance.md), and
[traces](traces.md) own authority, shared limits, and persistence details.

## Core data

Use owned domain types. The exact Rust spelling is your work, but distinguish:

| Value | Required meaning |
|-------|------------------|
| `RunId` | Harness-generated opaque identity |
| `RunAuthority` | Trusted immutable principal, capability, allowed tools/model profile, budgets, capture |
| `Conversation` | Ordered system/user/assistant/tool messages |
| `ToolCall` | Provider call ID, function name, raw JSON argument string |
| `ModelReply` | Complete final message, complete tool batch, incomplete output, or protocol failure |
| `ToolResult` | Status, bounded body, truncation, optional error code/message |
| `TaskContract` | Frozen Freeform or FileFieldsV1 specification; no model-editable criteria |
| `EvidenceRecord` | Runner-owned observation bound to owner, run, effect, resource, bytes and completeness |
| `AcceptanceReceipt` | Separate status, contract/checker versions, candidate digest and required check outcomes |
| `RunState` | History, counters, pending effect/batch, repetition state, phase |
| `Effect` | Proposed model call, tool invocation, or terminal decision |
| `Observation` | Actual normalized reply/result/error or recorded external event |

A pure transition takes state and an observation and returns state plus the next
proposed effect. It never treats a model proposal as an authorization decision.
The runner enforces policy and resources before effects execute.

Do not parse hidden reasoning or `Action:` text. The supported adapter uses
structured chat/tool messages. Preserve any tested provider continuation fields
needed for subsequent requests without interpreting them as permission.

## Lifecycle and terminal outcomes

The externally visible phases are `queued`, `running`, `cancelling`,
`completed`, `stopped`, `failed`, `cancelled`, and `interrupted`.

- `completed`: a complete nonempty answer candidate and acceptance receipt were recorded.
- `stopped`: an explicit limit, repeated batch, context/output exhaustion, or
  other planned stop prevented completion.
- `failed`: invalid configuration/protocol, transport, or persistence prevented
  normal execution.
- `cancelled`: cancellation was handled and owned work settled.
- `interrupted`: restart recovery found unfinished work.

Execution phase is separate from acceptance status: `unchecked`, `pending`,
`passed`, `failed`, or `inconclusive`. Terminal runs cannot remain pending.
Only `completed + passed` yields the derived `task_accepted=true`. A wrong
answer can be `completed + failed`; a freeform answer is `completed + unchecked`.
The [acceptance contract](verification.md) defines independent checks, evidence,
verdict persistence, and strict CLI exit codes. A model claim is not evidence.

`cancelling` is useful: a running filesystem closure cannot always be stopped
immediately. Do not say work is finished while its resource permit was merely
dropped by an impatient waiter. The shutdown policy may report unresolved work;
a process exit still does not prove a remote effect stopped.

After any terminal decision, no new model/tool effect starts. A failed run is
not represented as an empty successful answer.

Reserve terminal storage-inbox capacity, then check cancellation/deadline before
synchronously submitting the command. Observed cancellation at that boundary
takes precedence over a candidate and makes checked acceptance inconclusive.
Once the command enters the inbox, settle it; later cancellation cannot recall
or overwrite it. The single runner serializes this decision; the HTTP handler
only signals cancellation. Already completed effects remain observations.

## Provider request and reply

Koil prepares `POST /v1/chat/completions` with the approved model identity,
structured messages, and explicit `n: 1`. Tools are omitted until enabled.
When tools are present, use `tool_choice: "auto"` and
`parallel_tool_calls: false`; still handle a returned batch safely.
A request carries **either** `tools` **or** a constraint, never both. The server
derives its own grammar for tool calls from the chat template, so a second
constraint has no slot to occupy; supplying one alongside tools is rejected by
request parsing. Build the two as disjoint branches so the exclusion holds by
construction rather than by convention.

A checked run therefore answers twice. While it gathers, requests carry tools and
no constraint. When it produces a prose answer, the runtime asks once more with
tools withdrawn and `response_format` set to a JSON Schema derived from the
frozen contract. That turn's reply is the candidate.

The schema constrains **shape only**. llama.cpp's converter skips unsupported
keywords silently, so depending on one for correctness would mean believing a
constraint that was never applied. Which ids are acceptable, which values are
right, and whether the cited observation supports them stay with the checker: a
well-formed candidate carrying a wrong value still fails.

Require exactly one choice at index zero and an assistant message. For calls,
require `type: "function"`, nonempty bounded IDs/names, and a string-valued
`function.arguments`. Arguments contain JSON and need a second parse into
the particular tool's typed struct.

| Provider reply | Runtime action |
|----------------|----------------|
| `stop`, no calls, nonempty text | Answer candidate; assess before terminal commit |
| `tool_calls`, valid nonempty identified batch | Validate and process batch |
| `length` | Incomplete output; run stops; no tool executes |
| Empty content with no calls | Empty-response failure |
| Missing/duplicate IDs in one batch, wrong role/type, contradictory reason/calls, unknown finish reason | Protocol failure; no tool executes |

Assistant content may be null or empty when calls are present. A complete
response is required even with streaming. Current llama.cpp uses
`tool_calls` as the finish reason and can report `length` for either output
or context limits. Extra grammar combinations are template-dependent.
[llama.cpp response serialization](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/server-task.cpp),
[request parsing](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/server-common.cpp)

## Initial tool definition

One preflight tool can be advertised as follows. Its schema constrains the
argument shape; the Rust handler still validates authority and semantics.

```json
{
  "type": "function",
  "function": {
    "name": "read_file",
    "description": "Read a bounded prefix of a UTF-8 text file inside the selected workspace. Use a relative path. Output reports truncation; it does not contain unseen file contents.",
    "parameters": {
      "type": "object",
      "properties": {"path": {"type": "string"}},
      "required": ["path"],
      "additionalProperties": false
    }
  }
}
```

The real request includes this in its `tools` array, with the other chat fields.
Use typed structs with unknown-field rejection and a shared lexical path
validator. Do not write a general JSON Schema engine for a fixed tool set.

Eight compiled tools are offered, alongside explicitly allow-listed MCP tools.
The first three only read; file mutations and command execution are gated
separately (see below and [security](security.md)).

| Tool | Arguments | Returns | Mints evidence |
|------|-----------|---------|----------------|
| `read_file` | `path` | A bounded UTF-8 prefix of one file | Checked runs only |
| `list_files` | `path` | A bounded nonrecursive listing | No |
| `search_files` | `path`, `query`, optional `case_sensitive` | Matching file names and one-based line numbers below `path` | No |
| `write_file` | `path`, `content` | Bytes written; replaces one file atomically | No |
| `edit_file` | `path`, `find`, `replace` | Bytes written; replaces one unique passage | No |
| `delete_file` | `path` | Removes one regular file | No |
| `move_file` | `path`, `to` | Publishes a destination without replacement, then removes the source | No |
| `run_command` | `command`, `args` | Bounded encoded stdout/stderr and exit outcome | No |

`search_files` exists because listing and reading alone cannot answer "which
file mentions this" without walking the tree one directory at a time, spending
steps and context. Its `query` is **literal text, not a pattern language**: an
expression engine would add a dependency and an unbounded matching cost on
model-selected input.

Matching **ignores capitalization unless `case_sensitive` is set**. Lexical
search retrieves nothing when the caller guesses the wrong form of a term, and a
smaller model is the least reliable at noticing that and refining the query, so
the forgiving mode is the default and exactness is the deliberate request. Case
folding decides the match only; a reported line is always the file's own bytes.
See the [paper review](https://github.com/crussella0129/building-an-agent-harness/blob/main/paper-review.md) for the measurement behind this.

**Only checked-run `read_file` observations mint evidence.** Unchecked reads
remain ordinary tool data and do not consume the verifier inventory or protect
otherwise eligible groups from compaction. A
listing and a search both return partial views of the workspace. If a search
hit could be cited, a candidate could claim a field equals a value it never
observed completely. The acceptance contract compares a claimed value against a
complete successful read; a search result can locate a file but can never
certify its contents. A run that intends to report a value must read it.

Search is bounded on every axis it can grow: directory depth, total entries
visited, bytes read from any one file, and the caller's result budget. Symlinks
and non-regular files are skipped rather than followed, and content that is not
valid UTF-8 is skipped rather than searched as replacement characters. Any bound
reached or content skipped sets `truncated`, so an empty result never implies
the workspace was fully examined.

`write_file` is the first tool that changes the workspace, and it changes the
trust model with it. Read the safety properties in [security](security.md) before
implementing it. The essentials here: it lives behind a **separate**
`WorkspaceWriter` capability, so the read tools have no method that can write; it
is built only for a workspace whose operator listed it, so a run that was not
granted writes has no writer at all; the write is atomic, landing in a temporary
sibling then renaming into place; and it refuses to leave the root, write through
a symbolic link, overwrite a directory, or create parent directories. It mints no
evidence. Content is bounded, and it arrives inside the tool arguments so it is
already under the argument limit.

`edit_file` replaces one exact passage. The `find` text must occur **exactly
once**: an absent passage cannot edit, and an ambiguous one is refused rather than
guessed, so an edit is never applied to the wrong place. It shares `write_file`'s
capability and guards, requires the file to already exist, refuses a file larger
than the editable bound rather than truncating it, and is atomic.

`delete_file` removes one **regular** file. A directory or a symbolic link is
refused, so it never removes curated structure or reaches outside the root, and a
missing file is an error rather than a silent success. `move_file` atomically
publishes a hard link only when the destination is absent, then removes the
source. A concurrent destination wins without being overwritten. The operation
requires hard-link support on one filesystem; it is not an atomic two-name
rename. If source cleanup fails, `move_source_cleanup_failed` reports that the
destination was created and source cleanup failed. Concurrent source replacement
still requires the proposed lease/fencing contract. Both tools share the writer's
capability and guards.

A **checked** task cannot enable any mutating tool. If a run could edit a file it
then reads, it could plant the value a criterion checks and cite its own change as
evidence, so authorization refuses that combination. Writes and edits are a
freeform-run capability.

## Complete tool history

Keep the assistant call message and append one corresponding result for each
call. Earlier system/user history remains in place.

```json
[
  {"role":"user","content":"What title is in note.txt?"},
  {
    "role":"assistant",
    "content":null,
    "tool_calls":[{
      "id":"call_1",
      "type":"function",
      "function":{"name":"read_file","arguments":"{\"path\":\"note.txt\"}"}
    }]
  },
  {
    "role":"tool",
    "tool_call_id":"call_1",
    "content":"{\"status\":\"ok\",\"body\":\"Title: Kinesin notes\",\"truncated\":false}"
  }
]
```

A tool-call message and all its results form one conversation unit. Do not send
the next model request with unmatched results or half of an accepted batch.
Do not duplicate observations in a separate prompt field. No `finish` tool is
needed when ordinary final text already supplies an answer candidate.

## Batch execution

The loop must let new observations change the next proposed action. A scripted
prewritten action list is insufficient for tasks whose next input is discovered
by a tool. Return bounded actionable errors and preserve their correlation, so
the model can choose a different permitted action within the same budgets.
That next decision is not an automatic retry of an ambiguous transport/effect.
There is no requirement to emit or parse a free-form `Thought:` trace, add a
`think` tool, or reproduce the textual ReAct protocol.

Backend speculative decoding verifies candidate output tokens. Kinesin separately
validates and authorizes effects. A sidecar prediction, valid JSON prefix, or
decoder-accepted draft is never permission to execute a tool; the full reply and
batch must still pass the existing gate. See the [paper review](https://github.com/crussella0129/building-an-agent-harness/blob/main/paper-review.md).

1. Validate the full reply structure, all IDs, and the entire batch's remaining
   tool budget before any handler executes. Reject a structurally invalid or
   over-budget batch as a whole.
2. Compare ordered names and normalized arguments against the previous batch.
   Stop before the third consecutive identical batch when `repeat_limit=3`.
   Ignore new provider IDs and JSON whitespace for this comparison.
3. Append the assistant message once. Process calls serially in returned order.
   Invalid/unknown/denied calls consume the tool-call budget too.
4. Decode arguments, check the trusted tool allow-list, and validate the path.
   With a usable ID, ordinary argument/tool errors become bounded observations.
   Bad correlation is a protocol failure, not an observation to guess around.
5. Record `tool_planned` with the decision. A denial records `tool_finished`
   without a handler. An allowed call acquires bounded blocking capacity and
   rechecks deadline/cancellation before actual dispatch.
6. Record each result before feeding it to the core. If cancellation happens
   mid-batch, settle already started work, terminate, and send no incomplete
   history to the model.

Normalize recognized arguments through their typed representation. For an
unknown tool with valid JSON, compare parsed values. For malformed JSON, exact
string comparison is adequate because the overall budgets still bound variants.

## File-tool semantics

The private `WorkspaceReader` owns an open `cap_std::fs::Dir`. Trusted startup
opens the configured root; model input never opens a new ambient root.
Lexical validation rejects NUL, parent components, rooted paths, Windows
prefixes, `:`, and paths beyond a small declared limit (initially 4096 UTF-8
bytes). `.` is valid for directory listing.

`read_file` opens through that capability, checks the opened handle is a
regular file, and reads a bounded prefix. Reject unsupported text encodings.
At a size boundary, distinguish a clipped trailing UTF-8 character from invalid
interior data. The final serialized result—including escapes and error fields—
must fit `max_tool_result_bytes`; reserve envelope space and mark truncation.

`list_files` is nonrecursive. Bound visited entries (initially 256) and collected
name bytes as well as serialized output. Sort the collected names; if scanning
stops early, say the result is incomplete. Do not claim a stable subset of a huge
directory merely because you sorted the subset the OS happened to enumerate.
Handle unrepresentable names explicitly; never feed a lossy display name back as
a valid access path.

The initial workspace contract is an operator-provisioned ordinary file tree.
No special files, devices, credential trees, or cross-owner hard links/mounts are
part of that input surface. A post-open metadata check cannot prevent a FIFO open
from blocking. Capability resolution addresses path authority, not arbitrary
kernel I/O latency; bounded workers preserve capacity accounting.
[cap-std security model](https://github.com/bytecodealliance/cap-std),
[Dir API](https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html)

## Result envelope

```json
{
  "status": "error",
  "body": "",
  "truncated": false,
  "error": {"code": "not_found", "message": "The requested file was not found."}
}
```

Statuses are `ok`, `error`, or `denied`. Use stable error codes and bounded
helpful text. Do not expose service filesystem paths or credentials in errors.
Truncation must describe what the model actually received; never imply the
entire file was read if only a prefix was available.
For actual successful checked-run reads, the runner can add `evidence_id`, a unique
reference inside this run. Its envelope bytes count toward the same result cap.
Only runner-owned records can resolve that reference. The checker also verifies
resource identity, status, completeness, and the claimed value; a citation alone
does not pass. See [task acceptance](verification.md#runner-owned-evidence).

## Budgets and context

Local MCP tools use operator-declared stdio executables. Their descriptions and
results remain untrusted data, and dispatch requires both a frozen discovered
definition and the run's independent tool grant. Schema checking currently
requires an object and advertised required keys; it is not a general JSON
Schema validator. MCP tools never mint evidence and cannot be enabled in checked
runs.

Jobs carry declarations without starting processes. The admitted active runner
prepares its own server pool, records the frozen schemas before model dispatch,
and awaits process cleanup before normal terminal acknowledgement. Discovery
failures and cancellation have recorded outcomes; pure replay consumes the
frozen observations without reconnecting.

Protocol input is bounded before JSON decoding: 256 KiB per frame, 16 MiB and
4,096 frames per server session. Initialization, notifications and requests all
count toward that independent frame limit; a noisy server can reach it before
the run's tool-call budget. Discovery permits at most eight servers, 64 allowed tools,
16 pages and 1,024 advertised tools per server, with a 64 KiB total frozen
metadata envelope. Each description/schema has its own 4/16 KiB cap. Cyclic
cursors and duplicate advertised names fail. The model and tool result budgets
apply separately. Server processes receive a scrubbed environment and no
inherited stderr channel; owned descendants are terminated during awaited
cleanup. Operator-trusted servers must not deliberately escape that ownership.

Per-run limits are defined in [configuration](configuration.md); shared capacities
are in [performance](performance.md). A run's absolute deadline includes queueing,
journal acknowledgements, model waits, HTTP body reads, and tool work. Each wait
uses the smaller of its local timeout and the remaining run time.
After a stop decision, the separate settlement grace permits only bookkeeping;
it never extends execution authority. Pending blocking work/commits can outlive
both waits and stay controller-owned. See [performance](performance.md).

Limit conversation bytes separately from outgoing serialized request bytes.
Tool definitions, templates, and encoding add overhead. Before admitting a new
observation, check whether the next state would exceed its history budget; stop
clearly rather than silently dropping evidence.

Byte limits are not token accounting. Verify the server's actual per-slot context
and explicitly disable context shifting. When the pinned backend supports chat
input-token counting, count the same template/tools/options and reserve generation
space plus a stated margin. Otherwise rely on bounded inputs and explicit
context/length failure; do not claim exact preflight.

Streaming does not relax any bound. Frame accumulation, argument strings,
aggregate text, event queues, and subscribers need limits. Visible text is
provisional until the final reply is classified. [Integration](integration.md)
specifies stream assembly.

## Shared-service API contract

Implement this only after the concurrent runtime gate. Authentication/authorization
and deployment requirements are in [security](security.md).

| Endpoint | Contract |
|----------|----------|
| `POST /v1/runs` | Authenticate/validate; look up authorized retry; for absent key reserve admission and atomically recheck/create; controller owns acceptance and dispatch; return 202 and owner-scoped run ID |
| `GET /v1/runs` | Owner-filtered, cursor-paginated metadata; bounded page |
| `GET /v1/runs/{id}` | Owner-scoped phase, acceptance status/receipt, derived task_accepted, and bounded terminal candidate/result |
| `GET /v1/runs/{id}/events` | Authorized bounded public status projection and optional transient display text; never raw replay events; slow client disconnects without stopping the run |
| `POST /v1/runs/{id}/cancel` | Authorize and signal owning scheduler/runner; acknowledge request, not instantaneous effect termination |
| `GET /v1/runs/{id}/export` | Owner-authorized bounded export; replay payload requires capture permission |

A tagged submission is either `freeform` with `workspace`, `model`, and `prompt`,
or `checked` with `task` and `model`. Both may request smaller limits and an
allowed capture choice. The checked profile generates its fixed task instruction:
reject extra prompts, workspace overrides, supplied criteria or expected answers.
No submission supplies an authoritative owner, path, URL, credential, or arbitrary
tool definition. Reject unknown fields. Task permission intersects resource permissions.
Require a bounded principal-scoped `Idempotency-Key`; same key plus a different
normalized submission returns 409.
The fingerprint includes mode and task selection; effective contract contents
are frozen separately. An identical retry returns its original verdict even if
operator configuration changed. Recheck access to the retained run's resources.
HTTP 202 means admitted and 200 means retrieved, never accepted as correct.
Every status/list/terminal SSE/export projection includes both outcome axes and
the identified contract scope. Publish terminal state only after the combined
candidate/receipt transaction commits; do not run a checker after SSE closes.

Use 400 for invalid input, 401 for failed authentication, 404 for absent or
inaccessible run IDs, 413 for oversized bodies, 429 for caller quota exhaustion,
and 503 for global admission/storage unavailability. Do not leak another owner's
resource existence in an error. Completed cancellation requests are idempotent;
return the already terminal phase without repeating any effect.

Before acceptance can commit, a controller-owned submission operation takes
responsibility for settling persistence and scheduling, independent of the HTTP
receiver. Disconnect between commit and dispatch/202 cannot strand the run.
Client disconnect or an SSE observer leaving does not cancel accepted work;
cancellation requires the explicit endpoint
or its deadline. List/status remain available after a controller restart for
recorded runs. Unfinished runs become interrupted, not transparently resumed.

Bound ingress before expensive parsing/task creation; middleware order matters.
There is no browser UI or permissive cross-origin access in the initial service.
No client deletion endpoint is required; retention is an operator workflow.
