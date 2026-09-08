# Learning resources index

## Rust core

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

## Non-Rust dependencies

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

## Formats and background

- **HTTP messages (MDN):** https://developer.mozilla.org/docs/Web/HTTP/Messages
- **JSON grammar:** https://www.json.org/
- **TOML specification:** https://toml.io/en/v1.0.0
- **CommonMark (Markdown):** https://spec.commonmark.org/
- **ReAct paper (the loop idea):** https://arxiv.org/abs/2210.03629

## Tool calling

- **llama.cpp function calling guide** — the `--jinja` flag, the `tools` array,
  `tool_calls`, native versus generic formats, supported models, and the KV
  quantization warning. Read this first:
  https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md
- **llama.cpp GBNF grammars** — constrained decoding, JSON schema conversion, the
  supported subset, and the performance notes:
  https://github.com/ggml-org/llama.cpp/blob/master/grammars/README.md
- **Tool calling with local models, a practical evaluation** (Docker) — 21 models
  over 3,570 cases; the 8B floor, the Qwen results, and the four failure modes:
  https://www.docker.com/blog/local-llm-tool-calling-a-practical-evaluation/
- **CodeAct, "Executable Code Actions Elicit Better LLM Agents"** — the case for
  code as the action format, and the numbers behind it ([loop-and-tools.md](loop-and-tools.md), approaches not chosen):
  https://arxiv.org/abs/2402.01030
- **Model Context Protocol** — the common tool interface standard; shape your tool
  definition to match it ([loop-and-tools.md](loop-and-tools.md), how to define a tool): https://modelcontextprotocol.io/
- **Tool schema design: inputs, outputs, and error handling:**
  https://aiquinta.ai/blog/llm-tool-schema-design-inputs-outputs-error-handling/
