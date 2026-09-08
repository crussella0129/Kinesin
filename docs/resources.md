# Reference index

The [build guide](build-guide.md) links readings at the step where they become
useful. Use this page to find them again, not as a reading assignment to finish
before writing code.

## Rust foundations

- [The Rust Book](https://doc.rust-lang.org/book/): ownership, enums, errors,
  modules, tests, then async when the guide reaches it.
- [Brown ownership-error explanations](https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html):
  an additional explanation after encountering an actual borrow/move error.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/): small focused
  examples for an unfamiliar standard-library operation.
- [Cargo Book](https://doc.rust-lang.org/cargo/): package targets, dependencies,
  lockfiles, tests, and eventual workspaces.
- [Clippy usage](https://doc.rust-lang.org/clippy/usage.html): warnings as useful
  feedback while learning idiomatic Rust.
- [Rust File locking](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock):
  retained ownership of the controller state directory.

## Async runtime and network boundary

| Reference | Look for |
|-----------|----------|
| [Tokio tutorial](https://tokio.rs/tokio/tutorial) | Runtime, owned tasks, channels |
| [Spawning](https://tokio.rs/tokio/tutorial/spawning) | `Send`, `'static`, task ownership |
| [Channels](https://tokio.rs/tokio/tutorial/channels) | Bounded coordination and backpressure |
| [Shutdown](https://tokio.rs/tokio/topics/shutdown) | Detect, notify, wait |
| [Blocking work](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) | Actual closure lifetime and cancellation limits |
| [reqwest Client](https://docs.rs/reqwest/latest/reqwest/struct.Client.html) | Reused pooled connections |
| [ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html) | Explicit timeout, redirect, proxy, retry policy |
| [SSE standard](https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream) | Incremental event framing |
| [Axum](https://docs.rs/axum/latest/axum/) | Later authenticated service boundary |
| [Tower layer ordering](https://docs.rs/tower/latest/tower/struct.ServiceBuilder.html#order) | How limits and buffers compose |

## Data, capabilities, and persistence

- [Serde](https://serde.rs/), [serde_json](https://docs.rs/serde_json/latest/serde_json/),
  [TOML](https://docs.rs/toml/latest/toml/): typed formats rather than private parsers.
- [cap-std](https://github.com/bytecodealliance/cap-std) and
  [Dir](https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html):
  capabilities and their security limits.
- [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/),
  [transactions](https://www.sqlite.org/lang_transaction.html),
  [WAL](https://www.sqlite.org/wal.html),
  [synchronization](https://www.sqlite.org/pragma.html#pragma_synchronous),
  [backup](https://www.sqlite.org/backup.html): commit, query, and recovery contracts.
- [tracing](https://docs.rs/tracing/latest/tracing/): structured redacted diagnostics.
- [Security](security.md): authoritative credential, authorization, and platform
  references for the later service.

## Model boundary and evaluation

- [llama.cpp server](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
  and [function calls](https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md):
  read the pinned version's contract, not only current `master`.
- [llama.cpp builds](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md):
  native Windows/Linux backend setup.
- [ReAct](https://arxiv.org/abs/2210.03629): the action/observation loop idea.
- [SWE-agent](https://arxiv.org/abs/2405.15793): why the model-facing interface is
  worth evaluating, without transferring benchmark percentages to Kinesin.
- [Open versus closed load](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/open-vs-closed/):
  why overload tests should control arrival rate.
- [Task acceptance](verification.md): frozen requirements, actual evidence,
  deterministic field checks, result receipts, and explicit unchecked outcomes.
- [Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents):
  outcome-based evaluation and complementary grader types.
- [Adversarial attacks on LLM-as-a-judge systems](https://arxiv.org/abs/2504.18333):
  why evaluator prompts and model verdicts create another attack surface.

## Paper-guided improvements

The [paper review](paper-review.md) maps the supplied PDFs and related research
to concrete decisions, limitations, and experiments. Read the relevant section
alongside the guide: ICM for context selection, ReAct for feedback, PDDL-INSTRUCT
and PlanBench for independent checks, and OoO-Spec/ToolSpec/LLMCompiler for the
different optimization boundaries. Lost in the Middle and AgentDojo inform
context-quality and hostile-input evaluations.

## Later integrations

- [WireGuard](https://www.wireguard.com/quickstart/): an OS network route to remote
  inference, separate from API ownership.
- [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools):
  an external adapter with its own trust, version, and authorization requirements.
- [Rust process](https://doc.rust-lang.org/std/process/struct.Child.html):
  optional owned server lifecycle.
- [WSL networking](https://learn.microsoft.com/windows/wsl/networking):
  only if a deployment crosses that OS boundary.

Use the exact versions selected during implementation. A reference to a moving
documentation page is not a lockfile or a guarantee that every option exists in
an older executable.
