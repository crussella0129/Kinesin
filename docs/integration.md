# The non-Rust parts and how to connect them

Kinesin joins Rust code to three outside tools: `llama-server`, WireGuard, and
WSL. This section explains each link.

## Rust to `llama-server` (the most important link)

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

**Step 5 — Use the server's tool support.** `llama-server` can format and parse
tool calls for you. This matters a lot, and [loop-and-tools.md](loop-and-tools.md), how the model asks for an action explains why. The short
version:

- Start the server with the `--jinja` flag. This makes the server use the model's
  own chat template.
- Send your tool list in a `tools` array on `/v1/chat/completions`. Each entry has
  a `type`, and a `function` with a `name`, a `description`, and a JSON Schema in
  `parameters`.
- The server formats those tools the way the model was trained to receive them. It
  then parses the reply and returns `tool_calls` in the response message, with
  `finish_reason` set to `"tool"`.
- The server log tells you whether it used a **native** format for your model or
  the **generic** fallback. Native uses fewer tokens and works better. If the log
  says generic, change the model.
- Parallel tool calls are off by default. Turn them on with
  `"parallel_tool_calls": true` only when you need them.

**Step 6 — Constrain the output when you need a fixed shape.** Send a
`json_schema` or a `response_format` field on the chat endpoint, or a `grammar`
field on `/completion`. llama.cpp turns the schema into a GBNF grammar and allows
only tokens that fit it. The model then cannot produce malformed JSON. Note two
limits: the schema does **not** go into the prompt, so you must still describe the
shape in words; and only a subset of JSON Schema is supported.

**Study.**
- llama.cpp function calling guide (the `--jinja` flag, the `tools` array,
  `tool_calls`, native and generic formats, supported models):
  https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md
- llama.cpp GBNF grammars and JSON schema conversion (the supported subset and the
  performance notes):
  https://github.com/ggml-org/llama.cpp/blob/master/grammars/README.md

**Study.**
- llama.cpp server README (endpoints, flags, and fields):
  https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md
- llama.cpp main README (how to build `llama-server`):
  https://github.com/ggml-org/llama.cpp
- MDN "HTTP Messages" (the request and response structure) and "Content-Length":
  https://developer.mozilla.org/docs/Web/HTTP/Messages
- Rust book, Chapter 21.1 — the same read-and-parse pattern for a raw HTTP socket.

## Rust to WireGuard (Koil)

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

## Windows to Linux tools (WSL)

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

