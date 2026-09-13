# Kinesin

[![Rust checks](https://github.com/crussella0129/Kinesin/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/crussella0129/Kinesin/actions/workflows/ci.yml?query=branch%3Adev)

A Rust agent harness with explicit authority and bounded resources.

## Start here

Install Kinesin, a llama.cpp runtime and a GGUF model once, then type
**`kinesin` from any folder**. Choose a working folder, use the arrow keys to
choose a model, and press Enter. Kinesin starts the local model server, waits
for it to become ready, and stops its server when you leave. Sessions show the
selected folder, model, permitted actions, tool activity and readable answers.

From an existing checkout with Rust installed:

**Windows PowerShell**

```powershell
cd "$HOME\Kinesin"
cargo install --locked --path . --bin kinesin
$env:Path = "$HOME\.cargo\bin;$env:Path"
kinesin
```

**Linux**

```bash
cd "$HOME/Kinesin"
cargo install --locked --path . --bin kinesin
export PATH="$HOME/.cargo/bin:$PATH"
kinesin
```

For a faster development installation, append `--debug` to the install command.
`cargo run --locked` from the checkout opens the same entry point, and
`kinesin --help` needs no configuration or model. Cloning alone does not install
the command. See the [Windows and Linux guide](docs/getting-started.md) for build
prerequisites, persistent PATH and the complete first session. Before asking for
work, follow [the one-time local model setup](docs/getting-started.md#install-a-local-model-and-runtime).

Put GGUF files in `%LOCALAPPDATA%\Kinesin\models` on Windows or
`~/.local/share/kinesin/models` on Linux (or `$XDG_DATA_HOME/kinesin/models` when
configured). Install `llama-server` and its runtime dependencies in the sibling
`runtime` directory. A portable installation can instead keep `models/` and
`runtime/` beside the installed Kinesin executable, with `llama-server` directly
inside `runtime/`. Existing `model/` and `models/` folders in the launch directory
are also checked. To use files elsewhere:

```text
kinesin --model-path "PATH/TO/model-example.gguf" --runtime-path "PATH/TO/llama-server"
```

Use `llama-server.exe` on Windows. These terminal-session options select local
files; they do not grant the assistant access to the folders containing them.
Model/runtime directories must be disjoint from the working folder; an overlap
asks you to choose another working folder.

Choose a working folder at the prompt, then ask for work there:

```text
> make a folder called test 1
> list the files in this folder
```

The personal profile allows read/search, folder creation and file writing/editing
inside that selected folder. `/help`, `/permissions`, `/status`, `/new` and `/exit`
control the session. Normal local use needs no SSH or second terminal.
`kinesin --external` explicitly attaches to a separately managed model server;
see [optional remote-server connections](docs/getting-started.md#optional-remote-server-connection).

Normal terminal entry uses private per-user settings, so projects do not need
their own `kinesin.toml`. `--config PATH` explicitly selects a fixed profile and
skips folder selection; `--json` exposes session receipts for automation.
Human setup requires terminal input and output; use `--config PATH --json` when
redirecting session output.
The tracked [`kinesin.example.toml`](kinesin.example.toml) remains a read-only
checked-task example. One-shot and operator commands retain structured output.

**Why are there three binaries?** `kinesin` is the product. `cmd-fixture` is a
test child process for command execution and cleanup; `mcp-fixture` is a test
stdio server for MCP. Tests need real separate processes to exercise those
boundaries. You do not need to launch or install either fixture for normal use.
The `--bin kinesin` installation above installs only the product.

## What Kinesin is

Kinesin gives a model context, interprets its proposed tool calls, decides which
may run, records outcomes, and controls how the run ends. It also controls how
many runs may compete for model, filesystem, memory, and storage resources.

The Rust CLI and interactive session, concurrent controller, read and write file
tools, argv commands, local MCP tool servers, checker, journal, replay, streaming,
and authenticated loopback service run locally. The [roadmap](docs/roadmap.md) and
[threat model](docs/threat-model.md) distinguish delivered mechanisms from open
capabilities and evidence. The [validation ledger](docs/build-validation.md)
retains earlier proofs, model-quality failures and deployment limits. The handwritten
**build guide that teaches how to construct this from scratch lives in its own
repository**:
[building-an-agent-harness](https://github.com/crussella0129/building-an-agent-harness).

Execution completion and task acceptance are separate outcomes. A final answer
is a candidate: scoped tasks pass only after an independent checker verifies
their frozen contract. Freeform answers remain explicitly unchecked. See
[task acceptance](docs/verification.md) and the [adversarial review](docs/adversarial-review.md).

Concurrent independent runs and an owner-scoped loopback service are implemented.
Model-spawned subagents and non-loopback service exposure remain separate work.
Security, low latency, scalability, and minimality are equal design
goals. Minimality means a small number of necessary mechanisms you can explain,
rather than the fewest dependencies or source files.

## Working here

- `main` contains reviewed source snapshots. `dev` is where sprints happen; each sprint lands
  on `main` through a pull request.
- Read [the architecture](docs/architecture.md) for the complete shape.
- Use [the CLI instructions](docs/cli.md) to run the product.
- Read the [live task evaluation](docs/live-evaluation.md) and
  [performance baseline](docs/performance-baseline.md) before interpreting success.
- To build the runtime yourself by hand, follow the separate
  [build guide](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md#before-you-start).

## What the harness does

It accepts a bounded batch of independent tasks, runs them concurrently, and
gives each task its own conversation, workspace authority, budgets, cancellation,
and result. It provides read/list/search, write/edit/delete/move, bounded argv
commands and operator-declared local MCP tools through a llama.cpp adapter.
Streaming makes useful text visible sooner; partial tool arguments never execute.

Tests use scripted models. Benchmarks distinguish the runtime's overhead from
model inference. Capacity limits prevent an overload from becoming an unbounded
queue. Private state stores outcomes, with full replay capture explicitly selected.

```text
CLI / authenticated loopback API
             |
      admission + scheduler
             |
      one runner per run ----------> approved model endpoint
        |          |                   via pooled async HTTP
        |          +-> scoped files / argv commands / local MCP
        +-> bounded storage inbox -> SQLite owner thread
```

The existing names remain useful: **K-Core** is the pure decision logic,
**Koil** names the model-adapter responsibility. Local terminal sessions now
supervise their selected llama.cpp server. Broader **Kineserve** deployment
management and the separate Koil encrypted-overlay project remain proposed.
Explicit profiles can still attach to an externally started model endpoint.

## Main choices

- One Cargo package with a thin binary and library modules.
- Owned decision types and deterministic fixtures; Tokio and pooled async reqwest
  for runtime effects.
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
- The single-controller service enforces authentication, owner scope, per-owner
  quotas and fair scheduling. Its listener remains loopback-only.

Linux commands use mandatory Landlock/seccomp restrictions, with metadata and
same-user process limits described in the threat model. Windows Jobs control
process lifetime; Windows filesystem/network isolation remains proposed.
Non-loopback model origins require HTTPS. Sprint 10 records a
[real two-host checked run and actual-session cache observation](docs/sprints/s10/sprint-tests/remote-deployment.md).
Broader deployment, context-continuity and concurrent-slot claims remain on the roadmap.

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
│   ├── tools.rs         # File tools, argv execution and Linux sandbox
│   ├── process.rs       # Owned process groups/Jobs and environment defaults
│   ├── mcp.rs           # Local stdio tools, bounded discovery and calls
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

These describe and validate the runtime. Its roadmap, intent chapters and sprint
evidence live here. The handwritten learning guide lives in the separate
[building-an-agent-harness](https://github.com/crussella0129/building-an-agent-harness)
repository.

| Document | Responsibility |
|----------|----------------|
| [getting started](docs/getting-started.md) | Windows/Linux installation, model setup, first run and troubleshooting |
| [CLI reference](docs/cli.md) | Session, one-shot, batch, inspect/export/replay and service commands |
| [architecture](docs/architecture.md) | Ownership, interfaces, deployment, scale boundary |
| [loop and tools](docs/loop-and-tools.md) | Domain/protocol and tool execution contracts |
| [configuration](docs/configuration.md) | Configuration examples and per-run limits |
| [security](docs/security.md) | Authority, data disclosure, authentication, isolation |
| [threat model](docs/threat-model.md) | Individual OWASP risk mappings, native interfaces and evidence limits |
| [supply chain](docs/supply-chain.md) | Blocking dependency policy, exact exceptions, native builds and cargo-vet decision |
| [roadmap](docs/roadmap.md) | Current state, intent ownership and remaining priorities |
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
