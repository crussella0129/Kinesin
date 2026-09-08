# Kinesin

A small Rust runtime for agents with explicit authority and bounded resources.

Kinesin gives a model context, interprets its proposed tool calls, decides which
may run, records outcomes, and controls how a run ends. It also controls how many
runs may compete for model, filesystem, memory, and storage resources.

**This branch is the guide.** It holds the design contracts and a numbered build
sequence, and no implementation. You write the Rust yourself, in a separate empty
directory. Nothing here is written out as Rust source for you to copy.

**Start at [the build guide](docs/build-guide.md#before-you-start).** It takes you
from an empty directory to a working runtime in 41 steps. Every step states what
to build, how to prove it, which failure to exercise, and what to read.

## The two branches

| Branch | What it holds |
|--------|---------------|
| `main` | The guide and the contracts. No source. Build against this. |
| [`answer-key`](https://github.com/crussella0129/Kinesin/tree/answer-key) | A complete working implementation with green CI, and the evidence from validating this guide against it. |

Use `answer-key` when you are stuck, or to compare an approach after you have
attempted a step. Reading it before you attempt a step removes the reason to
build this yourself.

## What you will build

An execution runtime around a local model. Its first complete release accepts a
bounded batch of independent tasks, runs them concurrently, and gives each task
its own conversation, workspace authority, budgets, cancellation, and result. It
begins with two read-only file tools and one llama.cpp adapter.

```text
CLI / later authenticated API
             |
      admission + scheduler
             |
      one runner per run ----------> approved model endpoint
        |          |                   via pooled async HTTP
        |          +-> capability-scoped file tools
        +-> bounded storage inbox -> SQLite owner thread
```

Execution completion and task acceptance are separate outcomes. A final answer is
a candidate: scoped tasks pass only after an independent checker verifies their
frozen contract. Freeform answers stay explicitly unchecked. That distinction is
the point of [task acceptance](docs/verification.md), and the
[adversarial review](docs/adversarial-review.md) explains the false passes it
prevents.

The original names survive as roles rather than as separate daemons: **K-Core**
is the pure decision logic, **Koil** is the model adapter, and **Kineserve** is
optional model-process supervision.

## Main choices

- One Cargo package with a thin binary and library modules.
- Synchronous owned types and a pure core first; Tokio and async reqwest when
  networking begins.
- Trusted compiled tool handlers receive a narrow `WorkspaceReader` backed by
  `cap-std`, not permission to open arbitrary operating-system paths.
- Explicit admission, model-call, blocking-tool, queue-byte, and per-run limits.
- A single SQLite owner thread; acknowledged transactions couple event records
  with the run's current status.
- No automatic external-effect retry or crash resume. Those need semantics beyond
  an event log.

Minimality here means a small number of necessary mechanisms you can explain,
not the fewest dependencies. [Decisions](docs/decisions.md) records what was
chosen instead, and why, including the choices this project reversed.

## Checkpoints

| Steps | Checkpoint |
|-------|------------|
| 1–6 | Understandable types and a pure conversation core |
| 7–14 | One bounded, durable, async model turn |
| 15–22 | Two read-only tools, an acceptance check, and live evaluation |
| 23–29 | Bounded concurrent runs for one owner |
| 30–31 | Inspection, replay, and streaming: the first local release |
| 32–33 | Remote inference and optional server supervision |
| 34–41 | An authenticated shared service with an explicit exposure gate |

Track them in [the roadmap](docs/roadmap.md). No milestone is complete because
its design is documented, and shared use has a separate exposure gate that
passing a local demo does not satisfy.

## Documentation

| Document | Responsibility |
|----------|----------------|
| [build guide](docs/build-guide.md) | **Start here.** Orientation and 41 numbered build/prove/break/read steps |
| [roadmap](docs/roadmap.md) | Checkpoints and release gates |
| [architecture](docs/architecture.md) | Ownership, interfaces, deployment, scale boundary |
| [understanding](docs/understanding.md) | Feedback on the original plan, and self-check questions |
| [loop and tools](docs/loop-and-tools.md) | Domain/protocol and tool execution contracts |
| [configuration](docs/configuration.md) | Configuration examples and per-run limits |
| [security](docs/security.md) | Authority, data disclosure, authentication, isolation |
| [performance](docs/performance.md) | Shared limits, scheduling, latency targets, experiments |
| [traces](docs/traces.md) | SQLite journal, capture modes, replay, crash semantics |
| [task acceptance](docs/verification.md) | Execution versus acceptance, evidence and verdicts |
| [testing](docs/testing.md) | Invariant and adversarial verification |
| [process](docs/process.md) | Daily workflow, dependencies, CI |
| [integration](docs/integration.md) | Model API, streaming, process, remote inference |
| [components](docs/components.md) | Module ownership and Rust learning map |
| [cli](docs/cli.md) | The commands the finished runtime exposes |
| [decisions](docs/decisions.md) | Chosen tradeoffs and superseded choices |
| [adversarial review](docs/adversarial-review.md) | Attacks, findings, fixes, and remaining limits |
| [research](docs/research.md) | Primary evidence and review results |
| [paper review](docs/paper-review.md) | The research improvement pass |
| [resources](docs/resources.md) | Short reference index |

### Evidence from the reference build

These record what following this guide actually produced on `answer-key`,
including its failures and its unmet targets. They are results, not instructions.

| Document | Responsibility |
|----------|----------------|
| [build validation](docs/build-validation.md) | Observed proofs and remaining release gates |
| [live evaluation](docs/live-evaluation.md) | Checked-task results and preserved freeform failures |
| [live comparisons](docs/live-comparisons.md) | Controlled context and streaming experiments |
| [performance baseline](docs/performance-baseline.md) | Warm latency, concurrency, cancellation, soak |
| [service load](docs/service-load.md) | Fixed arrivals and HTTP overload evidence |
| [model preflight](docs/model-preflight.md) | The pinned model, server build, and wire fixtures |
| [native state audit](docs/native-state-audit.md) | What the Windows state-tree check does and does not prove |
