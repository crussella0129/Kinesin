# Verification strategy

The runtime must behave correctly even when the model produces a bad answer.
Use deterministic fakes for invariants, real-server compatibility fixtures for
the adapter, and task evaluations for model quality. Measure performance under
the same policy and durability settings you intend to ship.

## Test layout

Use unit tests beside private logic and integration tests through the library's
small public API. Keep `main.rs` thin. Introduce fake dependencies where a test
needs them; do not create a general mocking framework.

The synchronous core consumes events. The async scripted model records prepared
requests and returns configured replies, failures, delays, or stream fragments.
A controllable storage boundary can fail before or after acknowledgement.
Instrument tool starts/completions and held permits to detect unintended effects.

[Rust test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html),
[Tokio testing time](https://docs.rs/tokio/latest/tokio/time/fn.pause.html)

## Core and protocol invariants

| Case | Assertion |
|------|-----------|
| Ordinary freeform answer | Completed + unchecked, task_accepted false, strict exit 3; no tool effects |
| Empty/unknown/contradictory response | Explicit failure, no tool effects |
| `length` finish | Incomplete stop; no partial calls execute |
| Tool batch | IDs unique within batch; preserve assistant message and one result per call |
| Invalid/unknown/denied tool | Budget consumed, planned/finished pair recorded, handler never runs |
| Over-budget batch | No handler from that batch runs |
| Repeated normalized batch | Stop before the threshold batch, independent of IDs/JSON whitespace |
| Terminal state | No later model/tool dispatch |
| History growth | Bounds include results/encoding; complete call/result groups remain intact |
| Prepared request | Recorded fingerprint matches the exact body passed to HTTP |
| Captured data | Metadata omits intermediate content; replay retains required inputs; final answer retention is explicit |

JSON/TOML libraries handle format conformance. Your tests handle application
meaning, unsupported fields, budgets, correlation, and safe failure behavior.

## Per-run task acceptance

Test the pure `FileFieldsV1` checker separately from the model. Then feed scripted
model/tool observations through the real runner and terminal transaction. These
are runtime invariants for every applicable run, beyond offline model evals.
The authoritative outcomes and bounds are in [task acceptance](verification.md).

Start with a complete source `language=Rust` and exactly one required `language`
criterion. A candidate with the right evidence ID but `value=Python` must reach
`completed + failed`, not pass merely because the file was cited. A candidate
with `value=Rust` can pass only when the required actual source observation and
entire typed output contract validate.

| Attack or failure | Required assertion |
|-------------------|--------------------|
| Model says it passed; no selected task contract | Freeform remains unchecked |
| Empty/duplicate criteria or unknown checker/version | Reject before admission; no vacuous pass |
| Arbitrary prompt attached to checked task | Reject mixed submission shape; no weak check for an unrelated goal |
| Allowed task referencing forbidden workspace/tool/model | Reject admission; task permission does not grant resource access |
| Wrong value with genuine evidence | Failed exact-value check |
| Unknown, forged, other-run/owner, wrong-resource evidence ID | No foreign lookup or authority; failed evidence binding |
| No read, error containing expected value, incomplete source, missing key | No usable complete source; never pass. Source uncertainty is inconclusive; an additional forged-reference failure takes precedence |
| Complete observations conflict or source has duplicate keys | Inconclusive; no cherry-picking |
| Source edited after read | Compare observed bytes; never claim current-at-finish or reread |
| Revision constraint differs | Fail declared revision check |
| Extra prose, unknown/duplicate JSON fields, extra/missing/duplicate facts | Fail whole output contract, even with one correct field |
| Oversized candidate / evidence-cap exhaustion | Failed candidate check / explicit execution stop with inconclusive acceptance |
| Checker fault or partial successful checks | Inconclusive fault; no partial pass |
| Cancellation/deadline during checking or terminal-inbox wait | Before submission, execution cancellation/stop and inconclusive acceptance |
| Cancellation after terminal command enters inbox | Settle one submitted transaction; never overwrite it with another terminal write |
| Crash between checking and terminal commit | Interrupted + inconclusive on recovery, no rejudge or retry |
| Receipt/candidate/event write fails | Roll back all; no published accepted result |
| SSE or GET races checking | No passed projection until the combined terminal commit |
| Same idempotency key after profile edit | Return original run/receipt; changed requested mode/task conflicts |
| Imported receipt claims passed | Untrusted input; cannot install an accepted live verdict |

Use independent hand-labeled positive and negative checker fixtures. Mutate a
passing case by changing its answer value, source value, evidence identity,
completeness, owner, required criterion, or finalization point. Each meaningful
mutation must invalidate the expected acceptance or change its documented scope.
Test source LF/CRLF, exact whitespace/case, first-`=` parsing, invalid UTF-8,
blank/comment lines, absent keys, duplicate keys, and malformed non-comment lines.
Do not generate expected answers using the same parser being tested.

Keep rubric review separate from implementation tests: prove the checker rejects
known wrong answers, then check that its declared contract covers the intended
human task. A correct checker of a weak requirement is still insufficient.
See the [adversarial review](adversarial-review.md) for findings and residual risk.

## Filesystem tests

Use isolated fixtures: text with multibyte characters, missing files, a large
file, a large directory, similar sibling directory names, and inaccessible paths.
Test parent/rooted/drive-relative/UNC/device/alternate-stream syntax and escaping
links through the actual capability implementation on supported platforms.

Verify regular-file type on the opened handle, bounded reads/collection, valid
UTF-8 truncation, serialized envelope limits, and error codes. A truncated
directory result must not claim completeness or deterministic membership.
Keep private state/credentials outside all tool roots.

Test file content that requests a forbidden action. The model may obey or ignore
that text; the policy must deny the forbidden effect either way. A model evaluation
is not a replacement for directly feeding a forbidden call to the runner.

Report unavailable platform tests honestly. Do not claim a hostile-filesystem
sandbox from tests of a curated input tree. Named-pipe/device blocking and hard
links have the declared limitations in [security.md](security.md).

## Async ownership and overload tests

- Interleave run A and B: responses, cancellation, history, policy, and results
  never cross.
- Count started/finished model operations and blocking jobs; maximum held capacity
  never exceeds the configured limits.
- Submit beyond admission capacity. Bound task count, queue count, and aggregate
  retained bytes, including inputs waiting to be admitted.
- Cancel while queued, waiting for a model permit, awaiting an HTTP body, waiting
  for a journal acknowledgement, and running a blocking file closure.
- Exhaust the execution deadline before terminal recording. The one-time
  settlement grace allows bookkeeping, never more model/tool work. Grace expiry
  retains ownership/capacity for unresolved commits or blocking work.
- Do not release a blocking permit when only its async waiter was cancelled.
  Keep observing the real closure until it exits.
- A slow journal blocks further effects and eventually admission under its
  explicit policy; it does not grow an unbounded channel.
- A slow SSE subscriber is disconnected/resynchronized; it cannot block inference
  or durable events.
- Shutdown closes admission, signals runs, drains/settles owned work, and joins
  task handles. A timeout reports unresolved work instead of claiming it vanished.

Tokio documents that started blocking tasks cannot be aborted; that behavior
belongs in a failure exercise, not only a footnote.
[spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

## HTTP and streaming fixtures

Use a small bounded local fixture server or a narrowly selected test helper
dependency. Do not build a general HTTP server as a prerequisite for the client.
Known requests can use bounded headers and Content-Length reads; test helpers
need their own timeout and cleanup.

Cover non-success statuses, oversized bodies, invalid JSON, redirects, proxy
environment settings, stalls, connection failure, and explicit zero automatic
generation retries. Verify actual request count at the fixture server.

For SSE, split a valid example at every byte boundary, including UTF-8 characters
and event delimiters. Cover multiple events in one network chunk, incomplete
frames, interleaved tool-call indices, duplicate/conflicting IDs, oversized
arguments, incomplete EOF, `length`, and lost connection. Tool execution counts
remain zero until a complete validated terminal batch exists.

Display text is provisional. Do not test only time to the first network byte and
call it time to useful text. The [performance](performance.md) definitions govern
measurements.

## Storage tests

Use isolated temporary databases. Exercise the actual DDL and parameterized
queries in addition to an in-memory fake journal.

- Duplicate run/sequence/owner-scoped idempotency constraints.
- Candidate/result, acceptance receipt/projection, and terminal event commit or
  roll back together; completed always has a receipt, terminal never pending,
  and acceptance passed requires execution completed.
- Unknown schema versions fail before work begins.
- Second controller cannot acquire the same state lock.
- Writer admission count/bytes and timeouts remain bounded.
- Dequeue and cancelled waiters do not release a journal command's count/bytes
  before its actual completion or rejection.
- Event too large is rejected without incomplete replay data.
- Commit error closes admission and prevents the next external effect.
- Cancelled acknowledgement wait does not pretend an in-progress transaction
  was rolled back.
- Restart marks queued/running/cancelling runs interrupted with pending effects
  unresolved, checked acceptance inconclusive, and freeform unchecked, without rerunning them.
- Completed owner-scoped results remain retrievable after restart.
- Export and backup/restore preserve ownership, schema, and required data.
- Retention pressure never deletes a run inside its promised idempotency window;
  new admission fails when necessary to preserve that promise.

Inject failures before intent commit, after commit/before send, after effect,
and before outcome commit. These locations explain why a durable journal does
not imply exactly-once effects. Power-loss durability is a storage/environment
property requiring separate verification; an ordinary process-kill test is not
the same experiment.

## Effect-free replay

Replay captures must include versions, starting state, normalized observations,
and relevant time/cancellation inputs. Compare core decisions and prepared-body
fingerprints, then final outcome/counters. Install dependencies that fail if any
real network/tool effect is attempted.
For checked tasks also require the frozen specification, exact candidate and
actual evidence inputs. Compare semantic verdict, required checks and bindings;
remeasured checker duration need not equal the original duration. A metadata
receipt alone cannot reproduce verification. Replaying an imported capture does
not authenticate its origin or turn its claimed pass into a live accepted run.

Reject metadata-only, redacted, missing, incompatible, or corrupt inputs. A replay
failure must not automatically switch to live generation. Keep a few small
understandable golden captures, not thousands of opaque fixtures.

## Shared-service acceptance

Use two owners with deliberately different roots and allowed model aliases.
For every endpoint, test missing credentials, invalid credentials, expired/revoked
credentials, correct owner, and wrong owner with the correct run ID.

Test direct backend/model-port exposure, attempted request-supplied owner/root/URL,
oversized bodies/headers, idempotency retry/conflict, queue expiration, quota
exhaustion, and fairness while one owner stays busy. Public status/SSE projections
must not expose replay payloads that require export permission.

Bound listing/export pages and stream attachment count even for old completed
runs. Client disconnect leaves an accepted run owned by the service; explicit
cancellation is separately authorized. Restart interruption and result retrieval
must follow the same owner checks.

Race two first submissions with the same owner/key; prove one acceptance and
dispatch. Repeat a known key while new-run admission is full. Disconnect the
HTTP client between commit and dispatch and prove controller ownership persists.
Attach SSE while the terminal event commits between subscription and catch-up;
deduplicate by durable cursor and recover from coalesced wakeups. Prove the
global observer cap holds across different retained runs and terminal streams
close after catch-up. Failed provisional streams never add text to metadata.

No live shared-service exposure until the full gate in
[the build guide](build-guide.md) passes on the selected deployment OS.

## Model evaluation and latency

Create small harmless tasks with expected facts: answer directly, list files,
read one fact, compare two files, handle missing/large files, deny an outside path,
and ignore hostile instructions without expanding authority.

Record task success counts, invalid/extra calls, reasons, durations, token usage
when available, model/server/template/hardware, and configuration. Repeat
model-sensitive cases; exact wording is not an assertion of harness correctness.

For runtime overhead, use a scripted provider and real selected storage policy.
For saturation, use fixed arrival rates as well as ordinary bounded batches.
Mix short and long tasks. Report p50/p95 and sample size, successes, failures,
rejections, queue time, peak RSS, and the measurement window.
[Performance plan](performance.md) supplies provisional targets and workload rules.

## Context and feedback evaluations

Before a live evaluation, add these controlled cases to the synthetic task set:

| Case | Harness assertion | Goal-quality check |
|------|-------------------|--------------------|
| New fact determines next action | First observation enters the next request intact | Next call uses the discovered filename, not a guessed one |
| Missing file with a valid alternative | Bounded correlated error; no automatic effect retry | Model changes to an allowed discovery/read action |
| Instruction-looking workspace file | No change to owner, tools, capture, endpoint, or budgets | Task still answered from permitted evidence |
| Fluent answer without required evidence | Checked task is inconclusive/failed as specified; freeform unchecked | Neither becomes accepted merely because execution completed |
| Required fact at start/middle/end | Same valid input and context bounds | Record sensitivity on the chosen model; do not assume a universal curve |
| Config edited after acceptance | Initial context snapshot and replay remain unchanged | A separately admitted run uses the intended new version |

For context comparisons, first hold source text and source order constant while
changing labels/grouping, then separately vary order, distractors, and selected
input size. Record exactly what changed. Treat a required source that was omitted
as a failed task setup or selection outcome, never a faster successful answer.
Do not silently trim an admitted conversation to pass this experiment.

Use small explicit task cards with inputs, permitted effects, expected facts, and
an independent checker or documented human rubric. Formal action preconditions
can be tested in pure Rust fixtures; a PDDL parser, fine-tuning job, or model
self-verification loop is not necessary. Repeat model-dependent cases and report
sample sizes, correctness, policy violations, bytes/tokens when available, turns,
and latency. See [paper evidence and limits](paper-review.md).

Use paired fixtures with renamed files/entities, reordered irrelevant entries,
changed fact values, and evidence beyond the read truncation boundary. Keep an
unseen set for reporting. Missing evidence should produce a qualified answer or
task failure, not an invented fact. Validate any borrowed demonstration against
its own rules before making it a golden fixture.

## Daily checks

Once Rust exists:

```text
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

Use `cargo fmt` while editing. Run affected tests during a coherent change, then
the offline suite at checkpoints. CI starts with the first package, adds platform
coverage as tools/processes arrive, and never silently downloads a model.
Real-server tests are explicitly opt-in. This repository currently contains
documentation, so these are future acceptance checks.
