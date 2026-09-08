# Configuration: `kinesin.toml`

## `kinesin.toml`

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

**Decision.** The standard library has no TOML parser. See [decisions.md](decisions.md), item 3.

### Bare-basics skeleton

Start with this small set of settings. Each key maps to a value that another part
already needs ([integration.md](integration.md), llama-server for the server, [traces.md](traces.md) for the traces, [loop-and-tools.md](loop-and-tools.md) for
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
#   "wireguard" -> use a WireGuard tunnel ([integration.md](integration.md), WireGuard).
mode = "direct"

+++ instructions +++

You are Kinesin, a small agent harness.
Return one JSON object for each step. The object has an "action" field and an
"args" field. When you finish, return the final answer as plain text with no
action.

(Write the full agent instructions here, in Markdown.)
```

### The split rule

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

### Future settings and how to add them

Add these later, one at a time. For each one, the pattern is the same: add the
key, decide which crate reads it and when, then make the small code change. Some
of them raise the work for your hand-written TOML reader; those notes point back
to [decisions.md](decisions.md), item 3 (hand-write a subset, or add the `toml` crate).

- **Extra sampling controls.** Add `top_k`, `top_p`, `repeat_penalty`, `seed`, and
  `stop` to `[sampling]`. Read them in K-Core. Pass them straight into the request
  payload ([integration.md](integration.md), llama-server). A fixed `seed` gives repeatable runs, which fits the
  deterministic goal.

- **Function allow-list.** Add a `[functions]` table with `allow = ["list_files",
  "read_file"]` and, if you want, `deny`. Read it in K-Core. Check the action name
  against the list at loop step 6, before you call the function ([loop-and-tools.md](loop-and-tools.md), the step cycle). Add
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
  from these keys. For Path B1, pass them to the GotaTun library ([integration.md](integration.md), WireGuard).

- **Model profiles.** Add an array of tables, `[[model.profile]]`, each with a
  `name`, a `path`, and a `context_size`. Add `active_profile = "..."`. Read the
  chosen profile in Kineserve. Note: an array of tables (`[[...]]`) is harder to
  parse by hand, so this is a good point to weigh the `toml` crate ([decisions.md](decisions.md),
  item 3).

- **Parallel sessions.** Add `parallel_sessions` to `[loop]`. Kineserve passes the
  matching `-np` flag to llama-server. K-Core runs each session on its own thread
  (Rust book, Chapter 16). This also needs a session ID in every trace, which the
  schema already has ([traces.md](traces.md)).

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

