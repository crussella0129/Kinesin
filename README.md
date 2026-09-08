# Kinesin

The *even tinier* General Purpose Harness (but a very hard worker for its size!)

Kinesin is a small agent harness. It runs a local language model, it routes the
model's requests to tools, and it keeps a full record of every message. It is
written with the Rust standard library first. Outside libraries are added only
where the standard library cannot do the job, and each one is written down with
its reason.

**Status: nothing is built yet.** This repository holds the design and the plan.
Start at [docs/roadmap.md](docs/roadmap.md), Phase 0.

---

## The three parts

- **Kineserve** starts a local model server and points it at your model file. It
  uses `llama-server` from the llama.cpp project.
- **Koil** carries messages between the harness and the model server, and records
  every message that crosses it.
- **K-Core** is the brain. It runs the ReAct loop (reason, then act), it routes
  tool calls, and it reads your configuration and instructions.

```
K-Core  <->  Koil  <->  Kineserve  <->  llama-server  <->  model.gguf
(ReAct       (channel   (supervisor    (HTTP server      (your GGUF
 loop +       + trace     for the        from             text model)
 routing)     record)     model server)  llama.cpp)
```

Kineserve does not have to run on your machine. A second Koil can run beside a
remote Kineserve, with a private link between the two Koils. The local setup above
is the same design with one endpoint. See
[docs/architecture.md](docs/architecture.md).

---

## Intended layout

```
Kinesin/
├── .github/
│   └── workflows/                 # CI: format, lint, build, test
├── crates/                        # Workspace member crates
│   ├── kineserve/                 # Starts and supervises llama-server
│   │   ├── src/
│   │   │   ├── main.rs            # Entry point: start the server, wait for /health
│   │   │   └── input.rs           # Reads input with std only (io, fs)
│   │   ├── tests/                 # Integration tests (public API only)
│   │   └── Cargo.toml
│   ├── koil/                      # The transport seam; records traces
│   │   ├── src/
│   │   │   └── main.rs            # Direct or tunnel transport; writes traces
│   │   ├── tests/
│   │   └── Cargo.toml
│   ├── k-core/                    # The ReAct loop and tool routing
│   │   ├── src/
│   │   │   ├── main.rs            # Entry point: run the loop
│   │   │   └── input.rs           # Reads input with std only
│   │   ├── tests/
│   │   └── Cargo.toml
│   └── common/                    # Shared types: message, trace, config
│       ├── src/
│       └── Cargo.toml
├── docs/                          # The design and the plan (see below)
├── models/
│   └── your-gguf-here.gguf        # Your GGUF model; 8B parameters or more
├── traces/
│   ├── trace_hash.json            # Lookup table: ID -> trace file
│   └── logs/
│       └── <trace-id>.json        # One immutable trace per event; random ID
├── scripts/
│   ├── preflight.ps1              # Windows/WSL environment check and install
│   └── run-harness.ps1            # Start order for the three parts
├── kinesin.toml                   # TOML settings + Markdown instructions
├── Cargo.toml                     # Workspace manifest (lists the members)
├── LICENSE
└── README.md
```

---

## Design rules

1. **Standard library first.** Use `std` for input, output, files, sockets,
   threads, and collections. Add an outside library only when `std` has no
   answer, and write down why.
2. **Small surface.** Keep each crate small. Prefer clear code over clever code.
3. **One meaning per term.** A "trace" is always the JSON record of one event.
   "Record" is the verb for the act of saving a trace.
4. **Everything is observable.** Every message and every tool call is recorded. A
   trace is immutable after it is written.
5. **Configuration is the source of truth.** `kinesin.toml` holds the settings and
   the agent instructions. The program reads it; the program does not change it.
6. **Write down why.** For each design choice, record the reason next to the
   choice. A structure without a reason looks like a mistake to the next reader,
   and you are the next reader.

Three tasks break the "standard library only" rule: the standard library has no
JSON parser, no TOML parser, and no cryptography.
[docs/decisions.md](docs/decisions.md) gives the decision for each.

---

## Documentation

| Document | What it covers |
|----------|----------------|
| [roadmap.md](docs/roadmap.md) | **Start here.** Phases 0 to 6, with what to build in what order |
| [architecture.md](docs/architecture.md) | The parts, the data flow, and both deployment shapes |
| [components.md](docs/components.md) | Each crate and folder: purpose, `std` tools, and study material |
| [configuration.md](docs/configuration.md) | `kinesin.toml`: the skeleton, the split rule, and future settings |
| [integration.md](docs/integration.md) | The non-Rust parts: llama-server, WireGuard, and WSL |
| [traces.md](docs/traces.md) | The trace schema, tool events, and how to read the log |
| [loop-and-tools.md](docs/loop-and-tools.md) | The ReAct loop, and how tool calling works |
| [testing.md](docs/testing.md) | Units, unit tests, integration tests, and test doubles |
| [process.md](docs/process.md) | The development cycle, branches, commits, and CI |
| [decisions.md](docs/decisions.md) | Open decisions, and settled ones with their reasons |
| [resources.md](docs/resources.md) | Every study link, in one place |

---

## A note on the writing

These documents use ASD-STE100 Simplified Technical English: short sentences,
active voice, and one idea per sentence. The aim is that a step means the same
thing to every reader.
