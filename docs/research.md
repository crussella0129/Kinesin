# Research and design review

Reviewed **2026-09-08 UTC**. The repository is documentation only. No harness
implementation, security deployment, local-model benchmark, or latency result
was produced by this review.

The author authorized a complete redesign and clarified the destination:
**concurrent agents for one operator, then a shared service**. Security, low
latency, scalability, and minimality are equal goals.

## Review checklist

- [x] Read the existing and original checked-in plans.
- [x] Clarify the intended form of scalability.
- [x] Research concurrency, capability boundaries, storage, and provider behavior.
- [x] Reconcile the architecture and write the full build guide.
- [x] Give evidence-based feedback without attributing assistant text to the author.
- [x] Finish independent contract review and validate examples/references.

## Resulting recommendation

The subsequent [paper improvement pass](paper-review.md) reviews the four supplied
PDFs and closely related primary research. It retains this runtime architecture,
adds explicit context provenance and feedback tests, and separates inference
speculation from permission to execute tools.

The later [adversarial pass](adversarial-review.md) found that this distinction
was still only prose and offline evaluation. It adds per-run task contracts,
independent bounded checks, separate execution/acceptance outcomes, and atomic
receipts. [Task acceptance](verification.md) is the authoritative current contract.

Keep one controller package with a pure owned-state core. Introduce Tokio and
pooled async HTTP when networking begins. Admit work through explicit capacity
and byte limits. Give tools narrow directory capabilities. Store events and
run projections transactionally through one SQLite owner. Add per-owner
authorization and scheduling before exposing a service.

These choices are engineering judgments for this workload and learning goal.
The sources establish individual behaviors and limits; they do not prove that
this unbuilt combination meets its targets. [Decisions](decisions.md) records
alternatives and costs.

## Findings that changed the design

| Finding | Evidence and implication |
|---------|--------------------------|
| Async is useful for overlapping independent I/O, not creating inference capacity | [Tokio tutorial](https://tokio.rs/tokio/tutorial) supports an async shell around a synchronous core; model slots remain separate |
| HTTP client reuse avoids needless connection setup | [reqwest Client](https://docs.rs/reqwest/latest/reqwest/struct.Client.html) provides an internal shared pool and cheap clones |
| Unconfigured HTTP behavior can violate intended policy | [ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html) and [retry::never](https://docs.rs/reqwest/latest/reqwest/retry/fn.never.html) motivate explicit deadlines, redirect/proxy policy, and no retries |
| Bounded channels alone do not bound all work | [Tokio channels](https://tokio.rs/tokio/tutorial/channels) motivates bounding admitted tasks, queue bytes, and waiting producers as well |
| Started blocking work cannot simply be aborted | [spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) requires permits to live with actual work, not only an async waiter |
| Shutdown has several responsibilities | [Tokio shutdown](https://tokio.rs/tokio/topics/shutdown) separates detection, notification, and waiting |
| Capability filesystem access is stronger than a check/open path recipe | [cap-std](https://github.com/bytecodealliance/cap-std) supplies handle-relative authority but explicitly does not sandbox arbitrary Rust code |
| Transactions can keep events and status coherent | [SQLite transactions](https://www.sqlite.org/lang_transaction.html) and [WAL](https://www.sqlite.org/wal.html) replace a custom multi-file commit protocol |
| Durability and latency have a real tradeoff | [SQLite synchronous](https://www.sqlite.org/pragma.html#pragma_synchronous) defines what FULL versus NORMAL means; benchmark under the stated choice |
| Service ownership is an authorization decision | [OWASP authorization](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) supports checks on every resource operation, not only submission |
| TLS bearer authentication is a concrete small-service option | [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750.html#section-5) supports protected bearer-token transport; the chosen provisioning policy is specified in [security](security.md) |
| Stream chunks are not complete protocol messages | [SSE parsing standard](https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream) requires incremental framing before provider/tool assembly |
| Closed load tests can conceal queueing | [Grafana open/closed models](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/open-vs-closed/) motivates fixed-arrival saturation tests as well as ordinary batches |

Primary sources are linked beside the claim family they support. No universal
performance or model-quality percentage is inferred from them.

## Provider facts retained from the first review

Use the pinned llama.cpp chat API and complete tool-call/result history.
Current response code uses `finish_reason: "tool_calls"`; `length` is incomplete.
Require valid IDs/structure before executing any call.
[Response serialization](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/server-task.cpp)

Explicit custom grammar with active tools is rejected in current request parsing,
and other structured-output behavior varies with templates. Start with tools
alone; test separate tools-disabled structured output if needed.
[Request parsing](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/server-common.cpp),
[template handling](https://github.com/ggml-org/llama.cpp/blob/master/common/chat.cpp)

Explicitly disable context shifting. Oversized input can produce a context
error; capacity/output exhaustion during generation can produce `length`.
Neither permits tool execution. The actual slot context depends on the tested
server configuration; do not assume a universal division formula.
[Context implementation](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/server-context.cpp),
[server documentation](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

Where the pinned backend supports the chat input-token counting extension,
count the complete request/template/tools and verify against usage fixtures.
The extension is not a generic OpenAI-compatible guarantee.
[Token-counting endpoint](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md#post-v1chatcompletionsinput_tokens-token-counting)

## Version and platform checks that matter

Moving `latest` and `master` links are references, not a deployment lockfile.
Record the actual Rust/dependency/server/model versions and rerun compatibility
checks on upgrades.

- SQLite documents a WAL-reset fix in 3.51.3 and fixed backports. Check the actual
  engine linked by `rusqlite`, rather than assuming a crate version proves it.
  [SQLite WAL version note](https://www.sqlite.org/wal.html)
- `cap-std` documents a previous Windows device-path bypass fixed in 3.4.1.
  Use a maintained pinned release and exercise platform path cases.
  [Project advisory](https://github.com/bytecodealliance/cap-std/security/advisories/GHSA-hxf5-99xg-86hw)
- Rust's standard-library file locks, including `try_lock`, are available from
  1.89. Retain a controller lock before startup recovery.
  [Rust File](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock)
- Native Windows llama.cpp builds are supported; WSL is a choice rather than a
  harness prerequisite. Avoid crossing OS boundaries until necessary.
  [Build guide](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
- Landlock, AppContainer, and Job Objects solve different parts of execution
  confinement/lifecycle. They are later workload-specific work, not implied by
  Rust or capability wrappers. [Security](security.md) links the platform sources.

## Why no fixed model recommendation

Choose a tool-capable model that fits the actual machine, then evaluate it on
Kinesin's tasks. A parameter-count floor is not an engineering guarantee.
The official Qwen3-4B card already documents agentic use below the earlier 8B rule;
that does not prove reliability for this workload.
[Qwen3-4B](https://huggingface.co/Qwen/Qwen3-4B)

Keep a baseline with model artifact, quantization, template, context, sampling,
server build, hardware, and task set fixed. Change weights/cache/slots separately.
The llama.cpp guide warns about extreme KV-cache quantization, without making
all other settings universally harmless.
[Tool-calling guide](https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md)

## Experiments that can make Kinesin better

1. Improve tool descriptions, error messages, and bounded result formatting.
   Compare the same task set rather than assuming more tools help.
   Agent-interface research motivates this experiment without predicting
   Kinesin's gains. [SWE-agent paper](https://arxiv.org/abs/2405.15793)
2. Compare one, two, and four concurrent runs/model calls on a fixed backend.
   Measure successful tasks/minute alongside p95 latency and memory.
3. Compare nonstreaming completion with time to useful streamed text. Tool
   readiness still waits for a complete validated batch.
4. Measure journal acknowledgement latency under FULL synchronization before
   trying bounded group commit. Keep correctness/durability unchanged.
5. Exercise one busy owner beside another with short tasks. Check admission and
   model fairness, not only a global semaphore count.
6. Add context compaction, parallel independent tools, MCP, or bounded child
   agents only after an observed task need. Each adds a specific policy and
   failure contract.

Provisional performance targets are in [performance](performance.md).
Harness tests and model evaluations are separate in [testing](testing.md).

## Review scope and limits

Three independent review assignments covered security, concurrency/latency, and
the teaching sequence. Reconciliation fixed acceptance ownership across client
disconnects, duplicate retries under saturation, subscription/catch-up races,
journal reservation lifetimes, global observer capacity, capture privacy,
idempotency retention, and bookkeeping after execution deadlines.

The initial redesign's documentation checks passed across 18 Markdown files: 194 local links and
anchors, 40 ordered guide steps and matching roadmap entries, balanced fences,
and a build/proof/failure/reading section in every step. Parsed all three JSON
examples (including nested tool arguments), both TOML examples, and the proposed
SQL schema. In-memory SQLite checks confirmed owner-scoped key uniqueness and
atomic rollback of status/event changes. `git diff --check` passed. The 110
distinct external reference URLs are a reading index, not a claim that every
external URL was exhaustively link-tested.

No Cargo package exists, so Rust formatting, Clippy, runtime tests, live model
compatibility, and security/load experiments remain implementation work. The
schema/example checks above do not establish deployed runtime behavior.

The review checked the plan, primary documentation, proposed contracts, and
teaching sequence. It did not assess the author's coding ability from documents.
[Understanding](understanding.md) distinguishes evidence from inference and
acknowledges where my earlier recommendations needed changing.

The first shared service is one trusted controller with provisioned inputs and
trusted compiled read-only handlers. It is not a hostile-code hosting platform,
a distributed exactly-once worker system, or a certified secure deployment.

Before editing, the uncommitted preceding draft was copied to
`C:\Users\charl\AppData\Local\Temp\Kinesin-plan-20260907-221613`.
The original checked-in plan also remains available in Git history.
