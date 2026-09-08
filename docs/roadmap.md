# Roadmap

This section tells you where to start and how to build Kinesin out.

**Phase 1 is the work to do now.** Phases 2 to 6 are later work. They are in this
document for one reason: they tell you which seams Phase 1 must have. Read them
once before you start, then build Phase 1.

Finish a phase before you start the next one. Each phase gives a goal, the steps,
and the reason it comes at this point.

Two documents run beside this one. [testing.md](testing.md) says how to test what
each phase adds. [process.md](process.md) says when a phase is ready to start and
when it is done, and it holds the table of what proves each phase.

| Phase | Goal | State |
|-------|------|-------|
| 0 | Prove the environment works | Do this first |
| 1 | One prompt in, one answer out, one trace on disk | Build now |
| 2 | The loop and the tools make it an agent | Later |
| 3 | Long runs do not break | Later |
| 4 | The trace log becomes useful | Later |
| 5 | Kineserve moves to a bigger machine | Later |
| 6 | Change the harness without fear | Later |

---

## Phase 0 — Pre-flight (before you write any Rust)

**Goal.** Prove that the model and the server do what you need, before you build a
harness on top of them. Every step here is done by hand. Each one removes a way
for Phase 1 to fail for a reason that is not your code.

1. **Install Rust.** Confirm that `cargo --version` answers.
2. **Build or install `llama-server`.** Confirm it starts.
3. **Choose a model and put it in `models/`.** Use 8B parameters or more. A Qwen 3
   8B instruct GGUF is the safe first choice. ([components.md](components.md), models/.)
4. **Start the server by hand with `--jinja`.** Keep the KV cache at the default.
   Write down the exact command line that works.
5. **Check `/health`.** Confirm that it returns "ok" after the model loads. Note
   how long the load takes; this sets `startup_timeout_s`.
6. **Send one chat request by hand.** Confirm you receive an answer.
7. **Send one request with a `tools` array.** Confirm that you receive
   `tool_calls` back, and that `finish_reason` is `"tool"`.
8. **Read the server log for the format.** It says whether it used a **native**
   tool format or the **generic** fallback. If it says generic, change the model
   now. This one check can save you a rewrite. ([loop-and-tools.md](loop-and-tools.md), how the model asks for an action.)
9. **Send one request with a `json_schema`.** Confirm the output matches the shape.
10. **Save the working request bodies.** They are the targets that your Phase 1
    code must reproduce.

**Why this phase exists.** Steps 7 and 8 decide the design of your whole tool
layer. If you find out after Phase 2 that your model has no tool template, you
rewrite the loop. If you find out now, you change one file name.

---

## Phase 1 — Foundations (start here)

**Goal.** Send one prompt to a local model. Receive one answer. Write one trace.

1. **Set up the workspace.** Make the root `Cargo.toml` and one "hello" binary in
   `k-core`. Confirm the build runs. (Cargo Book, "Workspaces"; Rust book Ch 1, 7.)
2. **Add a shared library crate** for the common types: the message, the trace,
   and the config. Every binary depends on this one crate. This stops the same
   struct from appearing in two places. (Rust book Ch 7.)
3. **Write Kineserve.** Start `llama-server` as a child process. Poll `/health`
   until it returns "ok". (std::process, std::net; Rust book Ch 21; [integration.md](integration.md), llama-server.)
4. **Write the small HTTP client in K-Core.** Send one `/completion` request over
   a `TcpStream`. Print the raw answer. (Rust book Ch 21.1; MDN HTTP.)
5. **Add the small JSON encode and decode.** Encode the request. Decode the
   `content` field of the answer. ([decisions.md](decisions.md), item 2.)
6. **Define the transport seam in Koil.** Give it one implementation, `direct`,
   which passes messages across `127.0.0.1`. Read the seam note in [components.md](components.md), Koil
   before you write this.
7. **Write traces.** Use the schema in [traces.md](traces.md). Write each trace to
   `traces/logs/`. Build the `trace_hash.json` lookup table. (Rust book Ch 8, 12.)
8. **Read `kinesin.toml`.** Read the settings and the instructions. Split the two
   parts at the divider. ([configuration.md](configuration.md).)
9. **Add the first tests.** Write the JSON codec tests first, from the JSON rules,
   before the codec exists. Then test the config reader and the HTTP request
   builder. Keep every one of them pure, so no test needs a server.
   ([testing.md](testing.md); Rust book Ch 11 and 12.4.)

**Keep the pure core separate from the start.** [testing.md](testing.md) lists the
units in Kinesin and explains why the loop must be a pure function. If you build
Phase 1 that way, then unit tests, replay, and rewind all come from the same
structure. If you mix the logic into the input and output code, you pay for it in
every later phase.

**The one thing to get right.** The transport seam in step 6. Everything else in
Phase 1 you can rewrite cheaply. A wrong seam costs you the whole of Phase 5.

---

## Phase 2 — The loop and the tools

**Goal.** Make it an agent. The model chooses an action, and K-Core runs it.

1. **Move to the chat endpoint and native tool calls.** Send the `tools` array on
   `/v1/chat/completions`, and read `tool_calls` from the reply. Add the
   `json_schema` constraint. ([loop-and-tools.md](loop-and-tools.md), how the model asks for an action, Approach A plus B.)
2. **Build the tool definition.** Use the four parts and the rules in [loop-and-tools.md](loop-and-tools.md), how to define a tool.
   Shape it like an MCP tool so a later move is a mapping, not a rewrite.
3. **Build the ReAct loop.** Use the step cycle and the states in [loop-and-tools.md](loop-and-tools.md).
4. **Add the capability model.** An allow-list of tool names, read-only by
   default, and a confirm step for any tool that changes data. Validate the
   arguments before you call the handler. Treat every tool result as untrusted
   text. ([loop-and-tools.md](loop-and-tools.md), safety.)
5. **Add three real tools.** For example read a file, list a directory, and
   finish. Keep the count small; more tools make the choice harder.
6. **Record the tool events.** Add the `tool_call` and `tool_result` directions to
   the trace chain. ([traces.md](traces.md), tool events.)
7. **Add the stop conditions and the step limit.** ([loop-and-tools.md](loop-and-tools.md), stop conditions.)
8. **Write the minimal trace reader.** Sort the lookup table by `session` and
   `step`, and print the chain in order. This is about twenty lines, and this is
   the phase where multi-step runs start to go wrong, so it earns its place now.
   Full replay stays in Phase 4. ([traces.md](traces.md), manual reading.)

**Why here.** The tools make the harness general purpose. Design the tool
interface once and early, because every tool you add later takes its shape. Expect
the four failure modes in [loop-and-tools.md](loop-and-tools.md), what goes wrong, and treat them as normal rather than as
bugs in your code.

---

## Phase 3 — Survive a real session

**Goal.** A long run does not break.

1. **Add context management.** The conversation grows at every step, and the model
   has a fixed `context_size`. Decide what happens at the limit: remove the oldest
   messages, replace them with a summary, or stop with a clear error. A small
   local model reaches this limit fast, so do not leave this out.
2. **Cap the size of a tool result** before it goes into the prompt. One large
   file can fill the context in a single step.
3. **Add timeouts and retries** around the model call. ([configuration.md](configuration.md), future settings.)
4. **Finish the error path.** A tool error becomes an observation. Only an error
   that you cannot recover from stops the loop. ([loop-and-tools.md](loop-and-tools.md), errors in the loop.)

**Why here.** Phases 1 and 2 give you short runs that work. This phase is what
makes a run of twelve steps as safe as a run of two.

---

## Phase 4 — Inspect and replay

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

## Phase 5 — Distributed: the Koil-to-Koil link

**Goal.** Run Kineserve on a bigger machine. Serve more sessions. Keep the link
private.

1. **Add the second transport implementation**, `wireguard`, behind the seam from
   Phase 1. K-Core does not change.
2. **Use a WireGuard library or the system tools.** Do not write the protocol
   yourself. ([integration.md](integration.md), WireGuard, Path A or Path B1.)
3. **Make the bigger machine the reachable side.** Your local Koil calls out to
   it. ([integration.md](integration.md), WireGuard, "Reachability".)
4. **Put the concurrency work in K-Core and Kineserve.** Kineserve passes `-np` to
   `llama-server`. K-Core gives each session an ID and matches each answer to its
   request. Koil stays a private pipe. (Rust book Ch 16.)

---

## Phase 6 — Trust it

**Goal.** Change the harness without fear.

1. **Keep a set of recorded sessions as golden traces.** Replay them after each
   change and compare the result. This is your regression test, and it comes free
   from Phase 4.
2. **Add session resume.** The trace chain is close to an event log already.
   Decide whether a run can restart from its last trace after a crash. This
   matters more after Phase 5, because a remote machine can drop.
3. **Add the config hash check.** Compare the current `kinesin.toml` against the
   hash stored with the run. ([configuration.md](configuration.md), the split rule.)
4. **Pin a sampling `seed`** so that a run repeats. ([configuration.md](configuration.md), future settings.)
5. **Finish the scripts and CI.** `preflight.ps1`, `run-harness.ps1`, and the
   workflow that runs `cargo fmt --check`, `cargo clippy`, and `cargo test`.

---

