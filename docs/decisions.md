# What is missing: decisions to make first

These are the open points. Decide each one before you write the matching code.
Each item says the problem, the choices, and a suggested start.

1. **Workspace or one binary.** The first draft mixed a root `src/main.rs` with
   three sub-crates. That is unclear. Choose a Cargo workspace with three member
   crates (the project layout in the [README](../README.md)), **or** one binary crate with three modules. Suggested
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

6. **WireGuard scope (settled).** The tunnel stays. The reason is in [components.md](components.md), Koil:
   Kineserve is to run on a bigger remote machine, and the Koil-to-Koil link keeps
   that private. The local setup is the same design with one endpoint. Two points
   remain: write the tunnel with a library or the system tools, never by hand
   ([integration.md](integration.md), WireGuard); and build the transport seam in Phase 1, so Phase 5 adds the
   tunnel without a change to K-Core.

7. **Trace schema (drafted).** [traces.md](traces.md) now gives a first schema and examples.
   The open choices that remain: confirm the field names; decide whether to keep
   the full `llama-server` payload or only selected fields; and decide the ID
   scheme with items 4 and 5.

8. **The ReAct loop rules (drafted).** [loop-and-tools.md](loop-and-tools.md) now gives the step cycle, the
   states, the action format, and the stop conditions. The open choices that
   remain: set the maximum step count in `kinesin.toml`; pick the action format
   (the `/completion` JSON object or the `/v1/chat/completions` tool calls); and
   set the rule for a stuck loop.

9. **The config and instruction split (drafted).** [configuration.md](configuration.md) now gives the
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

