# Kinesin

The *even tinier* General Purpose Harness (but a very hard worker for its size!)

Kinesin is a small agent harness. It runs a local language model, it routes the
model's requests to functions, and it keeps a full record of every message. The
plan is to write it with the Rust standard library first, and to add outside
libraries only where the standard library cannot do the job.

This README is the scaffold. It describes each part, the standard library tools
that each part needs, the study material for each step, and the open decisions
that you must make before you write code. The document uses ASD-STE100 Simplified
Technical English: short sentences, active voice, and one idea per sentence.

---

## Contents

1. [What Kinesin is](#1-what-kinesin-is)
2. [Design rules](#2-design-rules)
3. [Architecture and data flow](#3-architecture-and-data-flow)
4. [Project layout](#4-project-layout)
5. [Component guide](#5-component-guide)
6. [The non-Rust parts and how to connect them](#6-the-non-rust-parts-and-how-to-connect-them)
7. [Trace schema](#7-trace-schema)
8. [The ReAct loop](#8-the-react-loop)
9. [What is missing: decisions to make first](#9-what-is-missing-decisions-to-make-first)
10. [Roadmap](#10-roadmap)
11. [Learning resources index](#11-learning-resources-index)

---

## 1. What Kinesin is

Kinesin has three parts:

- **Kineserve** starts a local model server and points it at your model file. It
  uses `llama-server` from the llama.cpp project.
- **Koil** is the channel between the harness and the model server. It also
  records every message that crosses it.
- **K-Core** is the brain. It runs the ReAct loop (reason, then act), it routes
  function calls, and it reads your configuration and instructions.

**Where it runs.** Kinesin splits the control work from the compute work. K-Core
is small and can run anywhere. Kineserve needs a machine with enough memory and a
good GPU. The two do not need to be the same machine.

- **Now (local).** All three parts run on one machine. Koil passes messages
  across `127.0.0.1`. One Koil is enough. This is the build target for Phase 1.
- **Later (distributed).** Kineserve runs on a bigger machine somewhere else, and
  serves more sessions at the same time. A second Koil runs beside it. The two
  Koils hold a private link between them, wherever each machine is.

Koil is the same component in both cases. The local setup is the simple case of
the same design, not a different design. Section 10, Phase 5 covers the
distributed step.

> **Note:** In this document, "user" means a human **or** another agent. Both send
> input to Kinesin in the same way.

The name is a metaphor only. A kinesin is a motor protein that carries cargo
along a track. Kinesin the program carries messages between the harness and the
model. Do not read more into the name than that.

---

## 2. Design rules

1. **Standard library first.** Use `std` for input, output, files, sockets,
   threads, and collections. Add an outside library only when `std` has no
   answer, and write down why.
2. **Small surface.** Keep each crate small. Prefer clear code over clever code.
3. **One meaning per term.** A "trace" is always the JSON record of one message.
   "Record" is the verb for the act of saving a trace.
4. **Everything is observable.** Koil records all traffic. A trace is immutable
   after Koil writes it.
5. **Configuration is the source of truth.** `kinesin.toml` holds the settings
   and the agent instructions. The program reads it; the program does not change
   it.
6. **Write down why.** For each design choice, record the reason next to the
   choice. A structure without a reason looks like a mistake to the next reader,
   and you are the next reader. The Koil note in Section 5.4 is the example to
   follow.

Three tasks break the "standard library only" rule. The standard library has no
JSON parser, no TOML parser, and no cryptography. Section 9 lists these and gives
you the decision for each.

---

## 3. Architecture and data flow

**Local (Phase 1, build this now).** One machine. One Koil. No tunnel.

```
K-Core  <->  Koil  <->  Kineserve  <->  llama-server  <->  model.gguf
(ReAct       (channel   (supervisor    (HTTP server      (your GGUF
 loop +       + trace     for the        from             text model)
 routing)     record)     model server)  llama.cpp)
```

**Distributed (Phase 5, later).** Two machines. Two Koils. A private link between
them.

```
  your machine                    |        the bigger machine
                                  |
K-Core  <->  Koil  <===============|===>  Koil  <->  Kineserve  <->  llama-server
(ReAct       (local     private    |      (remote    (supervisor)
 loop)        endpoint)  link      |       endpoint)
                                   |
```

K-Core sees the same interface in both pictures. It sends a request to Koil and
receives an answer. It does not know whether the bytes cross loopback or a
tunnel. Section 5.4 explains why this matters more than anything else you build
first.

The message flow for one step is:

1. K-Core builds a request from the instructions and the conversation so far.
2. K-Core sends the request through Koil to Kineserve.
3. Kineserve passes the request to `llama-server` over local HTTP.
4. `llama-server` returns the model's answer.
5. Koil records the request and the answer as traces.
6. K-Core reads the answer. If the answer asks for a function, K-Core calls the
   function and adds the result to the conversation.
7. The loop repeats until a stop condition is true.

**Traces** are the audit log. Each trace is one JSON file in `traces/logs/`. The
file name is a random ID, not a sequential number. `traces/trace_hash.json` is a
lookup table that maps each ID to its file. Section 7 gives the trace schema.
Section 9, items 4 and 5, give the ID decision.

---

## 4. Project layout

The tree below is a proposal. It is a **Cargo workspace** with three member
crates. A workspace lets the three crates share one build and one lock file, but
keeps each crate separate. This layout differs from the first draft in the git
history; Section 9, item 1 explains the change and asks you to confirm it.

```
Kinesin/
├── .github/
│   └── workflows/                 # CI jobs: format, lint, build, test
├── crates/                        # Workspace member crates
│   ├── kineserve/                 # Starts and supervises llama-server
│   │   ├── src/
│   │   │   ├── main.rs            # Entry point: start the server, wait for /health
│   │   │   └── input.rs          # Reads input with std only (io, fs)
│   │   └── Cargo.toml
│   ├── koil/                      # The channel; records traces
│   │   ├── src/
│   │   │   └── main.rs           # Starts or wraps WireGuard; writes traces
│   │   └── Cargo.toml
│   └── k-core/                    # The ReAct loop and function routing
│       ├── src/
│       │   ├── main.rs           # Entry point: run the loop, build JSON traces
│       │   └── input.rs          # Reads input with std only
│       └── Cargo.toml
├── models/
│   └── your-gguf-here.gguf        # Your chosen text-to-text GGUF model
├── traces/
│   ├── trace_hash.json            # Lookup table: ID -> trace file
│   └── logs/
│       └── <trace-id>.json        # One immutable trace per message; random ID
├── scripts/
│   ├── preflight.ps1              # Windows/WSL environment check and install
│   └── run-harness.ps1            # Start order for the three parts
├── kinesin.toml                   # TOML settings + Markdown instructions
├── Cargo.toml                     # Workspace manifest (lists the members)
├── LICENSE
└── README.md
```

---

## 5. Component guide

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

### 5.1 `Cargo.toml` (workspace manifest, root)

**Purpose.** Lists the member crates and shares build settings. This root file is
a workspace manifest, not a package. It has a `[workspace]` table with a
`members` list; it does not have a `[package]` table.

**Rust study.**
- The Cargo Book, "Workspaces": https://doc.rust-lang.org/cargo/reference/workspaces.html
- The Cargo Book, "The Manifest Format":
  https://doc.rust-lang.org/cargo/reference/manifest.html
- Rust book, Chapter 14.3 "Cargo Workspaces".

---

### 5.2 `crates/kineserve/` — the model server supervisor

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

**How it connects.** See Section 6.1 for the child-process and HTTP details.

---

### 5.3 `crates/kineserve/src/input.rs` and `crates/k-core/src/input.rs`

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

### 5.4 `crates/koil/` — the channel and the trace recorder

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
  Section 9, items 4 and 5).

**Rust study.**
- Rust book, Chapter 8.3 "Storing Keys with Associated Values in Hash Maps".
- Rust book, Chapter 12.2 and 12.4 — read and write files well.
- Rust book, Chapter 16 "Fearless Concurrency" — if Koil records traces on a
  separate thread, read 16.1 (threads), 16.2 (channels), and 16.3 (`Arc` and
  `Mutex` for shared state).
- Brown fork, Chapter 4 whole chapter — Koil holds data that more than one part
  reads; ownership and borrow rules matter most here.

**How it connects.** See Section 6.2 for WireGuard. See Section 7 for the trace
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

### 5.5 `crates/k-core/` — the ReAct loop and function routing

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

### 5.6 `models/`

**Purpose.** Holds your model file in GGUF format. Choose a text-to-text model. A
small coding model or agent model is a good start. Copy or clone the file here
before you run Kineserve.

**Study.** GGUF is the file format that llama.cpp reads. Read the format note in
the ggml repository so you pick a file that `llama-server` can load:
https://github.com/ggml-org/ggml/blob/master/docs/gguf.md

---

### 5.7 `traces/`

**Purpose.** Holds the audit log. `traces/logs/<trace-id>.json` is one trace per
message. `traces/trace_hash.json` maps each ID to its file. A trace is immutable
after Koil writes it.

**Standard library tools.** `std::fs`, `std::io::Write`, `std::collections::HashMap`.

**Rust study.** Rust book, Chapter 8.3 (hash maps) and Chapter 12.2 (file I/O).

**Decisions.** The trace schema is in Section 7. The ID scheme is open; see
Section 9, items 4 and 5.

---

### 5.8 `kinesin.toml`

**Purpose.** One file with two parts. The top part is a TOML header for
deterministic settings. The lower part is a Markdown body for the agent
instructions. Think of it as a "CLAUDE.md meets a config file". The program reads
this file; the program treats it as the source of truth.

**Standard library tools.** `std::fs` to read the file. String methods to split
the two parts.

**Rust study.**
- Rust book, Chapter 8.2 (strings) and Chapter 12.2 (read a file).
- The TOML specification, to learn the format you will read:
  https://toml.io/en/v1.0.0
- The CommonMark specification, for the Markdown body:
  https://spec.commonmark.org/

**Decision.** The standard library has no TOML parser. See Section 9, item 3.

#### 5.8.1 Bare-basics skeleton

Start with this small set of settings. Each key maps to a value that another part
already needs (Section 6.1 for the server, Section 7 for the traces, Section 8 for
the loop). Add more settings only when a part needs them.

```toml
# kinesin.toml
# The single source of truth for one Kinesin run.
# The file has two parts:
#   1. The TOML settings below.
#   2. The Markdown instructions after the "+++ instructions +++" line.
# The program reads this file. The program does not change it.

# A version number for the settings format. Raise it when you change the shape.
version = 1

[model]
# The model file inside models/. Kineserve gives this path to llama-server (-m).
path = "models/your-model.gguf"
# The context size in tokens (the -c flag for llama-server).
context_size = 4096

[server]
# The address and port for llama-server. Bind to localhost only.
host = "127.0.0.1"
port = 8080
# The endpoint that K-Core uses. Choose one and keep to it:
#   "/completion"          -> the simple prompt-in, text-out endpoint.
#   "/v1/chat/completions" -> the chat and tool-call endpoint.
endpoint = "/completion"
# The time to wait for the /health check to pass, in seconds.
startup_timeout_s = 120

[sampling]
# The maximum number of tokens for one answer (n_predict).
max_tokens = 256
temperature = 0.2
# Read one whole answer at first. Set to true later for token-by-token output.
stream = false

[loop]
# The maximum number of ReAct steps for one run. This stops a runaway loop.
max_steps = 12
# Stop if the same action and the same arguments repeat this many times.
repeat_limit = 3

[traces]
# The directory for the trace files and the lookup table.
dir = "traces"
# Write a trace for every message across Koil.
enabled = true

[koil]
# The channel mode:
#   "direct"    -> no tunnel; connect over 127.0.0.1 (good for the first run).
#   "wireguard" -> use a WireGuard tunnel (Section 6.2).
mode = "direct"

+++ instructions +++

You are Kinesin, a small agent harness.
Return one JSON object for each step. The object has an "action" field and an
"args" field. When you finish, return the final answer as plain text with no
action.

(Write the full agent instructions here, in Markdown.)
```

#### 5.8.2 The split rule

The file is **not** pure TOML. The Markdown body below the marker is not valid
TOML. So the reader works in two steps:

1. Read the whole file as text (`std::fs`).
2. Split the text at the first `+++ instructions +++` line. The text above is the
   settings. The text below is the instructions.
3. Parse the settings part as TOML. Keep the instructions part as Markdown text.

This is like the "front matter" pattern in static site tools: a settings header,
then a prose body. If you prefer to keep the file as pure TOML instead, put the
instructions inside a TOML multi-line literal string (`instructions = '''...'''`).
That choice lets a standard TOML parser read the whole file, but the prose is
harder to write and to read. Pick one style and keep to it.

**Immutable source of truth.** After a run starts, make a hash of the file and
store it with the run (for example in the first trace). Later, you can compare the
current file against that stored hash to see if the settings changed. This gives
the "compare against after compilation" check from the project goal.

#### 5.8.3 Future settings and how to add them

Add these later, one at a time. For each one, the pattern is the same: add the
key, decide which crate reads it and when, then make the small code change. Some
of them raise the work for your hand-written TOML reader; those notes point back
to Section 9, item 3 (hand-write a subset, or add the `toml` crate).

- **Extra sampling controls.** Add `top_k`, `top_p`, `repeat_penalty`, `seed`, and
  `stop` to `[sampling]`. Read them in K-Core. Pass them straight into the request
  payload (Section 6.1). A fixed `seed` gives repeatable runs, which fits the
  deterministic goal.

- **Function allow-list.** Add a `[functions]` table with `allow = ["list_files",
  "read_file"]` and, if you want, `deny`. Read it in K-Core. Check the action name
  against the list at loop step 6, before you call the function (Section 8.1). Add
  a `confirm = ["write_file"]` key for actions that change data, and stop for a
  confirmation before you run them.

- **Timeouts and retries.** Add `request_timeout_s` and `retries` to `[server]`.
  Read them in K-Core. Set a read timeout on the `TcpStream` with
  `set_read_timeout`, and retry the request in a small loop.

- **Logging.** Add a `[logging]` table with `level = "info"` and `to_file = false`.
  Read it in every crate. Write log lines to standard error (`std::io`), and skip
  lines below the chosen level. This is simpler than a logging library and stays
  standard library only.

- **Trace retention.** Add `max_files` or `keep_days` to `[traces]`. Read it in
  Koil at start-up. Scan `traces/logs/` and remove old files by their modified
  time (`std::fs` metadata). Note the tension: a trace is an immutable audit
  record, so deletion works against that goal. Decide this on purpose.

- **Prompt template.** Add a `[prompt]` table with `system_prefix` and a
  `template` string. Read it in K-Core. Build each prompt from the template plus
  the conversation. This matters most for the chat endpoint and its message roles.

- **WireGuard details (for Koil Path A or B).** Add a `[koil.wireguard]` table with
  `interface`, `peer_public_key`, `endpoint`, `allowed_ips`, and
  `persistent_keepalive`. Read it in Koil. For Path A, write the `wg` config file
  from these keys. For Path B1, pass them to the GotaTun library (Section 6.2).

- **Model profiles.** Add an array of tables, `[[model.profile]]`, each with a
  `name`, a `path`, and a `context_size`. Add `active_profile = "..."`. Read the
  chosen profile in Kineserve. Note: an array of tables (`[[...]]`) is harder to
  parse by hand, so this is a good point to weigh the `toml` crate (Section 9,
  item 3).

- **Parallel sessions.** Add `parallel_sessions` to `[loop]`. Kineserve passes the
  matching `-np` flag to llama-server. K-Core runs each session on its own thread
  (Rust book, Chapter 16). This also needs a session ID in every trace, which the
  schema already has (Section 7).

- **Secrets — do not store them here.** Never put a private key or a token in
  `kinesin.toml` if you commit the file. Instead, name the environment variable in
  the config (for example `api_key_env = "KINESIN_API_KEY"`), and read the value
  at run time with `std::env::var`. This keeps the secret out of the repository and
  out of the traces.

**How to grow the reader safely.** Each new key means one more field to parse and
one more default value. Give every setting a default in code, so an old
`kinesin.toml` still works after you add a key. Raise `version` when you change the
shape in a way that breaks old files, and check `version` on load.

---

### 5.9 `scripts/`

**Purpose.** Set up and start order. `preflight.ps1` checks the environment. It
detects Windows, checks or installs WSL, and installs `llama.cpp`, WireGuard, and
the other non-Rust dependencies with the correct package manager.
`run-harness.ps1` starts the three parts in the correct order.

**Study.**
- PowerShell scripting overview:
  https://learn.microsoft.com/powershell/scripting/overview
- WSL install and commands: https://learn.microsoft.com/windows/wsl/install
- Section 6.3 explains WSL and the package steps.

**Decision.** These scripts are Windows and PowerShell. Decide the Linux and
macOS equivalent, or move the checks into the Rust binary. See Section 9, item 10.

---

### 5.10 `.github/workflows/`

**Purpose.** Continuous integration. Run the format check, the lint, the build,
and the tests on each push.

**Study.**
- GitHub Actions quickstart:
  https://docs.github.com/actions/quickstart
- The three Rust commands to run in CI: `cargo fmt --check`, `cargo clippy`, and
  `cargo test`.
- Rust book, Chapter 11 "Writing Automated Tests" — write the tests that CI runs.

---

## 6. The non-Rust parts and how to connect them

Kinesin joins Rust code to three outside tools: `llama-server`, WireGuard, and
WSL. This section explains each link.

### 6.1 Rust to `llama-server` (the most important link)

`llama-server` is a C++ program. It is a normal HTTP server. Your Rust code does
**not** call C++ functions. Your Rust code starts the server and then talks to it
over local HTTP. This keeps the two languages fully separate.

**Step 1 — Start the server.** Kineserve runs `llama-server` with
`std::process::Command`. Set these flags:

- `-m` — the path to your model in `models/`.
- `--host 127.0.0.1` — bind to localhost only.
- `--port 8080` — the port (the default is 8080).
- `-c` — the context size.

**Step 2 — Wait for ready.** The model takes time to load. Do not send a request
too early. Poll `GET /health` until it returns "ok". Then send real requests.

**Step 3 — Send a request.** The standard library has no HTTP client. You write a
small one over `std::net::TcpStream`:

1. Open a `TcpStream` to `127.0.0.1:8080`.
2. Write the request line, for example `POST /completion HTTP/1.1`.
3. Write the headers. You must send `Host:`, `Content-Type: application/json`,
   and `Content-Length:` with the exact byte length of the body.
4. Write a blank line.
5. Write the JSON body.
6. Read the response bytes. Find the blank line that ends the headers. The JSON
   body follows it.

**Step 4 — Choose the endpoint.**

- `POST /completion` takes a single `prompt` field and returns a `content` field.
  It is the simplest to start with.
- `POST /v1/chat/completions` takes a `messages` array of role and content pairs.
  It matches the chat shape and the OpenAI format. Use it when you need multi-turn
  chat.

**Key request fields:** `prompt` (or `messages`), `n_predict` (max tokens),
`temperature`, and `stream`. Set `stream` to false at first, so you read one
whole answer.

**Key response fields:** `content` (the text), `stop_type` (why it stopped), and
`timings` (performance). For the chat endpoint, read
`choices[0].message.content`.

**Study.**
- llama.cpp server README (endpoints, flags, and fields):
  https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md
- llama.cpp main README (how to build `llama-server`):
  https://github.com/ggml-org/llama.cpp
- MDN "HTTP Messages" (the request and response structure) and "Content-Length":
  https://developer.mozilla.org/docs/Web/HTTP/Messages
- Rust book, Chapter 21.1 — the same read-and-parse pattern for a raw HTTP socket.

### 6.2 Rust to WireGuard (Koil)

There are two ways to build Koil. Pick one.

**Path A — Wrap the system tools (recommended start).** WireGuard has two
command-line tools: `wg` and `wg-quick`. Koil uses `std::process::Command` to run
them. Koil makes the keys, writes a config file, and runs `wg-quick up`. This
path is small Rust code and real, tested WireGuard.

**Path B — A userspace implementation in Rust (advanced).** Koil embeds the
WireGuard protocol in the Rust process. No system tools and no `wg-quick` are
needed for the tunnel logic. WireGuard needs cryptography: Curve25519,
ChaCha20-Poly1305, BLAKE2s, and the Noise handshake. The standard library has no
cryptography. So this path needs an outside library; it cannot be standard
library only. There are two ways:

- **B1 — Use a userspace WireGuard crate (recommended for Path B).** Add a Rust
  library that already implements the protocol, and call it from Koil. Two good
  options:
  - **GotaTun** — a userspace WireGuard in Rust from Mullvad, forked from
    BoringTun and now their standard for desktop. https://github.com/mullvad/gotatun
    and the crate at https://lib.rs/crates/gotatun . This fits your Rust-first
    goal, and it means Koil does not hand-write crypto.
  - **boringtun** — Cloudflare's userspace WireGuard in Rust, the parent of
    GotaTun. https://github.com/cloudflare/boringtun
  - Note: the tunnel still needs a TUN network device. That step may need extra
    privileges on the host.
- **B2 — Hand-write the protocol.** You write the handshake and the crypto calls
  yourself. This is a large task and is easy to get wrong. Do not start here; read
  GotaTun and boringtun as study references instead.

**The rule: use WireGuard, do not rewrite it.** The word "custom" in the first
draft meant a custom *connector*, not a custom *protocol*. Path A and Path B1 both
give you the private Koil-to-Koil link that the design needs. Path B2 gives you
nothing more, and it puts security-critical code in your hands. Choose Path A or
Path B1.

Suggested order for Koil: Path A first (small and tested), then Path B1 with
GotaTun if you want the tunnel inside the Rust process and a Koil that needs no
outside tools.

**Reachability.** WireGuard moves with a peer when its address changes. But it
does not open a path through NAT by itself. The link is simple when one side has
a fixed, reachable address, and the other side calls out to it. So make the
bigger machine the reachable side: give it a public address or a forwarded port,
and let your local Koil start the connection. Two machines that both sit behind
home routers need a relay, which is work you do not need.

**Study.**
- WireGuard Quick Start (keys, config, `wg-quick up`):
  https://www.wireguard.com/quickstart/
- WireGuard Conceptual Overview (cryptokey routing, peers, allowed IPs):
  https://www.wireguard.com/#conceptual-overview
- The `wg` and `wg-quick` man pages (the exact flags Koil will call):
  https://man7.org/linux/man-pages/man8/wg.8.html and
  https://man7.org/linux/man-pages/man8/wg-quick.8.html
- WireGuard Protocol & Cryptography and the Whitepaper (only for Path B):
  https://www.wireguard.com/protocol/ and https://www.wireguard.com/papers/wireguard.pdf

### 6.3 Windows to Linux tools (WSL)

`llama.cpp` and the WireGuard tools run more simply on Linux. On Windows, use
WSL2 (Windows Subsystem for Linux, version 2). WSL2 runs a real Linux kernel next
to Windows.

**The `preflight.ps1` flow:**

1. Detect the operating system. If it is Windows, continue. If it is Linux, use
   the package manager directly.
2. Check for WSL. If it is missing, run `wsl --install`.
3. Inside WSL, update the package list.
4. Inside WSL, install the build tools, clone and build `llama.cpp`, and install
   `wireguard-tools` with `apt`.

**Networking note.** WSL2 uses its own virtual network. A server that listens
inside WSL2 is often reachable from Windows on `localhost`, but not always. If
Kineserve runs inside WSL and a Windows-side part connects to it, read the WSL
networking page first:
https://learn.microsoft.com/windows/wsl/networking

**Study.**
- WSL install: https://learn.microsoft.com/windows/wsl/install
- WSL basic commands: https://learn.microsoft.com/windows/wsl/basic-commands
- WSL file system interop (reach Windows files from Linux and the reverse):
  https://learn.microsoft.com/windows/wsl/filesystems

---

## 7. Trace schema

A trace is the record of one message that crosses Koil. Koil writes one trace as
one JSON file in `traces/logs/`. The file name is the trace ID. The file is
immutable after Koil writes it.

This section gives a first schema. Change it to fit your needs. But keep the field
names stable after you start, because the lookup table and any later tool depend
on them.

### 7.1 Fields of one trace

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `id` | string | yes | The random trace ID. It is also the file name. |
| `time` | string | yes | The time Koil wrote the trace, in ISO 8601 (for example `2026-09-07T14:03:22Z`). |
| `session` | string | yes | The ID of the run that this trace belongs to. One run has many traces. |
| `step` | number | yes | The step number inside the session. It starts at 0 and counts up. |
| `direction` | string | yes | `to_model` or `from_model`. |
| `source` | string | yes | The part that sent the message (for example `k-core`). |
| `destination` | string | yes | The part that received the message (for example `kineserve`). |
| `endpoint` | string | for `to_model` | The `llama-server` path, for example `/completion`. |
| `model` | string | yes | The model file name from `models/`. |
| `payload` | object | yes | The exact JSON body that crossed the channel (see below). |
| `timings` | object | for `from_model` | The timing block from `llama-server`, if it is present. |
| `prev` | string or null | yes | The ID of the trace before this one in the session, or null for the first. |

The `payload` field is the key to the phrase "works with `llama-server`". For a
`to_model` trace, the payload is the request that you sent to `llama-server`, with
no change. For a `from_model` trace, the payload is the answer that `llama-server`
returned, with no change. So a trace holds the real `llama-server` body plus the
metadata around it. The `prev` field links the traces in order, so you can replay
a session from first to last.

### 7.2 Example trace to the model

```json
{
  "id": "b1c4f9a2e8d74630",
  "time": "2026-09-07T14:03:22Z",
  "session": "9f2a77c0",
  "step": 0,
  "direction": "to_model",
  "source": "k-core",
  "destination": "kineserve",
  "endpoint": "/completion",
  "model": "your-model.gguf",
  "payload": {
    "prompt": "System instructions...\nUser: list the files\n",
    "n_predict": 256,
    "temperature": 0.2,
    "stream": false
  },
  "prev": null
}
```

### 7.3 Example trace from the model

```json
{
  "id": "7d0e5a13c9b28f44",
  "time": "2026-09-07T14:03:24Z",
  "session": "9f2a77c0",
  "step": 1,
  "direction": "from_model",
  "source": "kineserve",
  "destination": "k-core",
  "model": "your-model.gguf",
  "payload": {
    "content": "{\"action\": \"list_files\", \"args\": {\"path\": \".\"}}",
    "stop_type": "eos"
  },
  "timings": {
    "prompt_n": 42,
    "predicted_n": 18,
    "predicted_ms": 640.5
  },
  "prev": "b1c4f9a2e8d74630"
}
```

### 7.4 The lookup table `trace_hash.json`

`traces/trace_hash.json` maps each trace ID to its file. It gives quick access
without a scan of the directory. A simple shape is one object. Each key is a trace
ID. Each value holds the file path, the session, the step, and the time.

```json
{
  "b1c4f9a2e8d74630": {
    "file": "traces/logs/b1c4f9a2e8d74630.json",
    "session": "9f2a77c0",
    "step": 0,
    "time": "2026-09-07T14:03:22Z"
  },
  "7d0e5a13c9b28f44": {
    "file": "traces/logs/7d0e5a13c9b28f44.json",
    "session": "9f2a77c0",
    "step": 1,
    "time": "2026-09-07T14:03:24Z"
  }
}
```

Rules for the recorder:

- Write the trace file first. Add the lookup entry second. This order makes sure
  the table never points to a missing file.
- Do not change a trace after you write it. To correct a record, write a new trace
  and link it with `prev`.
- The ID scheme (random bytes or a hash) is a separate decision. See Section 9,
  items 4 and 5.

---

## 8. The ReAct loop

K-Core runs the ReAct loop. "ReAct" means reason, then act. The model reasons in
text. Then it asks for an action. K-Core runs the action, observes the result, and
gives the result back to the model. The loop repeats until a stop condition is
true.

### 8.1 The step cycle

One step of the loop does these things in order:

1. Build the prompt. Join the instructions from `kinesin.toml`, the conversation
   so far, and the last observation.
2. Send the prompt to the model through Koil (Section 6.1). Record a `to_model`
   trace.
3. Read the answer. Record a `from_model` trace.
4. Look for an action in the answer.
5. If there is no action, treat the answer as the final answer. Stop the loop.
6. If there is an action, find the function for that action name in the function
   table.
7. If the name is not in the table, make an error observation. Go to step 10.
8. Check the arguments against what the function needs. If the arguments are
   wrong, make an error observation. Go to step 10.
9. Call the function. Capture its result or its error as the observation.
10. Add the action and the observation to the conversation.
11. Add 1 to the step count. Go to step 1.

### 8.2 The loop states

Model the loop with a small set of states. An `enum` fits well (Rust book,
Chapter 6):

- `Think` — build and send the prompt; wait for the answer.
- `Act` — a valid action is present; call the function.
- `Observe` — record the result and add it to the conversation.
- `Done` — a stop condition is true; return the final answer.
- `Failed` — an error stops the loop; return the error.

### 8.3 How the model asks for an action

Decide one clear format for an action. Put the rule in `kinesin.toml`, so the
model follows it. A simple rule is: the model returns one JSON object with an
`action` field and an `args` field. For example:

```json
{
  "action": "list_files",
  "args": { "path": "." }
}
```

K-Core reads the `content` field of the answer. Then it parses this small JSON. If
the parse fails, K-Core makes an error observation and lets the model try again.

Note: this format uses the plain `/completion` endpoint. The
`/v1/chat/completions` endpoint has its own tool-call fields. Pick one endpoint
and one format, and keep to it.

### 8.4 Stop conditions

The loop must not run forever. Stop when any of these is true:

- The answer has no action. This is the normal, successful end.
- The step count reaches the maximum. Set the maximum in `kinesin.toml`, for
  example 12 steps.
- The same action and the same arguments repeat too many times. This shows a
  stuck loop.
- A function returns a stop result on purpose (for example a `finish` function).
- An error happens that you cannot recover from (for example Kineserve is not
  reachable).

### 8.5 What to record

Record a trace for every message to and from the model (Section 7). Also decide
whether to record the function results. A good first rule: put the observation
text inside the next `to_model` trace, because the observation becomes part of the
next prompt. This keeps the full history in the traces.

### 8.6 Errors in the loop

Return a `Result` from each function (Rust book, Chapter 9). Turn a function error
into an observation, not a crash, so the model can react to it. Stop the loop only
for an error that you cannot recover from. For that case, write a `Failed` trace
with the reason.

Rust study for this section: Chapter 5 (structs for the message and the action),
Chapter 6 (enums and `match` for the states), Chapter 9 (`Result` and `?`), and
Chapter 13 (iterators to walk the conversation). Read the Brown fork, Chapter 4.3
"Fixing Ownership Errors", before you pass the conversation between functions.

---

## 9. What is missing: decisions to make first

These are the open points. Decide each one before you write the matching code.
Each item says the problem, the choices, and a suggested start.

1. **Workspace or one binary.** The first draft mixed a root `src/main.rs` with
   three sub-crates. That is unclear. Choose a Cargo workspace with three member
   crates (Section 4), **or** one binary crate with three modules. Suggested
   start: the workspace, because the three parts have different jobs and may run
   as separate processes. Confirm this choice, then build the root `Cargo.toml`
   as a workspace manifest.

2. **JSON.** The standard library has no JSON parser. Both `llama-server` and the
   traces use JSON. Choose:
   - **Hand-write a small encoder and decoder** for the few shapes you use. This
     is more work but more learning, and it keeps the "standard library only"
     rule. The JSON grammar is small; read https://www.json.org/ .
   - **Add `serde` and `serde_json`.** This is the common, safe choice. It breaks
     the "standard library only" rule for one clear reason.
   - Suggested start: hand-write a small encoder for requests and a small decoder
     that reads only the few response fields you need. Move to `serde_json` later
     if the hand-written code becomes a burden.

3. **TOML.** The standard library has no TOML parser. `kinesin.toml` needs one.
   Choose:
   - **Hand-write a reader** for the small subset you use (key, value, and
     section headers). The config is small and under your control.
   - **Add the `toml` crate.**
   - Suggested start: hand-write the subset reader, because you control the file.

4. **Randomness for trace IDs.** The trace ID is random, not sequential. The
   standard library has no random number generator. Choose the entropy source:
   - **Read the operating system randomness.** On Linux, read bytes from
     `/dev/urandom` with `std::fs`. This is standard library only, but Linux only.
   - **Add the `getrandom` crate** for a cross-platform source.
   - **Derive an ID** from a hash of the timestamp plus a counter. This is
     standard library only and cross-platform, but it is weaker and can collide.
   - Suggested start: read `/dev/urandom` inside WSL/Linux for the first version.

5. **Hashing for the lookup table.** The standard library has `DefaultHasher` in
   `std::collections::hash_map`. You can use it for a non-cryptographic ID or a
   bucket key. Note one limit: its output is not stable across Rust versions, so
   do not store it as a long-term stable key. Decide whether the trace ID is a
   random value (item 4) or a hash, and write the rule down.

6. **WireGuard scope (settled).** The tunnel stays. The reason is in Section 5.4:
   Kineserve is to run on a bigger remote machine, and the Koil-to-Koil link keeps
   that private. The local setup is the same design with one endpoint. Two points
   remain: write the tunnel with a library or the system tools, never by hand
   (Section 6.2); and build the transport seam in Phase 1, so Phase 5 adds the
   tunnel without a change to K-Core.

7. **Trace schema (drafted).** Section 7 now gives a first schema and examples.
   The open choices that remain: confirm the field names; decide whether to keep
   the full `llama-server` payload or only selected fields; and decide the ID
   scheme with items 4 and 5.

8. **The ReAct loop rules (drafted).** Section 8 now gives the step cycle, the
   states, the action format, and the stop conditions. The open choices that
   remain: set the maximum step count in `kinesin.toml`; pick the action format
   (the `/completion` JSON object or the `/v1/chat/completions` tool calls); and
   set the rule for a stuck loop.

9. **The config and instruction split (drafted).** Section 5.8 now gives the
   skeleton, the divider rule (`+++ instructions +++`), and the "immutable source
   of truth" hash check. The open choices that remain: pick the split style (the
   divider, or a TOML multi-line string) and confirm the setting names.

10. **Cross-platform start.** `preflight.ps1` and `run-harness.ps1` are Windows
    and PowerShell. Decide the Linux and macOS path: a shell script, or checks
    inside the Rust binary. Suggested start: keep the PowerShell scripts for
    Windows, and add a short shell script for Linux later.

11. **Error strategy.** Decide how each crate reports errors. A common pattern is
    a custom `enum` per crate plus `Result`. Read Rust book, Chapter 9. Decide how
    the top level prints an error and what exit code it returns.

12. **Tests.** There is no test plan yet. Decide the unit tests per module and
    put integration tests in a `tests/` directory in each crate. Read Rust book,
    Chapter 11.

---

## 10. Roadmap

This section tells you where to start and how to build Kinesin out.

**Phase 1 is the work to do now.** Phases 2 to 6 are later work. They are in this
document for one reason: they tell you which seams Phase 1 must have. Read them
once before you start, then build Phase 1.

Finish a phase before you start the next one. Each phase gives a goal, the steps,
and the reason it comes at this point.

| Phase | Goal | State |
|-------|------|-------|
| 1 | One prompt in, one answer out, one trace on disk | Build now |
| 2 | The loop and the tools make it an agent | Later |
| 3 | Long runs do not break | Later |
| 4 | The trace log becomes useful | Later |
| 5 | Kineserve moves to a bigger machine | Later |
| 6 | Change the harness without fear | Later |

---

### Phase 1 — Foundations (start here)

**Goal.** Send one prompt to a local model. Receive one answer. Write one trace.

1. **Set up the workspace.** Make the root `Cargo.toml` and one "hello" binary in
   `k-core`. Confirm the build runs. (Cargo Book, "Workspaces"; Rust book Ch 1, 7.)
2. **Add a shared library crate** for the common types: the message, the trace,
   and the config. Every binary depends on this one crate. This stops the same
   struct from appearing in two places. (Rust book Ch 7.)
3. **Write Kineserve.** Start `llama-server` as a child process. Poll `/health`
   until it returns "ok". (std::process, std::net; Rust book Ch 21; Section 6.1.)
4. **Write the small HTTP client in K-Core.** Send one `/completion` request over
   a `TcpStream`. Print the raw answer. (Rust book Ch 21.1; MDN HTTP.)
5. **Add the small JSON encode and decode.** Encode the request. Decode the
   `content` field of the answer. (Section 9, item 2.)
6. **Define the transport seam in Koil.** Give it one implementation, `direct`,
   which passes messages across `127.0.0.1`. Read the seam note in Section 5.4
   before you write this.
7. **Write traces.** Use the schema in Section 7. Write each trace to
   `traces/logs/`. Build the `trace_hash.json` lookup table. (Rust book Ch 8, 12.)
8. **Read `kinesin.toml`.** Read the settings and the instructions. Split the two
   parts at the divider. (Section 5.8.)
9. **Add the first tests.** Test the JSON codec and the config reader. These two
   are easy to test and easy to get wrong. (Rust book Ch 11.)

**The one thing to get right.** The transport seam in step 6. Everything else in
Phase 1 you can rewrite cheaply. A wrong seam costs you the whole of Phase 5.

---

### Phase 2 — The loop and the tools

**Goal.** Make it an agent. The model chooses an action, and K-Core runs it.

1. **Build the ReAct loop.** Use the step cycle and the states in Section 8.
2. **Design the tool interface.** This is the least designed part of Kinesin, and
   it is the product surface of a general purpose harness. Decide four things:
   - how you declare a tool: its name, its description, its argument names, and
     its argument types;
   - how the tool list reaches the model: in the instructions, or as a schema;
   - how you check the arguments before you call the tool;
   - what shape a tool result has, and how a tool error becomes an observation.
3. **Add the capability model.** The model chooses the actions, so the harness
   must bound them. Add an allow-list of tool names. Add a confirm step for any
   tool that changes data. (Section 5.8.3.)
4. **Add two or three real tools.** Keep them small: read a file, list a
   directory, and finish.
5. **Add the stop conditions and the step limit.** (Section 8.4.)

**Why here.** The tools make the harness general purpose. Design the tool
interface once and early, because every tool you add later takes its shape.

---

### Phase 3 — Survive a real session

**Goal.** A long run does not break.

1. **Add context management.** The conversation grows at every step, and the model
   has a fixed `context_size`. Decide what happens at the limit: remove the oldest
   messages, replace them with a summary, or stop with a clear error. A small
   local model reaches this limit fast, so do not leave this out.
2. **Cap the size of a tool result** before it goes into the prompt. One large
   file can fill the context in a single step.
3. **Add timeouts and retries** around the model call. (Section 5.8.3.)
4. **Finish the error path.** A tool error becomes an observation. Only an error
   that you cannot recover from stops the loop. (Section 8.6.)

**Why here.** Phases 1 and 2 give you short runs that work. This phase is what
makes a run of twelve steps as safe as a run of two.

---

### Phase 4 — Inspect and replay

**Goal.** Make the trace log useful. Until now you only write traces. Nothing
reads them.

1. **Write a trace reader.** Rebuild one session from `trace_hash.json` and the
   `prev` chain. Print the steps in order.
2. **Add a run record.** A run has a start time, an end time, the config hash, the
   model name, the step count, and the result. A person needs one summary, not
   many files.
3. **Add replay.** Feed the recorded answers back into the loop in place of a live
   model. The loop must behave the same way.

**Why this pays.** You already record everything that replay needs. This phase
adds no new data. It turns the audit log into a test tool, which Phase 6 then
uses.

---

### Phase 5 — Distributed: the Koil-to-Koil link

**Goal.** Run Kineserve on a bigger machine. Serve more sessions. Keep the link
private.

1. **Add the second transport implementation**, `wireguard`, behind the seam from
   Phase 1. K-Core does not change.
2. **Use a WireGuard library or the system tools.** Do not write the protocol
   yourself. (Section 6.2, Path A or Path B1.)
3. **Make the bigger machine the reachable side.** Your local Koil calls out to
   it. (Section 6.2, "Reachability".)
4. **Put the concurrency work in K-Core and Kineserve.** Kineserve passes `-np` to
   `llama-server`. K-Core gives each session an ID and matches each answer to its
   request. Koil stays a private pipe. (Rust book Ch 16.)

---

### Phase 6 — Trust it

**Goal.** Change the harness without fear.

1. **Keep a set of recorded sessions as golden traces.** Replay them after each
   change and compare the result. This is your regression test, and it comes free
   from Phase 4.
2. **Add session resume.** The trace chain is close to an event log already.
   Decide whether a run can restart from its last trace after a crash. This
   matters more after Phase 5, because a remote machine can drop.
3. **Add the config hash check.** Compare the current `kinesin.toml` against the
   hash stored with the run. (Section 5.8.2.)
4. **Pin a sampling `seed`** so that a run repeats. (Section 5.8.3.)
5. **Finish the scripts and CI.** `preflight.ps1`, `run-harness.ps1`, and the
   workflow that runs `cargo fmt --check`, `cargo clippy`, and `cargo test`.

---

## 11. Learning resources index

### Rust core

- **The Rust Programming Language (standard):** https://doc.rust-lang.org/book/
  - Ch 2 (guessing game: first `stdin`), Ch 3 (common concepts).
  - Ch 5 (structs), Ch 6 (enums and `match`), Ch 7 (modules).
  - Ch 8 (collections: 8.1 `Vec`, 8.2 `String`, 8.3 `HashMap`).
  - Ch 9 (error handling: `Result` and `?`).
  - Ch 11 (tests), Ch 12 (I/O project: args, files, stderr).
  - Ch 13 (iterators and closures).
  - Ch 15 (smart pointers: `Box`, `Rc`, `RefCell` — only if you share state).
  - Ch 16 (concurrency: 16.1 threads, 16.2 channels, 16.3 `Arc`/`Mutex`).
  - Ch 21 (final project: a std-only multithreaded web server — the HTTP model).
- **The Rust Programming Language (Brown University fork):**
  https://rust-book.cs.brown.edu/
  - Ch 4 (Understanding Ownership, expanded): 4.1 What is Ownership?, 4.2
    References and Borrowing, 4.3 Fixing Ownership Errors, 4.4 The Slice Type, 4.5
    Ownership Recap (the Read/Write/Own permission model). Read this Ch 4 first.
- **Rust by Example:** https://doc.rust-lang.org/rust-by-example/
  - Std Misc → child processes, threads, channels, file I/O. Fills the gaps that
    the book skips for process spawning.
- **Standard library API docs:** https://doc.rust-lang.org/std/
  - `std::process::Command`, `std::net::{TcpStream, TcpListener}`,
    `std::io::{Read, Write, BufReader, BufRead}`, `std::fs`, `std::thread`,
    `std::sync::{Arc, Mutex, mpsc}`, `std::collections::HashMap`, `std::time`.
- **The Cargo Book:** https://doc.rust-lang.org/cargo/
  - "Workspaces" and "The Manifest Format".

### Non-Rust dependencies

- **llama.cpp / `llama-server`:** https://github.com/ggml-org/llama.cpp
  - Server README (endpoints, flags, JSON fields):
    https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md
  - Read: start flags (`-m`, `--host`, `--port`, `-c`); endpoints (`/health`,
    `/completion`, `/v1/chat/completions`, `/props`, `/tokenize`); fields
    (`prompt`, `n_predict`, `temperature`, `stream`; `content`, `stop_type`,
    `timings`).
- **GGUF format:** https://github.com/ggml-org/ggml/blob/master/docs/gguf.md
- **WireGuard:** https://www.wireguard.com/
  - Quick Start; Conceptual Overview; `wg` and `wg-quick` man pages; Protocol &
    Cryptography and the Whitepaper (only for a userspace Koil).
  - **GotaTun** (Mullvad userspace WireGuard in Rust; a fork of BoringTun; use as
    a crate for Path B1): https://github.com/mullvad/gotatun and
    https://lib.rs/crates/gotatun
  - `boringtun` (Cloudflare userspace WireGuard in Rust; the parent of GotaTun):
    https://github.com/cloudflare/boringtun
- **WSL:** https://learn.microsoft.com/windows/wsl/
  - Install; basic commands; networking; file system interop.
- **PowerShell:** https://learn.microsoft.com/powershell/scripting/overview
- **GitHub Actions:** https://docs.github.com/actions/quickstart

### Formats and background

- **HTTP messages (MDN):** https://developer.mozilla.org/docs/Web/HTTP/Messages
- **JSON grammar:** https://www.json.org/
- **TOML specification:** https://toml.io/en/v1.0.0
- **CommonMark (Markdown):** https://spec.commonmark.org/
- **ReAct paper (the loop idea):** https://arxiv.org/abs/2210.03629
