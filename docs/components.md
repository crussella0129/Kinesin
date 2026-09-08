# Component guide

Each entry below gives the same four things:

- **Purpose** — what the part does.
- **Standard library tools** — the `std` modules to learn.
- **Rust study** — the exact book sections to read.
- **How it connects** — the link to other parts, where relevant.

For ownership, read the **Brown University fork** of the Rust book first. Its
Chapter 4 is larger than the standard book. It adds a "Fixing Ownership Errors"
section and an "Ownership Recap" that shows the Read/Write/Own permission model.
That model makes the borrow checker easier to understand. Read the standard book
for the other chapters.

- Standard book: https://doc.rust-lang.org/book/
- Brown fork: https://rust-book.cs.brown.edu/

---

## `Cargo.toml` (workspace manifest, root)

**Purpose.** Lists the member crates and shares build settings. This root file is
a workspace manifest, not a package. It has a `[workspace]` table with a
`members` list; it does not have a `[package]` table.

**Rust study.**
- The Cargo Book, "Workspaces": https://doc.rust-lang.org/cargo/reference/workspaces.html
- The Cargo Book, "The Manifest Format":
  https://doc.rust-lang.org/cargo/reference/manifest.html
- Rust book, Chapter 14.3 "Cargo Workspaces".

---

## `crates/kineserve/` — the model server supervisor

**Purpose.** Starts `llama-server` as a child process. Points it at the model in
`models/`. Waits until the server is ready. Stops the server on shutdown.

**Standard library tools.**
- `std::process::Command` — start `llama-server`, set its flags, hold the child
  handle.
- `std::net::TcpStream` — poll the server's `/health` endpoint until it answers.
- `std::io` (`Read`, `Write`, `BufReader`) — read the child's output and the
  socket.
- `std::thread` and `std::time` — wait and retry during model load.

**Rust study.**
- Rust book, Chapter 21 "Final Project: Building a Multithreaded Web Server".
  Sections 21.1–21.3 build an HTTP server and a thread pool with `std::net` only.
  You do not build a server here, but the same TCP and stream skills apply to the
  health poll and the raw HTTP client.
- Rust by Example, "Std Misc → Child processes":
  https://doc.rust-lang.org/rust-by-example/std_misc/process.html
- Standard library docs for `std::process::Command`:
  https://doc.rust-lang.org/std/process/struct.Command.html
- Rust book, Chapter 9 "Error Handling" — the child process can fail to start;
  return a `Result`.
- Brown fork, Chapter 4 — the child handle is a value that Kineserve owns; learn
  move and borrow rules before you pass the handle around.

**How it connects.** See [integration.md](integration.md), llama-server for the child-process and HTTP details.

---

## `crates/kineserve/src/input.rs` and `crates/k-core/src/input.rs`

**Purpose.** Collects user input with the standard library only. Treats the
collected input as an immutable value. The two `input.rs` files do the same job
in their own crate.

**Standard library tools.**
- `std::io::stdin` and `std::io::BufRead` — read a line or a block of text.
- `std::fs` — read input from a file when the input is not from the keyboard.
- `String` and `&str` — hold and borrow the text.

**Rust study.**
- Rust book, Chapter 2 "Programming a Guessing Game" — the first `stdin` example.
- Rust book, Chapter 8.2 "Storing UTF-8 Encoded Text with Strings".
- Rust book, Chapter 12.1–12.3 — read arguments, read a file, and structure the
  program well. This is the closest pattern to a small command-line tool.
- Brown fork, Chapter 4.1–4.2 and 4.5 — an immutable input is a value with read
  permission but no write permission. The permission model explains why the rest
  of the program cannot change it.

---

## `crates/koil/` — the channel and the trace recorder

**Purpose.** Two jobs. First, it carries messages between K-Core and Kineserve.
Second, it records every message as a trace file.

**Why Koil exists (the reason, written down).** Kineserve does not have to run on
your machine. The goal is to run the model on a bigger machine somewhere else,
which serves more sessions at the same time, and to keep that link private
wherever the two machines are. Koil is the endpoint that makes this possible. In
the local setup, one Koil passes messages across `127.0.0.1`. In the distributed
setup, a second Koil runs beside the remote Kineserve, and the two Koils hold a
private WireGuard link. The local case is the same design with one endpoint, not
a different design.

Because Koil sits at the boundary, it is also the correct place to record the
traces. It sees every message in both directions. The transport and the audit
record belong together here for that reason.

**Standard library tools.**
- `std::process::Command` — start or configure the system WireGuard tools.
- `std::fs` and `std::io::Write` — write each trace to `traces/logs/`.
- `std::collections::HashMap` — hold the ID-to-file lookup table in memory before
  it writes `trace_hash.json`.
- `std::time::SystemTime` — add a timestamp to each trace.
- `std::hash` (`Hasher`) or an entropy read — make the random trace ID (see
  [decisions.md](decisions.md), items 4 and 5).

**Rust study.**
- Rust book, Chapter 8.3 "Storing Keys with Associated Values in Hash Maps".
- Rust book, Chapter 12.2 and 12.4 — read and write files well.
- Rust book, Chapter 16 "Fearless Concurrency" — if Koil records traces on a
  separate thread, read 16.1 (threads), 16.2 (channels), and 16.3 (`Arc` and
  `Mutex` for shared state).
- Brown fork, Chapter 4 whole chapter — Koil holds data that more than one part
  reads; ownership and borrow rules matter most here.

**How it connects.** See [integration.md](integration.md), WireGuard for WireGuard. See [traces.md](traces.md) for the trace
schema.

> **Build the seam first.** This is the most important thing in Phase 1. Koil must
> show K-Core one interface: "send this request to Kineserve, and give me the
> answer". K-Core must never know whether the bytes cross loopback or a tunnel.
> Give the interface one implementation now, `direct`, which passes messages
> across `127.0.0.1`. Phase 5 adds a second implementation, `wireguard`, behind
> the same interface. If the seam is correct, Phase 5 changes nothing in K-Core.
> If the seam is wrong, Phase 5 touches every part of the harness.

> **Keep concurrency out of Koil.** The bigger machine gives you more sessions at
> the same time, but the tunnel is not what provides them. `llama-server` provides
> them with its parallel slots (`-np`), and K-Core keeps the sessions apart with
> the `session` field in each trace. Koil stays a private pipe. Do not put a
> scheduler in it.

---

## `crates/k-core/` — the ReAct loop and function routing

**Purpose.** The brain of the harness. It reads the configuration and the
instructions. It builds each request to the model. It reads the model's answer.
It detects a function call in the answer, routes the call to the correct
function, and adds the result to the conversation. It maps the order of these
steps to JSON traces that work with `llama-server`.

**Standard library tools.**
- `enum` and `match` — model the loop state and the message types.
- `struct` — hold a message, a function call, and a trace.
- `std::collections::HashMap` — map a function name to its handler.
- `Result` and the `?` operator — carry errors up the call chain.
- Iterators — walk the conversation and the parsed fields.

**Rust study.**
- Rust book, Chapter 5 "Using Structs" — model the data.
- Rust book, Chapter 6 "Enums and Pattern Matching" — model the loop states and
  branch on them with `match`.
- Rust book, Chapter 7 "Managing Growing Projects" — split `main.rs` and
  `input.rs` into modules and bring them into scope with `use`.
- Rust book, Chapter 9 "Error Handling" — a robust loop needs a clear error path.
- Rust book, Chapter 13 "Iterators and Closures" — parse and route with iterators.
- Brown fork, Chapter 4 and Chapter 4.3 "Fixing Ownership Errors" — the loop
  passes strings and structs between functions; this section shows how to fix the
  common borrow errors that you will meet.

**Background on the loop.** The ReAct pattern comes from the paper "ReAct:
Synergizing Reasoning and Acting in Language Models" (Yao and others, 2022):
https://arxiv.org/abs/2210.03629 . Read it once to understand the reason–act
cycle. You do not need to copy the paper; you need the idea of a loop that thinks,
acts, observes, and repeats.

---

## `models/`

**Purpose.** Holds your model file in GGUF format. Choose a text-to-text model.
Copy or clone the file here before you run Kineserve.

**Choose the model with care. It decides whether the harness works at all.** A
harness that routes tool calls depends on the model's ability to choose a tool and
to fill in the arguments. Measurements of local models show a sharp limit:

- **8B parameters is the practical minimum** for tool calls. Below that, tool
  selection and argument accuracy fall fast.
- **The Qwen 3 family gives the best local results.** Qwen 3 8B is the best
  balance of speed and accuracy. Qwen 3 14B is more accurate and slower.
- **Size alone does not decide quality.** In one measurement, a 70B model scored
  below an 8B Qwen model at tool selection. Some models that advertise tool use
  scored worst of all. Test the model you plan to use.
- **A model must have a chat template that supports tools.** [loop-and-tools.md](loop-and-tools.md), how the model asks for an action explains
  how to check this before you build anything.

**Quantization: one trap to avoid.**

- **Weight quantization is safe.** A Q4_K_M file performs about the same as the
  full-precision file for tool calls. Use it.
- **KV-cache quantization is not safe.** The llama.cpp documentation warns that
  extreme KV quantization, for example `-ctk q4_0`, substantially degrades tool
  call performance. Separate measurements show 4-bit KV cache causes *silent*
  failures on long, tool-heavy prompts: the answer still reads well, but a wrong
  token early corrupts everything after it. Keep the default KV cache. If you must
  save memory, use 8-bit, never 4-bit.

**Study.** GGUF is the file format that llama.cpp reads. Read the format note in
the ggml repository so you pick a file that `llama-server` can load:
https://github.com/ggml-org/ggml/blob/master/docs/gguf.md

---

## `traces/`

**Purpose.** Holds the audit log. `traces/logs/<trace-id>.json` is one trace per
message. `traces/trace_hash.json` maps each ID to its file. A trace is immutable
after Koil writes it.

**Standard library tools.** `std::fs`, `std::io::Write`, `std::collections::HashMap`.

**Rust study.** Rust book, Chapter 8.3 (hash maps) and Chapter 12.2 (file I/O).

**Decisions.** The trace schema is in [traces.md](traces.md). The ID scheme is open; see
[decisions.md](decisions.md), items 4 and 5.

---

## `kinesin.toml`

The settings and the agent instructions live in one file. It has its own
document, because it grew: [configuration.md](configuration.md) holds the
skeleton, the split rule, and the settings to add later.

---

## `crates/common/` — the shared types

**Purpose.** Holds the types that more than one crate uses: the message, the
trace, the tool definition, and the settings. Every binary crate depends on this
one. Without it, the same struct appears in two places and the two copies drift
apart.

**Standard library tools.** Only type definitions and their methods. This crate
does no input and no output, which is what makes it simple to test.

**Rust study.**
- Rust book, Chapter 7 "Managing Growing Projects" — how a library crate differs
  from a binary crate, and how `pub` controls what other crates can see.
- Rust book, Chapter 5 (structs) and Chapter 6 (enums).
- Rust API Guidelines, on what to make public:
  https://rust-lang.github.io/api-guidelines/

**Note.** What you mark `pub` here decides what an integration test can reach. See
[testing.md](testing.md), section 1.

---

## `scripts/`

**Purpose.** Set up and start order. `preflight.ps1` checks the environment. It
detects Windows, checks or installs WSL, and installs `llama.cpp`, WireGuard, and
the other non-Rust dependencies with the correct package manager.
`run-harness.ps1` starts the three parts in the correct order.

**Study.**
- PowerShell scripting overview:
  https://learn.microsoft.com/powershell/scripting/overview
- WSL install and commands: https://learn.microsoft.com/windows/wsl/install
- [integration.md](integration.md), WSL explains WSL and the package steps.

**Decision.** These scripts are Windows and PowerShell. Decide the Linux and
macOS equivalent, or move the checks into the Rust binary. See [decisions.md](decisions.md), item 10.

---

## `.github/workflows/`

**Purpose.** Continuous integration. Run the format check, the lint, the build,
and the tests on each push.

**Study.**
- GitHub Actions quickstart:
  https://docs.github.com/actions/quickstart
- The three Rust commands to run in CI: `cargo fmt --check`, `cargo clippy`, and
  `cargo test`.
- Rust book, Chapter 11 "Writing Automated Tests" — write the tests that CI runs.

---

