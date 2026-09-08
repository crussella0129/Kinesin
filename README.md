# Kinesin

[![Rust checks](https://github.com/crussella0129/Kinesin/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/crussella0129/Kinesin/actions/workflows/ci.yml?query=branch%3Adev)

A small Rust runtime for agents with explicit authority and bounded resources.

Kinesin gives a model context, interprets its proposed tool calls, decides which
may run, records outcomes, and controls how the run ends. It also controls how
many runs may compete for model, filesystem, memory, and storage resources.

**This is the reference implementation.** The Rust CLI and interactive session,
concurrent controller, read and write file tools, checker, journal, replay,
streaming, and authenticated loopback service run locally. The
[validation ledger](docs/build-validation.md) separates passing proofs from
model-quality failures and deployment gates that remain unproven. The handwritten
**build guide that teaches how to construct this from scratch lives in its own
repository**:
[building-an-agent-harness](https://github.com/crussella0129/building-an-agent-harness).

Execution completion and task acceptance are separate outcomes. A final answer
is a candidate: scoped tasks pass only after an independent checker verifies
their frozen contract. Freeform answers remain explicitly unchecked. See
[task acceptance](docs/verification.md) and the [adversarial review](docs/adversarial-review.md).

The destination is **concurrent agents for one operator, followed by a shared
service**. Security, low latency, scalability, and minimality are equal design
goals. Minimality means a small number of necessary mechanisms you can explain,
rather than the fewest dependencies or source files.

## Working here

- `main` is the released branch. `dev` is where sprints happen; each sprint lands
  on `main` through a pull request.
- Read [the architecture](docs/architecture.md) for the complete shape.
- Use [the CLI instructions](docs/cli.md) to run the product.
- Read the [live task evaluation](docs/live-evaluation.md) and
  [performance baseline](docs/performance-baseline.md) before interpreting success.
- To build the runtime yourself by hand, follow the separate
  [build guide](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md#before-you-start).

## What the first complete release does

It accepts a bounded batch of independent tasks, runs them concurrently, and
gives each task its own conversation, workspace authority, budgets, cancellation,
and result. It begins with three read-only file tools plus a bounded write/edit/delete/move
set, and one llama.cpp adapter.
Streaming makes useful text visible sooner; partial tool arguments never execute.

Tests use scripted models. Benchmarks distinguish the runtime's overhead from
model inference. Capacity limits prevent an overload from becoming an unbounded
queue. Private state stores outcomes, with full replay capture explicitly selected.

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

The existing names remain useful: **K-Core** is the pure decision logic,
**Koil** is the model adapter, and **Kineserve** is optional model-process
supervision. They are responsibilities, not three mandatory daemons.

## Main choices

- One Cargo package with a thin binary and library modules.
- Synchronous owned types and fake events first; Tokio and async reqwest when
  networking begins.
- Trusted compiled tool handlers receive a narrow `WorkspaceReader` backed by
  `cap-std`, not permission to open arbitrary operating-system paths. Writes use a
  separate `WorkspaceWriter`, built only where an operator granted a write tool.
- Explicit admission, model-call, blocking-tool, queue-byte, and per-run limits.
- A single SQLite owner thread; acknowledged transactions couple event records
  with the run's current status. No custom crash-recovery file format.
- Metadata capture by default, owner-scoped final results, optional private
  replay data. Operational metrics do not contain prompts.
- No automatic external-effect retry or crash resume. Those require semantics
  beyond an event log.
- A later single-controller shared service adds authentication, authorization,
  per-owner quotas, and fair scheduling before network exposure.

See [decisions](docs/decisions.md) for alternatives and
[security](docs/security.md) for exactly what these boundaries protect.

## Implemented layout

```text
Kinesin/
├── Cargo.toml / Cargo.lock
├── src/
│   ├── main.rs          # CLI and runtime entry
│   ├── lib.rs
│   ├── core.rs          # K-Core: owned state and pure transitions
│   ├── runner.rs        # One run's effects and cancellation
│   ├── model.rs         # Koil: prepare, send, decode; scripted/HTTP variants
│   ├── config.rs        # Validated deployment settings
│   ├── policy.rs        # Immutable RunAuthority
│   ├── tools.rs         # Bounded capability-backed read-only handlers
│   ├── verification.rs  # Frozen task contracts, evidence, pure acceptance checks
│   ├── storage.rs       # Transactional events, status, owner-scoped queries
│   ├── scheduler.rs     # Bounded admission and resource allocation
│   ├── dispatch.rs      # Fair assignment of actual model capacity
│   ├── auth.rs          # Credential provisioning and verification
│   ├── private_state.rs # Native permissions and state-tree inspection
│   ├── ingress.rs       # Bounded HTTP connections and shutdown
│   ├── service.rs       # Authenticated owner-scoped API
│   ├── operator.rs      # Private service startup and readiness supervision
│   ├── signal.rs        # Native console cancellation registration
│   ├── cli.rs           # Commands, batch pacing and exit policy
│   └── replay.rs        # Pure recorded-decision verification
├── tests/fixtures/      # Synthetic protocol, tool, and event examples
├── docs/
├── models/              # Local GGUF artifacts, outside Git
├── workspace/           # Deliberately provisioned tool inputs
├── state/               # Private SQLite database and sidecars, outside Git
└── kinesin.toml         # Operator-controlled configuration
```

## Documentation map

These describe and validate the runtime. The build guide, roadmap, working
process, and research review moved to the separate
[building-an-agent-harness](https://github.com/crussella0129/building-an-agent-harness)
repository.

| Document | Responsibility |
|----------|----------------|
| [architecture](docs/architecture.md) | Ownership, interfaces, deployment, scale boundary |
| [loop and tools](docs/loop-and-tools.md) | Domain/protocol and tool execution contracts |
| [configuration](docs/configuration.md) | Configuration examples and per-run limits |
| [security](docs/security.md) | Authority, data disclosure, authentication, isolation |
| [performance](docs/performance.md) | Shared limits, scheduling, latency targets, experiments |
| [build validation](docs/build-validation.md) | Observed proofs and remaining release gates |
| [live evaluation](docs/live-evaluation.md) | Checked-task results and preserved freeform failures |
| [live comparisons](docs/live-comparisons.md) | Controlled context and streaming experiments |
| [performance baseline](docs/performance-baseline.md) | Warm latency, concurrency, cancellation and soak evidence |
| [service load](docs/service-load.md) | Fixed arrivals, generator omissions and HTTP overload evidence |
| [traces](docs/traces.md) | SQLite journal, capture modes, replay, crash semantics |
| [integration](docs/integration.md) | Model API, streaming, process, remote inference |
| [testing](docs/testing.md) | Invariant and adversarial verification |
| [task acceptance](docs/verification.md) | Execution versus acceptance, first checker, evidence and verdicts |
| [adversarial review](docs/adversarial-review.md) | Attacks, findings, fixes, and remaining limits |
| [components](docs/components.md) | Module ownership and Rust learning map |
| [decisions](docs/decisions.md) | Chosen tradeoffs and superseded choices |
| [resources](docs/resources.md) | Short reference index |

No milestone is complete merely because its design is documented. Shared use has
a separate exposure gate; passing a local demo is not that gate.
