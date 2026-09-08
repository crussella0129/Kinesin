# Build guide

This guide takes Kinesin from an empty Cargo package to concurrent local agents,
then to a deliberately bounded shared service. It is the path for writing your
own Rust implementation, with contracts, reading, exercises, and proofs at each
boundary.

The **`answer-key` branch contains the reference implementation**. Its observed
results and remaining deployment gates are in the [validation ledger](build-validation.md).
Keep that checkout available for comparison and use a separate empty directory
for your handwritten build. Mark your own steps complete only after their proofs
pass; the reference's results do not establish your implementation's behavior.

## Before you start

For your first sitting, work through steps 1–3: describe the personal and shared
operating profiles, practice ownership and `Result` in a separate scratch
package, and create one small package whose binary calls its library. The
numbered steps below supply the commands and checks.

You can learn the pure core and run a scripted model without a downloaded model
or GPU. Step 4's provider preflight is a separate gate before live networking
and real tool calls.

Each completed checkpoint remains useful even if the next takes weeks. Step
numbers express dependency order, not a schedule or a fixed step count. Use the
[roadmap](roadmap.md) as your checkpoint checklist, [resources](resources.md) as
the reading index, and [understanding](understanding.md) for feedback and
self-check questions.

| Steps | Checkpoint |
|-------|------------|
| 1–6 | Understandable types and a pure conversation core |
| 7–14 | One bounded, durable, async model turn |
| 15–22 | Two read-only tools, a per-run acceptance check, and live task evaluation |
| 23–29 | Bounded concurrent runs for the same owner |
| 30–31 | Inspection, replay, and streaming: the first local release |
| 32–33 | Remote inference and optional server supervision |
| 34–41 | An authenticated shared service with an explicit exposure gate |

Read [architecture](architecture.md) once before starting. Keep
[configuration](configuration.md), [loop and tools](loop-and-tools.md),
[traces](traces.md), [verification](verification.md), [security](security.md), and
[performance](performance.md) beside the guide as contracts. Their tables own
the field names and limits. If an experiment changes a contract, update the
contract and its tests together.

Use one package. Add modules as their steps arrive: `core.rs`, `runner.rs`,
`model.rs`, `config.rs`, `policy.rs`, `tools.rs`, `storage.rs`, `verification.rs`, and later
`scheduler.rs`, `service.rs`, `ingress.rs`, and `operator.rs`. K-Core names the
decision-making role; Koil names the model adapter. Kineserve is an optional
process supervisor.

The pure core stays synchronous. The first real HTTP client uses Tokio and
reqwest. SQLite owns journal transactions from the first persisted run.
Capability-based file access arrives with the first file tool. These choices
serve the stated requirements; they are not exercises in replacing libraries.

Each step has a build task, a proof, a failure exercise, and focused reading.
Read enough to attempt the exercise, then return to the reference when a real
question appears. Prefer owned data first. A small understandable clone is
better than lifetime parameters you cannot yet explain.
When a Rust concept is unfamiliar, reproduce it in scratch code. Finish the
current proof before adding another module or dependency.

When a diagnostic stops you, work down this list before changing the design:

1. Read the whole diagnostic, starting at its `help:` and `note:` lines.
2. Run `rustc --explain` on the error code.
3. Read the type's signature in the standard library documentation.
4. Reduce the problem to the smallest case that still fails.
5. Write the expected behavior as a sentence; a wrong assumption often appears
   while you write it.
6. Return to it later. Borrow-checker errors respond well to a break.
7. Ask on the [Rust users forum](https://users.rust-lang.org/).

Most early ownership errors resolve at step 1 or 2. Reach for a redesign after
that list, not before it.

At coherent checkpoints, run:

```text
cargo fmt --all
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

While working, use `cargo check` and targeted tests. The commands above become
applicable after step 3 creates the package and lockfile. Live model tests remain
separate from the normal offline suite. See [process](process.md).

## 1. Write the two operating profiles

**Build:** save a short project note describing the first release and the later
service. The first release accepts concurrent independent runs from one trusted
owner, with operator-selected model and workspace aliases. The service later
accepts several authenticated owners with separate workspaces, limits, and data.
Both initially expose only trusted, compiled, read-only file tools.

Name the assets: workspace contents, prompts, outputs, model credentials, service
credentials, stored results, journal data, compute time, memory, and disk space.
Draw who can submit input and which component is allowed to open each resource.
Keep an untrusted model response and an untrusted file in the picture.

Write three observable goals: unauthorized tool access is denied; overload stays
bounded; latency can be divided into waiting, model, tool, and storage time.
Minimality means each mechanism has a job you can explain.

**Prove:** explain why the local release is useful before there is a public API.
Explain why remote inference alone does not create a multi-user harness service.

**Break:** imagine a file telling the model to read credentials. Identify the
Rust policy check that must deny the action regardless of the model's wording.

**Read:** [security](security.md) and [performance](performance.md). Use their
threat model and measurement definitions as the starting project requirements.

## 2. Set up Rust and an ownership scratch project

**Build:** verify one consistent development environment. Native Windows is
fine; use WSL only if you deliberately choose its filesystem and toolchain.

```text
rustup --version
rustc --version
cargo --version
rustup component add rustfmt clippy
rustup doc
```

Create a separate scratch package outside the Kinesin source tree. Write a
function taking a `String`, another borrowing `&str`, and a fallible function
returning `Result`. Deliberately use a moved value, then fix the ownership.
Keep the compiler diagnostic and a sentence explaining what changed.

**Prove:** explain which function owns its input and why its caller can or cannot
use that input afterward. Read the complete diagnostic rather than only its
first line. Try `rustc --explain E0382`.

**Break:** return an error from the fallible function. Handle it with `match`,
then propagate it with `?` from another fallible function.

**Read:** [ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
and [recoverable errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html).
Focus on moves, short borrows, and the caller's responsibility for failure.
For the diagnostic you just produced, the Brown fork's
[fixing ownership errors](https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html)
explains each rejection and its correction directly.

## 3. Create one package and a repeatable checkpoint

**Build:** create a separate empty directory for your handwritten Kinesin and
open a terminal there. Keep the `answer-key` checkout as your reference; it
already has a Cargo package. Initialize your learning package:

```text
cargo init --bin --name kinesin .
cargo check
cargo run
```

If Cargo reports an unrelated or invalid ancestor `Cargo.toml`, add an empty
`[workspace]` table to this package's manifest and rerun `cargo check`. That
explicit boundary keeps Cargo's workspace discovery inside your independent
learning package. This was required on the validation machine; see the
[recorded setup friction](build-validation.md#steps-23-observed-results-and-guide-friction).

Keep `main.rs` small. Add `lib.rs` with one public function that the binary
calls and a small unit test. Add a CI workflow running the checks in [process](process.md).
Commit `Cargo.lock` for this application and record the toolchain used in CI.
Exclude build outputs, model weights, databases, secrets, and generated captures.

A package containing a library and a binary contains two crate targets; it does
not require a multi-package workspace. Add modules when behavior requires them.
Do not create empty plugin, worker, or transport frameworks.

**Prove:** the binary calls library code, the offline test passes, and CI runs
format, Clippy, and tests from a clean checkout.

**Break:** make the unit expectation fail locally, inspect the output, and fix
it. Verify that the CI command would propagate the nonzero result.

**Read:** [packages and crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
and [Cargo lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).

## 4. Prove the model contract independently

**Build:** follow [integration](integration.md) to record the exact llama-server
build, model source/revision, quantization, chat template, launch arguments, and
effective context and slot settings. Start the server manually.

Save a successful health check and non-streaming text exchange. Separately save
a complete tool exchange: assistant call, matching tool result ID, final answer.
Save sanitized request and response bodies under `tests/fixtures/`.
An answer that merely describes a tool is not a structured tool request.

Record whether tool support is proven or unresolved. Text support is enough for
step 13; resolve tool support before step 22's live exercise. The pure and fake
steps can proceed without a downloaded model or GPU.

**Prove:** another run of the recorded command reproduces a valid response
shape. Identify nullable fields, finish reasons, and the tool-arguments string
in the actual response.

**Break:** stop the server and repeat the request. Save the failure category,
not a secret-bearing terminal dump.

**Read:** the pinned build's [server documentation](https://github.com/ggml-org/llama.cpp/tree/master/tools/server)
and [function-calling guide](https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md).
The repository links track development; retain your tested commit separately.

## 5. Define owned domain types

**Build:** add `core.rs`. Describe a conversation message, model observation,
tool call, run state, and terminal outcome with ordinary structs and enums.
Use `String`, `Vec`, `Option`, and typed error variants. Keep HTTP DTOs,
database rows, clocks, file handles, and Tokio types outside this module.

Give each status one meaning. Distinguish completed, failed, cancelled,
interrupted, budget-limited, and incomplete-generation outcomes. An empty answer
is not automatically successful. A queued or running status is not terminal.
Follow the shared vocabulary in [loop and tools](loop-and-tools.md).
Execution ends as `completed`; task acceptance is a separate outcome. Start
with `unchecked` for freeform work. Step 21 adds the first checked task.

Begin with the fields needed for one text turn. Add identifiers and counters
where they establish real correlations; do not generalize to every possible
provider message format. Fixed identifiers are sufficient inside unit tests.

**Prove:** construct an initial conversation and inspect it through a test.
Explain which value owns the conversation and how an outcome reaches the CLI.

**Break:** try representing mutually contradictory statuses with booleans in
scratch code. Replace that representation with an enum and explain the gain.

**Read:** [structs](https://doc.rust-lang.org/book/ch05-00-structs.html),
[enums and matching](https://doc.rust-lang.org/book/ch06-00-enums.html).

## 6. Make state transitions without I/O

**Build:** write the smallest pure transition functions. Initial instructions
and a prompt produce ordered messages. An ordinary model observation produces an
answer candidate or a defined error. Unexpected tool calls fail at this checkpoint,
because tools have not been enabled.

Have transitions propose the next effect or terminal outcome. The caller will
perform the effect and return an observation. Do not read a clock, generate an
ID, open a file, log a message, or make a network request inside the transition.
Passed-in budget/deadline observations can be ordinary values.

Test roles, message order, empty text, invalid response classification, and
terminal-state behavior. A terminal state must not propose another model call.
Keep tests small enough that the input explains the expected result.

**Prove:** `cargo test` exercises the core with no external resources. Explain
the difference between proposing a model request and sending it.

**Break:** submit another observation to a terminal run. Return the documented
invalid-transition result rather than starting a second run accidentally.

**Read:** [writing tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
and [patterns](https://doc.rust-lang.org/book/ch19-00-patterns.html).

## 7. Learn enough async to run the shell

**Build:** first use the scratch package for two delayed async operations.
Observe the difference between awaiting them in order and driving both before
awaiting completion. Then add Tokio to Kinesin:

```text
cargo add tokio --features rt-multi-thread,macros,time,sync,signal
```

Give the binary one runtime entry point. Keep `core.rs` synchronous.
Practice moving owned data into a task and retaining its join handle.
Explain `Send` as transferability between threads and `'static` task inputs
as freedom from borrowed data with shorter lifetimes.

**Prove:** a slow fake operation does not prevent an independent timer from
advancing. The program waits for work it owns before reporting completion.

**Break:** put `std::thread::sleep` inside an async task in scratch code,
observe the blocked runtime thread, then replace the example appropriately.
Do not solve every future error by adding `Arc<Mutex<_>>`.

**Read:** [Rust async fundamentals](https://doc.rust-lang.org/book/ch17-00-async-await.html)
and [Tokio spawning](https://tokio.rs/tokio/tutorial/spawning).
The useful distinction is waiting versus executing, not a claim that async
makes the model compute faster.

## 8. Run a scripted model through one runner

**Build:** add `model.rs` and `runner.rs`. Start with a `ModelClient` enum
whose variants are `Scripted` and, when implemented, `Http`. Ordinary enum
dispatch is sufficient; an async plugin trait is not a prerequisite.

Define two operations: pure request preparation, then async sending of the
prepared request. Preparation produces owned immutable bytes plus safe metadata.
Sending uses those bytes without rebuilding the conversation. The scripted
client captures the request and returns the next scripted observation or error.

Let one runner own one run's state. It asks the core for a decision, prepares
and sends the request, then returns the observation to the core. The fake uses
the same boundary as the live adapter; it does not bypass important validation.

**Prove:** one fake-backed run returns its answer and makes exactly one expected
request. A second test returns a model error without a panic or retry.

**Break:** exhaust the script. Report an explicit fixture/client error instead
of silently repeating its last response.

**Read:** [enum methods and matching](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
and [Tokio testing](https://tokio.rs/tokio/topics/testing). Use the fake to learn
the boundary before making a real HTTP request.

## 9. Validate configuration and construct run authority

**Build:** add `config.rs` and `policy.rs`, with Serde and TOML parsing:

```text
cargo add serde --features derive
cargo add serde_json toml
```

Parse text separately from reading a file. Validate required fields, unknown
fields, nonzero limits, capture policy, model destinations, and workspace aliases.
Resolve operator-provisioned aliases before a run starts. Model input never
supplies a new endpoint or host filesystem root.

Construct immutable `RunAuthority`: owner, allowed workspace capability
selection, permitted model alias, tool allow-list, and effective limits.
Local mode uses the explicit local owner; service mode will derive identity
from authentication. Configuration is fixed for an admitted run.

Keep a small ordered inventory of the initial context: installed instructions
and user request, their source IDs/origins, purpose, and byte counts. Use concrete
owned values beside the conversation. Hashing/persistence arrives with the
journal boundary. This step needs no directory scanner or profile loader.
Later tool observations keep their originating call as source provenance.

Add the CLI shape used in step 13. Keep path arguments as `PathBuf` and report
invalid prompt text or missing arguments clearly. Generate run IDs at the shell
boundary using an established random-ID crate; supply fixed IDs in tests.

**Prove:** a valid config drives the fake. Unknown aliases and invalid budgets
fail before admission, journal creation, model requests, or tool execution.

**Break:** use malformed TOML, a missing file, and a caller-supplied URL in the
model-alias argument. Each must have a specific validation error.
Also edit the config after admission: the admitted run must keep its original
instructions, and a later run may pick up the edit.

**Read:** [Serde attributes](https://serde.rs/container-attrs.html),
[TOML crate](https://docs.rs/toml/latest/toml/), and [configuration](configuration.md).
For the context design rationale, read the [folder-method assessment](paper-review.md#folder-structure-and-context).

## 10. Persist a run through one SQLite owner

**Build:** add `storage.rs` and `rusqlite` with its bundled SQLite feature.
First test a synchronous `Store` against temporary databases. Create the
versioned schema in [traces](traces.md): runs, ordered events, and necessary
correlations. Bind SQL parameters. Enable foreign keys, WAL, and
`synchronous=FULL`; verify the effective settings when opening the database.
Require SQLite 3.51.3 or newer, or a documented backport of its WAL-reset fix;
check the actual linked engine version, not just the Rust crate version.

Before opening/recovering live state, open `state/controller.lock` without
truncation and acquire an exclusive process lock. Keep its handle for the
controller's lifetime, without cloning it or passing it to children.
A second controller using that state directory fails before recovery.

Append an event and update its run-status projection in one transaction.
The terminal transaction includes the bounded candidate/result and acceptance
receipt. Initially that receipt says `unchecked`; step 21 adds real assessment.
Uniqueness on run/sequence and valid status transitions catch accidental
duplicates. A transaction acknowledgement means commit succeeded; enqueueing
a command is not a durable acknowledgement.

Then move the connection into one dedicated `std::thread`. Give it bounded
Tokio `mpsc` commands and `oneshot` replies; the thread uses blocking receive.
The async shell awaits replies. No SQLite call runs on a Tokio runtime thread.
Bound command count, serialized bytes, waiting callers, and individual commands.
Step 24 generalizes this admission discipline across the controller.

**Prove:** a fake run reopens with coherent events and terminal status. Default
metadata capture excludes prompt and tool bodies. An explicitly enabled
synthetic replay fixture retains normalized inputs and observations once.
The bounded final answer and receipt are always stored with terminal state,
separately from replay capture; they remain sensitive even in metadata mode.

**Break:** fail between event insertion and projection update; both must roll
back. Fail the store worker or acknowledgement path; the runner stops further
effects and never retries an external operation merely to repair its journal.

**Read:** [rusqlite transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html),
[SQLite WAL](https://www.sqlite.org/wal.html), and
[synchronous settings](https://www.sqlite.org/pragma.html#pragma_synchronous).
Use [`File::try_lock`](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock),
stable since Rust 1.89, with the lifetime rules in [traces](traces.md).

## 11. Map the wire format with saved fixtures

**Build:** define private request and response DTOs in `model.rs`.
Serialize prepared requests with `serde_json`. Translate the response shape
from step 4 into the domain observation expected by the core.

Use structured chat history with one requested choice. Check the assistant role,
choice count/index, finish reason, and response/call consistency before returning
an executable tool batch. Preserve tool IDs and nullable assistant content.
Decode a function's arguments string separately when its tool is selected.

For this checkpoint, omit tools and use non-streaming requests. Preserve the
error categories needed later: HTTP failure, invalid JSON, unsupported protocol,
generation length, and context failure. An HTTP 200 is only transport success.

**Prove:** saved ordinary responses decode correctly; malformed choices and
contradictory tool/finish fields fail. Preparation is deterministic for the same
domain request and settings, independent of sockets and the store.

**Break:** supply `finish_reason: "length"` with apparently complete tool
arguments. Classification must never return executable calls from it.

**Read:** [serde_json](https://docs.rs/serde_json/latest/serde_json/) and the
[wire contract](loop-and-tools.md). Keep fixture expectations tied to the pinned
server rather than assuming every OpenAI-style endpoint behaves identically.

## 12. Add the bounded HTTP adapter

**Build:** implement the `Http` variant using a reusable reqwest client.
For the researched 0.13 release line, a focused starting feature selection is:

```text
cargo add reqwest@0.13 --no-default-features --features rustls,stream
```

Send prepared bytes with an explicit JSON content type. Configure total/connect
timeouts, trusted destinations, disabled redirects, and the documented proxy
policy. Do not put credentials in request URLs or journal metadata.
Bound body bytes while reading chunks, before decoding JSON; a response's
`Content-Length` alone is not enforcement.

Use a local controlled HTTP stub for success, HTTP errors, stalled reads,
oversized bodies, and unexpected encodings. Prefer a maintained test server
library or framework over writing a general HTTP parser. Bound the stub too.
Reuse connections across turns; do not build a new client for each request.

**Prove:** the same core and runner work through both client variants.
The request sent is the prepared request, and limits apply to error bodies too.

**Break:** send a redirect to a different listener and an unbounded chunked body.
Neither may bypass destination or byte policy.

**Read:** [reqwest Client](https://docs.rs/reqwest/latest/reqwest/struct.Client.html),
[feature flags](https://docs.rs/crate/reqwest/latest/features), and
[integration](integration.md). Client clones reuse its internal shared pool.

## 13. Complete the first live command

**Build:** compose validated CLI input, immutable authority, store worker,
runner, and HTTP adapter. Start llama-server manually with the step 4 command.

```text
cargo run -- run --config kinesin.toml --workspace practice --model local --prompt "Say hello" --allow-unchecked
```

Here `practice` and `local` are configured aliases. Tools remain disabled for
this checkpoint, even though a workspace is selected. Print the answer to stdout
and diagnostic/status information to stderr. Print `Completed; unchecked` for
this intentional freeform demonstration. Without `--allow-unchecked`, completed
freeform work exits 3. The strict default exits zero only for a completed task
whose checks passed; follow [the exit table](verification.md#cli-and-service-meaning).
The flag changes exit policy, never task acceptance, and cannot accompany a
checked-task selection. Failures, cancellation, and limits remain nonzero.

Persist required lifecycle events before claiming a fully recorded completion.
Always store the bounded final result and receipt with terminal state in the same transaction.
This owner-scoped result is sensitive retained content; metadata capture excludes
intermediate payloads, not the final answer. See [traces](traces.md).
Close admission to the store, finish acknowledged work, and join its thread.

**Prove:** a real answer is displayed and the stored run has the matching
`completed + unchecked` status. Run without the opt-in and verify exit 3.
Record the exact smoke-test command and versions.

**Break:** stop the model server. The command must end within its configured
bounds with a useful error and a failed run, not an empty successful answer.

**Read:** [CLI argument handling](https://doc.rust-lang.org/std/env/fn.args_os.html)
and [process](process.md). This is a useful checkpoint; commit its evidence.

## 14. Enforce stopping and cancellation around effects

**Build:** apply the limits from [configuration](configuration.md) to the runner.
Track model attempts, retained history, prepared request bytes, response bytes,
and elapsed run time. Derive operation deadlines from remaining run time instead
of restarting a fresh full timeout after every wait.

Add a cancellation token and check it before admission and before each new
effect, including after a journal acknowledgement or permit wait.
Record model intent before sending. If sending becomes disallowed, close that
intent as not started. If the request may have reached the server, classify the
transport outcome honestly; no automatic generation retry is needed.

Apply the cancellation precedence rule in [task acceptance](verification.md#one-finalization-path):
observed cancellation wins before the terminal command enters the storage inbox.
Once the inbox accepts that command, settle it; a later cancellation cannot
recall it or rewrite its eventual committed result. A finished external
action remains finished even when the caller no longer wants its result.

Give stopped runs the nonrenewable five-second bookkeeping grace specified in
[performance](performance.md). It allows recording the timeout/unsent outcome
when the execution budget is already exhausted, never another model/tool call.
Retain ownership and permits for a commit or blocking operation still finishing
after that grace; report unresolved settlement instead of claiming it stopped.

**Prove:** cancellation and budget exhaustion prevent the next effect.
Malformed/oversized responses and `length` outcomes cannot become success.

**Break:** cancel after intent commit but before send; then cancel while the
stub is responding. Distinguish known not-started work from an uncertain remote
outcome. Do not assert that dropping HTTP guarantees model compute stopped.

**Read:** [CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)
and [Tokio timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html).
Timeouts require cooperative async execution; they do not preempt arbitrary code.

## 15. Define the three tool contracts

**Build:** add typed argument structs and explicit dispatch in `tools.rs`.
Start with `list_files(path)`, `read_file(path)`, and `search_files(path,
query)`. Describe their purpose,
relative-path requirement, output bounds, and error behavior in the schemas
sent to the model. Reject unknown argument fields and validate semantics after
JSON decoding.

Use one bounded result envelope with a clear success, error, or denied status.
A usable call ID permits an error observation to be returned to the model.
A broken ID or contradictory response is a protocol failure before dispatch.
Follow [loop and tools](loop-and-tools.md) for exact field names and counting.

Test the schema examples against the Rust argument validators. Include an
unknown tool, a misspelled field, a wrong JSON type, empty input, and malformed
JSON inside the arguments string. A schema describes input; authority comes
from `RunAuthority` and the tool implementation.

**Prove:** valid arguments reach the intended handler; rejected arguments reach
no handler and produce a bounded, correlated observation where possible.

Decide now which tools may mint evidence, because the acceptance contract
depends on it. Only a complete successful read qualifies. A listing and a search
return partial views, so they must carry no evidence reference even when the
runner holds one. Otherwise a candidate could cite a search hit as proof of a
value it never observed completely.

**Break:** add an apparently helpful extra argument such as a new workspace
root. It must not broaden the run's authority. Send a `query` to `read_file`
and omit it from `search_files`; both must be denied before any handler runs.

**Read:** [Serde container attributes](https://serde.rs/container-attrs.html)
and [tool contracts](loop-and-tools.md). Keep argument definitions concrete;
a general plugin registry can wait.

## 16. Create the workspace capability

**Build:** add `cap-std`. At trusted setup, open the configured workspace
directory once. Put its directory capability inside a private `WorkspaceReader`
wrapper that exposes only the two read operations required by this project.
The wrapper must not expose write methods or its underlying directory handle.

Validate model paths conservatively: allow the documented relative form and
`.` for listing; reject traversal, rooted paths, drive-relative/UNC/device
prefixes, and Windows alternate-stream syntax. Then perform access relative to
the capability. Do not turn the result back into an absolute path and call
ambient `std::fs` from the handler.

Create a practice tree containing only operator-approved text and directories.
Capability confinement does not decide whether a file inside that tree is
sensitive. Hard links and special files also require the provisioning rules in
[security](security.md), particularly before shared service use.

**Prove:** ordinary nested files work, while traversal and links escaping the
root are denied on each supported platform. Record unavailable symlink-test
privileges explicitly rather than silently treating a skipped case as passed.

**Break:** replace a path with a link during repeated reads. Confirm the access
uses the capability boundary and cannot reach the outside sentinel file.

**Read:** [cap-std](https://docs.rs/cap-std/latest/cap_std/) and
[directory capabilities](https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html).
The private read-only wrapper is Kinesin's policy, beyond the library's API.

## 17. Implement a bounded file read

**Build:** have `WorkspaceReader` open an authorized relative file for reading.
Check the opened handle's type and read only a bounded prefix plus the small
lookahead needed to detect truncation. Do not call whole-file convenience
functions and truncate afterward.

Reject unsupported file types and invalid UTF-8 according to the tool contract.
Distinguish a character cut by the byte limit from an invalid sequence earlier
in the input. Fit the complete serialized result envelope, including escaping,
within the result budget; shortening only the unescaped body is insufficient.

Provisioned workspaces must exclude FIFOs, devices, and other special files.
An OS file operation can still stall; the blocking-work limit in step 26 limits
resource use, not the maximum duration of every kernel operation.
Keep this limitation visible in the supported local/service profile.

**Prove:** empty, short, large, missing, invalid-text, and boundary-UTF-8 files
produce the expected result and `truncated` value without oversized allocation.

**Break:** use quotes, backslashes, and control characters that expand during
JSON serialization. The final encoded envelope must still fit.

**Read:** [Read and bounded adapters](https://doc.rust-lang.org/std/io/trait.Read.html)
and [UTF-8 validation](https://doc.rust-lang.org/std/str/fn.from_utf8.html).
Keep reads synchronous inside the wrapper; the runner will place them correctly.

## 18. Implement a bounded directory listing

**Build:** list one authorized directory without recursion. Bound visited/kept
entries and serialized result bytes during iteration. Handle per-entry I/O
errors deliberately and avoid collecting a whole directory before limiting it.
Sort only the bounded set actually collected.

Represent entries with only the information the model needs: relative name and
supported kind. Decide how non-UTF-8 names are reported; lossy display text must
not accidentally become a different executable path. Never reveal ambient
absolute paths in a denial or diagnostic.

A truncated listing is incomplete. It is not a promise that the alphabetically
first entries were returned, because the underlying enumeration order is not
your sorting order. State that clearly in the tool description.

**Prove:** a small directory returns understandable stable output for the
collected entries. A large directory stays within both traversal and output
budgets and explicitly reports truncation.

**Break:** create a bounded synthetic fixture with many long names and a
disappearing entry. The handler must return a defined result without a panic.

Then add `search_files(path, query)`, so a run can find which file mentions a
term instead of walking the tree one directory at a time. Keep `query` literal
text. A pattern language would add a dependency and an unbounded matching cost
on model-selected input. Bound every axis it can grow along: directory depth,
entries visited, bytes read from any one file, and the caller's result budget.
Skip symlinks and non-regular files rather than following them, and skip content
that is not valid UTF-8 rather than searching replacement characters. Report all
of that as truncation, so an empty result never implies a complete examination.

**Prove (search):** a term in a nested file is found with its one-based line
number; a term that appears nowhere returns an empty result rather than an error;
a search never returns an evidence reference.

**Break (search):** point a search at a path outside the workspace root, at a
tree deeper than the depth bound, and at a file with thousands of matches. Each
must produce a bounded, defined result that reports its own incompleteness.

**Read:** [cap-std directory iteration](https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html#method.entries)
and [OS strings](https://doc.rust-lang.org/std/ffi/struct.OsStr.html).

## 19. Complete one model/tool/model round trip

**Build:** extend the core and runner to handle a tool batch. Keep the assistant
message containing the requests, then append one correlated tool result for
each accepted call. Send another model request only after the batch has all
required observations.

Validate the entire batch's IDs and remaining call budget before any handler
runs. Process calls in returned order, serially within one run. Independent runs
will overlap later; serial tools keep each conversation's first implementation
easy to reason about.

Around each effect, enforce authority and limits, commit its intent, recheck
cancellation, run the handler, and commit the observation. A failed observation
write stops further work. It never causes the tool to be executed again.
Store normalized message deltas under replay capture rather than copying the
entire growing conversation into every event.

**Prove:** a scripted `read_file` request receives its correlated result and
then completes. Inspect the second model request to verify the history order.
Add a contingent case: a first observation reveals a previously unknown filename,
and the next call must use it. In a second case, a `not_found` observation leads
to a different allowed discovery/read action. Fakes prove feedback delivery;
the live task evaluation separately checks whether a model uses that feedback.

**Break:** cancel between two calls in a batch. Stop explicitly; never send
incomplete call/result history to the model merely to obtain a final answer.

**Read:** [the loop contract](loop-and-tools.md) and [journal ordering](traces.md).
Read [ReAct, sections 2–4](https://arxiv.org/abs/2210.03629v3) for the feedback
pattern; Kinesin uses structured calls rather than parsing its textual syntax.
Explain each boundary aloud before adding another tool.

## 20. Prove policy, protocol, and loop failure paths

**Build:** complete the hostile-input matrix in [testing](testing.md).
Include duplicate/missing IDs, an unknown finish reason, contradictory fields,
unknown tools, denied paths, invalid arguments, repeated calls, and exhausted
run/model/tool/history/request budgets.

Compare repeated requests using normalized tool names and arguments, not
model-generated call IDs or JSON whitespace. Keep the whole-run limits as the
final bound for loops that alternate different requests.
Count denied and invalid admitted calls according to the same budget contract.

Use explicit failure codes in tests rather than matching an entire human error
sentence. A valid correlated tool error can enter history; broken outer protocol
or failed storage ends the run. These are different recovery choices.

**Prove:** every malformed batch test records zero forbidden handler invocations.
One boundary test must cover each condition that prevents a new effect.

**Break:** return a valid first call and an invalid second call ID. The valid
first handler must not run before the whole batch's correlation check.

**Read:** [table-driven tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
and [testing](testing.md). Prefer fixtures that catch plausible mistakes over
large numbers of tests mirroring individual implementation lines.

## 21. Check file fields before accepting a task

**Build:** add `verification.rs` with the owned `TaskContract` enum and pure
`FileFieldsV1` checker in [task acceptance](verification.md). Begin with handwritten
candidate JSON and synthetic evidence; no model or new dependency is necessary.
Use one to four required fields from tiny UTF-8 `key=value` files. Implement the
specified source grammar, strict candidate JSON, revision rules, and byte limits.
Deserialize directly into typed structures so duplicate or unknown fields cannot
silently disappear. Require exactly the configured criterion IDs, with no extra
narrative or claims. The renderer displays accepted fields itself.

Resolve an operator-provisioned task alias before admission. Its immutable
profile fixes workspace, required fields, checker/version, and generated task
instruction. Freeze its effective specification digest in `RunAuthority`.
Keep freeform and checked submissions disjoint: a checked task accepts task/model
aliases, without a prompt, workspace, criteria, or `--allow-unchecked` override.
Reject empty/duplicate criteria and unknown checkers before any model effect.
Provision a synthetic `practice-fields` alias using the profile contract in
[configuration](configuration.md), then add the checked CLI shape:

```text
cargo run -- run --config kinesin.toml --task practice-fields --model local
```

Give each actual successful file observation a runner-owned `evidence_id` in its
bounded tool-result envelope. Keep a private bounded inventory binding that ID
to this owner/run, effect, normalized resource, observed bytes/digest, completeness,
and journal sequence. The model can cite an ID but cannot create the record.
Retain/account for these observation bytes under the evidence cap, even if history
later releases them; do not reread the filesystem during checking.

For every required field, resolve the real observation, parse the configured key,
and compare the candidate's exact value. A real citation to an unrelated file or
an error containing the right word is insufficient. Wrong values, forged IDs,
and wrong-source references fail. Missing, malformed, truncated, or conflicting
source evidence is inconclusive. Use the canonical precedence when checks differ;
no empty-set or average-score pass is allowed.

Now wire the tested checker into the runner's single finalization path. A complete
model answer is a candidate. Finish its observation acknowledgement and release
model capacity, then check remaining time/cancellation, assess bounded inputs,
and produce the receipt. Recheck time/cancellation before submitting one terminal
transaction for phase, candidate/result, receipt/projection, and terminal event.
The storage-inbox acceptance boundary decides cancellation arbitration. A commit
failure cannot advertise acceptance, and no checker runs after termination.

Report both execution and acceptance. Only `completed + passed` derives
`task_accepted=true` and gets strict exit zero. Rejected candidates remain bounded
owner-visible results; their presence is not a pass. Freeform stays unchecked.
Keep receipt diagnostics sanitized and evidence bodies out of metadata capture.

**Prove:** a scripted checked task extracts `language=Rust` from the configured
source, cites its real evidence ID, and returns a durable passed receipt. Change
the candidate to `Python` while retaining that genuine citation: execution still
completes, acceptance fails, and the CLI exits 2. Reopen the store and retrieve
the same result and receipt together.

**Break:** use irrelevant/forged evidence, an omitted field, duplicate JSON keys,
extra prose, conflicting reads, truncated input, and a checker fault. A file
instructing the checker to mark PASS changes nothing. Exercise cancellation and
deadline expiry before terminal submission, then cancellation after inbox
acceptance. Every case follows the explicit failed/inconclusive/execution rules;
none silently skips checking, retries a model, or passes unavailable evidence.

**Read:** [task acceptance](verification.md),
[Serde container attributes](https://serde.rs/container-attrs.html), and
[adversarial checks](testing.md). Explain the exact claim this checker establishes:
configured fields match complete observations from this run, not arbitrary-world
truth or freshness at completion. Keep the broader model evaluation separate.

## 22. Evaluate the useful read-only agent

**Build:** prepare a tiny synthetic workspace and a task set with clear
verifiable answers: find a file, read its title, compare two short notes, answer
a greeting without tools, handle a missing file, and deny an outside path.
Record expected facts and permitted effects rather than exact generated wording.

Create a small task card per fixture: selected inputs and their order, required
evidence, allowed/forbidden effects, expected facts, and an independent checker
or manual rubric. Evaluate the step 21 checker itself, including false positives
and false negatives; reuse it per run for its supported task profiles. Keep
execution completion, contract acceptance, broader goal correctness, and policy
violations separate. A fluent freeform answer remains `completed + unchecked`.

Run the controlled context and feedback cases in [testing](testing.md). Compare
identical source text under different organization separately from a smaller selected
context; otherwise a reduction in input size can be mistaken for a layout gain.
Use the existing explicit config and synthetic fixtures for this experiment.
No automatic folder-based workflow or PDDL planner is required.

Run the entire set against scripted models first. Then use the pinned real
model after its manual tool round trip has passed. Capture task success,
protocol failures, denied calls, model turns, tool calls, elapsed time, and the
model/server/config identity.
Include the receipt's contract/checker identity and acceptance result. Measure a
wrong but fluent answer even when its protocol and runtime behavior are valid.

Keep payload replay opt-in and confined to the synthetic fixture. Journal
metadata and the always-retained final result have different privacy purposes.
Do not save private file bodies just because an evaluation failed.

**Prove:** one real checked run reads the required practice fields and passes
its contract. Separately assess a freeform task with the human rubric without
relabelling it passed. The offline suite remains independent of models and GPUs.

**Break:** put instruction-like text in a practice file asking for an outside
read. Even if the model follows that text, Rust must enforce the denial.

**Read:** [testing and evaluation](testing.md) and [security](security.md).
The [planning-paper assessment](paper-review.md#symbolic-planning-and-validation)
explains why deterministic checks help without implying that instruction tuning
is a prompt-only feature or that training iterations are runtime retries.
Save this as the read-only checkpoint before adding concurrency.

## 23. Make each concurrent run own its state

**Build:** make the runner's ownership explicit before spawning many runs.
Each run owns its conversation, counters, pending calls, event sequence,
effective authority, frozen task contract, evidence inventory, and child cancellation token.
Share only suitable controller resources: immutable configuration, the pooled
HTTP client, bounded store access, and resource/admission controls.

Use a controller cancellation token with a child per run. Cancelling one child
must not cancel its siblings. Give each scripted run its own response script.
Keep one logical writer of transitions for each run even though storage is
physically serialized by one database thread.

Run two fakes with controlled interleaving. Put different sentinel text in
their prompts, workspaces, and outputs so any cross-run contamination is obvious.
Do not protect the entire loop with a global mutex to make the test pass.

**Prove:** both runs advance while the other waits. Neither sees the other's
messages, tool results, evidence, receipts, cancellation, identifiers, or authority.

**Break:** cancel run A while run B waits for its model response. B must finish
normally and retain its own coherent journal.

**Read:** [Tokio shared state](https://tokio.rs/tokio/tutorial/shared-state)
and [child cancellation tokens](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html#method.child_token).

## 24. Bound admission, queues, bytes, and waiters

**Build:** add `scheduler.rs`. Give incoming work an admission decision before
creating an unbounded number of tasks or allocating all batch input.
Apply the configured pending-run count, queued-byte budget, and queue deadline.
Use explicit overload/queue-expired outcomes with no model or tool effects.

A bounded channel alone is insufficient if unlimited tasks wait outside it
holding prompts. Either reject immediately with nonblocking admission or bound
the number and size of waiting submitters as well. Reserve queued-run count and
bytes together. Return both on rejection or queued cancellation; when a run is
dequeued, transfer ownership to its active-run allowance and release its queued
count and bytes exactly once.

Apply the same accounting to store commands. Commands have a maximum size;
retained payloads and queued payloads cannot bypass the byte reservation. A store
command keeps its count and byte permits after dequeue, through actual transaction
completion, even if its acknowledgement receiver is dropped.
Do not drop lifecycle events silently when their queue fills. Fail closed on
store admission failure and use the bounded cleanup path for owned work.

**Prove:** a controlled flood reaches a stable bound for queued runs, queued
bytes, waiting callers, and store commands. Rejected runs invoke no model.

**Break:** mix many tiny requests with a few maximum-sized ones, then cancel
half while queued. Every count/byte reservation must return exactly once.

**Read:** [Tokio bounded channels](https://tokio.rs/tokio/tutorial/channels)
and [capacity budgets](performance.md). Draw the queues, including the ones
implied by futures waiting for a permit.

## 25. Separate active runs from model capacity

**Build:** implement distinct active-run and per-model request limits.
The starting settings are in [performance](performance.md); model concurrency
must be clamped to the verified backend capacity, even if more runs are active.
Acquire model capacity only for an actual model exchange.

Keep the permit until response consumption and request cleanup have finished.
Release it before tool execution and before waiting for the next model turn.
Avoid holding unrelated permits while waiting on each other; document a single
acquisition order wherever multiple reservations are needed.

Put counters in a fake model that records current and peak in-flight calls.
Use a barrier-controlled response so the test can inspect the saturated state
without relying on brittle millisecond sleeps.

**Prove:** active runs and model requests independently remain under their
limits. A run waiting on a file does not occupy a model slot.

**Break:** cancel a request while waiting for a model permit, then during the
response body. Verify accounting and record any uncertainty about remote work;
client cancellation is not a guaranteed server-side kill.

**Read:** [Tokio Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html)
and [model capacity](performance.md). More permits are a tuning experiment,
not an automatic performance improvement.

## 26. Bound blocking tool work by its real lifetime

**Build:** place synchronous workspace operations in `spawn_blocking` after
acquiring the dedicated tool-work capacity. Move the owned permit into the
blocking closure so it is held until the actual file work returns.
Keep SQLite on its existing dedicated thread, not this per-tool pool.

Track the join handle and operation ID. A caller timeout may stop waiting, but
started blocking work continues. It must not free capacity early, restart the
same tool, or write new state directly from a detached closure.
Return observations through the run's coordinated completion path.

Use a synthetic blocking operation controlled by a test channel. This allows
testing a slow operation and cancellation without introducing a real hanging
filesystem into the normal suite.

**Prove:** unrelated async timers and model runs continue while tool work waits.
Peak actual blocking operations, including timed-out callers, stays bounded.

**Break:** time out several callers while their closures remain blocked.
No replacement work starts beyond the configured capacity until those closures
actually finish. Release the test barriers so the suite can cleanly terminate.

**Read:** [spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
and [blocking-work policy](performance.md). Started blocking tasks cannot be
aborted; a shutdown timeout only changes how long the runtime waits.

## 27. Shut down and cancel without losing ownership

**Build:** implement shutdown in order: stop admission, request cancellation,
finish or reconcile owned operations, persist terminal state where possible,
drain acknowledged store work, and join owned tasks and the store thread.
Use a task collection such as `JoinSet` so task failures are observed.

Keep a clear distinction between cancellation requested and a terminal run.
Persist intent for cancellation where the service contract requires it.
Avoid using task abortion as the ordinary cleanup protocol.
No new model or tool effect may start after the run observes cancellation.
For checked work cancelled before terminal-command submission, settle acceptance
as inconclusive. Once the storage inbox accepts finalization, observe its actual
outcome rather than racing another terminal write. Do not launch a checker during
settlement grace or turn a partial assessment into a pass.

Define bounded shutdown waiting and the residual state when an OS operation
does not return. Resource permits belong to that work until it really finishes.
The process cannot promise both hard preemption and completion recording for
arbitrary in-process blocking operations.

**Prove:** cancel while queued, waiting for model capacity, reading HTTP,
waiting on a tool, and waiting for a journal acknowledgement.
Every scenario has an explicit outcome and owned cleanup path.

**Break:** make the store worker fail while two runs are active. Stop admission
and further effects; do not claim journaled success for the affected runs.

**Read:** [graceful shutdown](https://tokio.rs/tokio/topics/shutdown) and
[JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html).

## 28. Add a batch command for independent agents

**Build:** add a documented batch input format with the same disjoint freeform
and checked-task shapes as single runs. Parse it incrementally with entry and byte limits.
Submit each item through the same admission path as a single run.
The CLI must not spawn all inputs first and hope the scheduler bounds them later.

Assign a run ID at accepted submission and display concise progress keyed by ID.
Return a per-run result and an aggregate exit status. One failed run must not
erase another run's answer or convert the whole batch into unqualified success.
Include each receipt and derived `task_accepted`. Apply the strict per-item rules
and aggregate exit priority in [task acceptance](verification.md#cli-and-service-meaning).
An unchecked opt-in cannot excuse a failed or inconclusive checked task.
Bound output buffering; a slow terminal must not create an unlimited result queue.

This milestone is concurrent independent agents for one owner. It does not
implicitly grant models an agent-spawning tool, shared mutable conversation,
or permission to exchange each other's workspace data.

**Prove:** a batch of more items than active capacity processes correctly with
bounded waiting. One intentionally slow or failed run leaves others useful.

**Break:** cancel a queued item, fail an active item, and then request controller
shutdown. Check each run's final status and the batch command's exit status.

**Read:** [iterator-based processing](https://doc.rust-lang.org/book/ch13-02-iterators.html)
and [scheduler behavior](performance.md). Keep command syntax and example input
in the CLI documentation once you implement them.

## 29. Measure saturation before tuning concurrency

**Build:** add measurement spans and a repeatable load driver using scripted
models first. Record admission wait, model-permit wait, model time, tool time,
checker time, store-ack time, total time, rejection, queue size, and peak resource use.
Separate execution completion, contract acceptance, and independently judged goal
correctness. Include evidence/receipt memory and report accepted-task throughput
alongside raw completed-run throughput; quick wrong answers are not useful gains.
Keep metric labels low-cardinality; prompts and run IDs do not belong in labels.

Repeat fixed workloads at increasing concurrency. Then measure the pinned live
model with known slot/context settings. Include short and long runs and both
warm and cold conditions. Do not combine model-loading time with a warm-request
latency percentile without labeling the experiment.

Use the measurement plan and initial objectives in [performance](performance.md).
Record sample counts and workload/config identity beside p50/p95/p99 values.
Too few observations do not support a precise tail-latency claim.

**Prove:** describe the saturation point using evidence: what stops increasing,
what begins waiting, and which resource reaches its limit. Demonstrate stable
memory and explicit overload rather than an ever-growing accepted backlog.

**Break:** make the fake backend slower mid-test. Admission and memory remain
bounded, and waiting/rejection metrics explain the latency change.

**Read:** [performance](performance.md) and
[tracing spans](https://docs.rs/tracing/latest/tracing/). Save the measured
concurrent local checkpoint before adding streaming complexity.
Use [the optimization experiments](paper-review.md#optional-optimization-experiments)
to distinguish reducing context, parallel tool scheduling, and backend decoding
acceleration. They are optional follow-ups to a measured bottleneck.

## 30. Inspect stored runs and replay decisions

**Build:** add inspection and export commands backed by bounded store queries.
Validate schema version, sequence order, correlations, terminal projection, and
capture completeness. Show execution/acceptance, contract scope/version, elapsed
time, counts, errors, and the bounded final result/receipt without requiring replay capture.

Implement replay as a pure path that consumes retained normalized inputs and
observations and compares the core's proposed effects and final outcome.
Checked replay also needs the exact frozen specification, checker version, and
evidence inputs. Recompute the receipt and compare it; metadata receipts alone
cannot supply missing source bytes. Compare verdicts, evidence bindings, and
candidate/specification digests, not a newly measured checker duration.
Report verification replay unavailable when
the required capture is absent. Never substitute today's workspace or profile.
Give this path no model client, workspace capability, or executor handle.
It must be structurally unable to perform real effects.

Default metadata capture is insufficient for exact decision replay. Report that
fact directly. Opt into replay only for the synthetic examples, storing initial
inputs and subsequent message/observation deltas once. Removed payloads,
unsupported versions, or changed contracts produce an explicit replay refusal.

**Prove:** replay a complete fixture with the network disabled and workspace
absent. A deliberate core regression changes its proposed effect and is caught.

**Break:** remove one tool observation or edit a call ID in an exported fixture.
Reject the inconsistent input; do not fill gaps by running the tool again.

**Read:** [traces and replay](traces.md). Replay explains recorded decisions;
it neither recreates a model's generation nor makes interrupted work resumable.

## 31. Stream output with bounded assembly

**Build:** retain the non-streaming adapter as a tested fallback, then add
streamed model responses. Use SSE framing according to the standard, through a
small maintained decoder such as `eventsource-stream`. Bound incoming bytes
before the decoder and bound accumulated content and tool arguments.

Assemble choice and tool-call deltas by their protocol indices. Treat arbitrary
HTTP chunks as bytes, not complete JSON or SSE messages. Validate the final
assembled response and finish reason before any tool dispatch.
Terminal content displayed before completion is provisional, including while
the checker runs. Publish final acceptance only after the combined terminal
transaction commits; a streamed candidate cannot display a passed badge early.

Send text progress through a bounded output path. Coalesce or disconnect a slow
consumer according to the documented policy; never accumulate unlimited tokens.
Commit completed model observations and lifecycle events, not an fsync for every
token. Capture policy still governs retained intermediate text.
For failed streams, metadata mode stores bounded status, counts, and error
categories without partial text or arguments. Retaining bounded partial payloads
for diagnosis requires explicit replay capture.

**Prove:** test splits inside UTF-8, SSE delimiters, JSON, and tool arguments;
also test comments, empty fragments, length termination, and missing completion.
Compare time to first useful visible text with step 29's non-streaming baseline.

**Break:** disconnect after half a tool argument and saturate the display queue.
No partial tool executes, no unqualified success appears, and memory stays bounded.

**Read:** [SSE standard](https://html.spec.whatwg.org/multipage/server-sent-events.html),
[eventsource-stream](https://docs.rs/eventsource-stream/latest/eventsource_stream/),
and [streaming contract](integration.md). Passing this and the previous gates
completes the planned first local release.

## 32. Reach a private remote model

**Build:** keep tools, controller, and SQLite on their existing machine while
pointing an operator-provisioned model alias to the remote inference endpoint.
Set up OS networking or WireGuard outside Kinesin using [integration](integration.md).
There is no second Koil forwarding process in this step.

Verify binding, firewall restrictions, destination policy, and the selected
transport authentication/encryption. Do not expose an unauthenticated model
endpoint merely because its normal address is private.
Repeat the pinned text/tool/streaming fixtures over the real route.

Measure network wait separately from model generation and compare with the
local baseline. The remote model may have a different supported slot/context
capacity, so validate and clamp its configured request concurrency separately.

**Prove:** the same runner and model boundary work without core changes.
Local workspace contents reach only the explicitly authorized model destination.

**Break:** disconnect during a request and restart the remote model while a
batch is active. Outcomes remain bounded and understandable; no completed tool
is repeated automatically as a consequence of reconnecting.

**Read:** [WireGuard quick start](https://www.wireguard.com/quickstart/) and
[remote integration](integration.md). A private route does not replace the
shared-service identity and authorization work that follows.

## 33. Optionally supervise an owned model process

**Build if useful:** add managed-server mode only when manual startup is now an
actual inconvenience. Keep attach mode as the default simple path.
Kineserve owns a child process lifecycle; it does not forward model requests.

Use an argument vector rather than a shell-built launch string. Specify the
working directory, allowed environment, log destination, readiness deadline,
and expected model/template/slot configuration.
Detect early exit while waiting for readiness.

Distinguish the exact child this controller started from an attached server.
Bound log buffering and cleanup waits. Prevent child inheritance of the
controller lock, workspace handles, and unrelated service secrets.
Document the platform's process-tree cleanup behavior and crash limitations.

**Prove if implemented:** readiness succeeds; failed startup and child death
produce useful errors; orderly exit waits for the owned child. An attached
server survives harness exit.

**Break:** make the child never become ready, exit immediately, and generate
more log output than the buffer allows. Every case must terminate predictably.
If this feature is skipped, record attach mode as the supported configuration.

**Read:** [Tokio process handling](https://docs.rs/tokio/latest/tokio/process/index.html)
and [managed-server contract](integration.md). This conditional step is not a
requirement to build a shared service.

## 34. Define and implement private API ingress

**Build:** add one Axum service around the existing controller. Keep it bound to
loopback/private development access until step 41 passes. Write request/response
DTOs, bounded errors, and route tests before introducing real clients.

| Route | Initial contract |
|-------|------------------|
| `POST /v1/runs` | Validate the disjoint freeform/checked submission, admit once, persist admission, return a run ID. |
| `GET /v1/runs` | Return the caller's paginated execution/acceptance summaries. |
| `GET /v1/runs/{id}` | Return authorized outcomes, retained result, bounded receipt, and derived `task_accepted`. |
| `GET /v1/runs/{id}/events` | Stream authorized lifecycle projections, including committed terminal acceptance, using SSE. |
| `POST /v1/runs/{id}/cancel` | Request cancellation and return its current status. |
| `GET /v1/runs/{id}/export` | Export the caller's retained, bounded journal view. |

Use `202` for newly accepted asynchronous work, bounded `4xx` validation/auth
errors, and explicit overload responses. A submission cannot choose its owner,
host path, arbitrary URL, tools, or higher limits. Retention remains an operator
operation; there is no general client-delete endpoint in this release.
Reject freeform prompt/workspace overrides in checked submissions. HTTP 202
means admission and HTTP 200 retrieval; neither means task acceptance.

**Prove:** route tests exercise fake controller admission and invalid input.
Bound body size, concurrent connections/handlers, and header/read time at the
HTTP server or its trusted ingress before expensive work.

**Break:** submit a huge body and an unknown workspace/model alias. Neither
creates a run or reaches the model.

**Read:** [Axum](https://docs.rs/axum/latest/axum/) and
[body limits](https://docs.rs/axum/latest/axum/extract/struct.DefaultBodyLimit.html).
An extractor's body limit does not substitute for every ingress resource limit.

## 35. Authenticate owners with provisioned credentials

**Build:** implement the initial credential scheme in [security](security.md).
An operator provisions an opaque bearer token with at least 256 random bits;
use OS-backed randomness from a vetted crate. Deliver its secret once through
the operator workflow and keep only its cryptographic digest for verification.

Use a public token identifier to select the stored record, then verify a fixed
length digest with a vetted constant-time comparison. Derive owner identity
from that verified record. Return a uniform unauthorized result for invalid,
unknown, revoked, or malformed credentials; never log the bearer secret.

Require TLS for remote clients, either in the service's vetted TLS layer or at
a configured trusted reverse proxy. Trust identity/proxy headers only from that
specific ingress; ordinary callers cannot assert them.
Keep token configuration outside model-readable workspaces.

**Prove:** two tokens map to two owners regardless of request body fields.
Invalid/revoked tokens invoke no controller or store query carrying user data.

**Break:** send an owner field and forged proxy identity headers with another
owner's token. The verified token remains authoritative.

**Read:** [OS randomness](https://docs.rs/getrandom/latest/getrandom/),
[SHA-2](https://docs.rs/sha2/latest/sha2/),
[constant-time comparison](https://docs.rs/subtle/latest/subtle/), and
[OWASP REST security](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html).
This step does not require writing an account system, JWT issuer, or OIDC server.

## 36. Persist owner-scoped results and idempotent creation

**Build:** require verified owner context for every run query, mutation, export,
and event subscription. Bind owner and run ID in the database operation itself.
Paginate lists/events by stable cursors with bounded page counts and bytes.
An unknown or unauthorized ID returns the same not-found behavior.

For `POST /v1/runs`, require a bounded idempotency key scoped to that owner.
First make an authorized, bounded lookup for an existing key. Same key plus same
validated request returns the same run without reserving new run capacity;
different content returns `409`. Compare fingerprints, not retained plaintext
prompts. Keep ingress and lookup resources bounded even for duplicate requests.
Fingerprint the versioned requested mode/task alias, and separately freeze the
effective contract. An identical retry returns its original receipt after a
profile edit; a changed task/mode conflicts. Never rejudge a retained result on GET.

For a new key, reserve capacity and transfer the submission to a bounded
controller-owned create operation before committing. The HTTP handler waits for
its result; a client disconnect cannot abandon commit or dispatch ownership.
In one transaction, recheck the key for races, then insert the accepted run,
admission event, and fingerprint mapping. Release any unused race reservation.
Dispatch once after committed admission. Reconcile ambiguous acknowledgements.
Commit terminal status, event, bounded final result, and acceptance receipt/projection together.

Keep idempotency mappings for the configured TTL, with a minimum of 24 hours.
Never evict them early to meet a storage cap; reject new admission when the
retention promise cannot be met.

For SSE, subscribe to bounded notifications before reading durable catch-up
pages. Notifications only wake fresh owner-scoped cursor queries; they are not
the journal. Close each read transaction before waiting. Stream whitelisted
status projections, never raw stored event payloads. Optional provisional
`text_delta` events have no durable ID or reconnect promise and exclude tool
arguments and private provider fields.

Hold observer permits for each stream's lifetime: at most 16 globally and two
per run, plus bounded per-observer event count and bytes. Close a stream once
its terminal status and acceptance receipt projection have been caught up; do not
keep terminal-run watchers alive. Expose only the bounded authorized public
receipt fields, never raw evidence bodies from replay capture.

**Prove:** race duplicate submissions and verify one accepted run and one
dispatch, including when the submitting HTTP client disconnects. A known-key
retry still succeeds when new-run capacity is full. Test all routes as owner A
against owner B's run.
Also retry after editing a task profile: the original run and receipt remain
unchanged. Reject an added prompt or a different task under the same key.

**Break:** lose the create response, retry with the same key, and reconnect an
SSE reader from its last durable sequence. No duplicate run or cross-owner event
appears. Slow readers are bounded and can disconnect/reconnect explicitly.

**Read:** [storage contract](traces.md) and
[Axum SSE](https://docs.rs/axum/latest/axum/response/sse/index.html).

## 37. Give each owner bounded and fair capacity

**Build:** add per-owner active/queued limits and queue-byte accounting beneath
the existing global bounds. Use the starting settings in [performance](performance.md).
Add a scheduler policy such as round-robin among owners with eligible work.
Per-owner fairness must apply where scarce model capacity is assigned as well.

Keep admission, scheduling, and release under a single understandable owner or
small explicitly synchronized component. A run finishing or cancelling must
return all relevant reservations once. Limit the number of owner queue records
and remove idle entries so random submissions cannot grow a registry forever.

Return a specific bounded overload response when an owner or the service is
full. Waiting indefinitely is not a fairness policy.
Do not keep retrying model generations as an overload strategy.

**Prove:** flood owner A while B sends occasional short tasks. B receives
capacity under the documented policy, and every owner/global limit holds.
Measure each owner's wait and completion distributions separately.

**Break:** cancel A's queued work repeatedly while B runs, then revoke A's
credential. Reservations and queued work follow the documented revocation rule
without affecting B's authority or event stream.

**Read:** [performance](performance.md) and [Tokio channels](https://tokio.rs/tokio/tutorial/channels).
Write down which queues are fair; a FIFO semaphore alone is not a complete
multi-owner scheduling contract.

## 38. Harden the shared filesystem and service process

**Build:** provision owner/workspace/model mappings administratively.
Each run receives only its owner's selected `WorkspaceReader`; there is no
route for callers to open arbitrary host paths or supply capability objects.
Keep service state, credentials, backups, and logs outside every tool root.

Provision controlled workspace trees with the link and special-file restrictions
in [security](security.md). Test that owner A cannot select owner B's alias.
Use a dedicated OS service identity with only required filesystem/network rights,
explicit credential handling, restricted model egress, and resource limits.

The initial shared service executes trusted compiled read-only handlers in one
controller process. Capability discipline limits those handlers' access; it is
not process isolation against compromised Rust/native dependencies or arbitrary
code. Do not add shell, user plugins, or executable uploads under this gate.

**Prove:** run the cross-owner sentinel-file tests on the actual deployment OS,
including supported link cases. Check the deployed service identity cannot
access unrelated operator credentials and cannot contact forbidden destinations.

**Break:** place a confusing alias, forbidden link, or special file in a test
workspace through the operator fixture setup. Provisioning and execution must
reject it with bounded behavior under the chosen profile.

**Read:** [security](security.md). Arbitrary-code tools require a separately
designed worker sandbox, isolation tests, and a new exposure gate.

## 39. Recover interruption without inventing execution history

**Build:** hold the exclusive controller process lock before recovery.
In a recovery transaction, mark previously queued/running unfinished runs
interrupted and append the documented recovery event. Checked unfinished runs
become acceptance-inconclusive; freeform stays unchecked. Preserve terminal runs
and their combined results/receipts. Do not automatically requeue, reassess, or
resume interrupted work.

For committed effect intent lacking its observation, record uncertainty.
The tool or HTTP request might have completed before the crash.
Metadata capture intentionally lacks the full inputs needed for replay; even
full replay capture would not establish whether an external effect happened.

Handle graceful shutdown and process death separately. On graceful shutdown,
stop new submissions and follow step 27's drain procedure. After process death,
SQLite recovery plus the startup transaction establishes a coherent state view;
it does not reconstruct facts that were never committed.

**Prove:** terminate a test controller after acceptance, after effect intent,
and after checking but before terminal commit. On restart, no run silently starts
or passes from an uncommitted assessment. Every previously committed result and
receipt remains readable by its owner, without reading today's workspace.

**Break:** start a second controller against the same state directory while
the first is live. It must fail to acquire the lock and must not alter runs.

**Read:** [recovery contract](traces.md) and
[exclusive file locking](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock).
Use disposable test state; this exercise is never run against useful work.

## 40. Operate retention, backups, readiness, and overload

**Build:** add bounded operator retention for completed runs, replay payloads,
results, and idempotency records, with explicit active-run exclusions and TTLs.
Honor the idempotency minimum from step 36, including deletion through related
run rows. A storage cap must reject admission rather than shorten that promise.
Measure database/WAL/disk growth. Store and export metadata capture defaults,
but treat final results as sensitive retained content in every mode.

Implement health and readiness separately. A live process can be unready because
admission is closed, configuration is invalid, storage is unavailable, or the
required model cannot serve. Bound checks and keep them from consuming all model
slots or exposing credentials.

Back up through SQLite's supported backup mechanism and test restoring to a new
state directory. Do not copy only the main database file while ignoring active
WAL state. Keep backup reads/maintenance bounded so they cannot starve journal
commits; an offline backup after controlled shutdown is a valid starting mode.

**Prove:** restore owner-scoped completed results and events, run integrity and
schema checks, and start under the process lock with the documented recovery.
Record recovery time, storage growth, queue wait, and admission failures.

**Break:** simulate full/unwritable storage, unavailable model, slow journal
acknowledgements, and a slow export subscriber. No path reports durable success
without its required commit or buffers an unbounded backlog.

**Read:** [SQLite backup](https://www.sqlite.org/backup.html),
[SQLite integrity checks](https://www.sqlite.org/pragma.html#pragma_integrity_check),
and [operations and limits](performance.md).

## 41. Pass the shared-service exposure gate

**Build:** collect the evidence from steps 34–40 into a deployable runbook:
exact toolchain/dependencies, SQLite engine, model/template/slot settings,
OS/filesystem assumptions, TLS ingress, operator credentials, aliases, limits,
retention, backup restore, shutdown, rollback, and known restrictions.
Keep one controller and one local SQLite state directory per deployment.

Run a matrix covering both owners and every route: create, list, fetch, events,
cancel, and export. Add overload, slow bodies/readers, revoked credentials,
idempotent retries, model disconnects, process crashes, full storage, and restore.
Include wrong-but-fluent checked candidates, forged evidence, checker faults,
unchecked runs, and profile edits. Verify every public outcome exposes the same
persisted receipt and cannot manufacture `task_accepted` from final text or HTTP codes.
Use the measured concurrency and latency objectives from [performance](performance.md).

Re-run the offline suite and the pinned live text/tool/streaming checks.
Review the security boundary and residual risks against the actual deployed
configuration. A changed proxy, filesystem, model server, or execution capability
can invalidate earlier evidence and must receive its own relevant checks.

**Prove:** another operator can follow the runbook to start, verify, stop, and
restore the service. Cross-owner data access fails, accepted work stays bounded,
completed results and receipts survive restart, and overload fails promptly and observably.

**Break:** rehearse rollback using synthetic data and the restored database.
Explain which newer schema/data changes prevent reverting just the executable.
Do not expose the service until the chosen profile's required proofs pass.

**Read:** [security](security.md), [testing](testing.md), and [process](process.md).
This gate supports the specific tested shared service. Horizontal controllers,
arbitrary-code tools, automatic resume, and autonomous child-agent spawning
each require a new design and their own evidence.
