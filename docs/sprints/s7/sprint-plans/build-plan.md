# Sprint 7 Build Plan

## Intents
- [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) — state: planned; acceptance criteria covered: Linux Landlock+seccomp confinement (out-of-workspace read + network denied, in-workspace ok), mandatory refuse-if-unavailable, existing guarantees preserved, cfg(linux)-gated.

## Schema Tree
- Sprint Goal: mandatory Linux command sandbox
  - Dependencies
    - T-001: add landlock + seccompiler (Linux) and keep the gate green
  - Enforcement
    - T-002: apply Landlock + seccomp to the child via pre_exec
  - Verification
    - T-003: Linux enforcement tests

## Execution Sequence

### T-001: Add the Linux sandbox dependencies; keep the supply-chain gate green
- **Intent:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- **Touches:** Cargo.toml, Cargo.lock, deny.toml
- **Depends on:** (none)
- **Acceptance criterion:** the new deps are Linux-only and the INT-0013 gate stays green.
- **Success criterion (EARS):**
  - **WHEN** `landlock` and `seccompiler` are added under `[target.'cfg(target_os="linux")'.dependencies]`, **THEN** `cargo deny check` and `cargo audit` **SHALL** still pass (extend the `deny.toml` allow-list only if a new license appears).
  - **WHEN** the crate is built off Linux, **THEN** the new deps **SHALL NOT** be compiled.
- **Notes:** install a pinned Rust toolchain in WSL for local Linux build/test.

### T-002: Apply Landlock + seccomp to the child (`src/tools.rs`)
- **Intent:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- **Touches:** src/tools.rs
- **Depends on:** T-001
- **Acceptance criterion:** on a capable Linux kernel the child is confined (workspace-only filesystem, no network); when a protection is unavailable the runner refuses; other platforms unchanged.
- **Success criterion (EARS):**
  - **WHEN** a command runs on a Landlock+seccomp-capable Linux kernel, **THEN** it **SHALL** execute confined to the workspace root (Landlock: RW workspace, RX system prefixes, deny else) with network syscalls denied (seccomp EPERM on `socket`/`connect`/…).
  - **WHEN** Landlock or seccomp is unavailable, **THEN** `CommandRunner` **SHALL** return a defined `Error` (`sandbox_unavailable`) and **SHALL NOT** spawn an unconfined child.
  - **WHEN** the platform is not Linux, **THEN** behavior **SHALL** be unchanged (no sandbox path compiled).
- **Notes:** build the ruleset + compile the BPF **in the parent**; apply (`restrict_self` + seccomp install) in an `unsafe` `pre_exec` closure (tokio `Command::pre_exec`), apply-only/no-alloc; preserve tree-kill, output cap, timeout, env-scrub.

### T-003: Linux enforcement tests (`tests/`)
- **Intent:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- **Touches:** tests/ (new `tests/sandbox_linux.rs` or extend `tests/command_tool.rs`)
- **Depends on:** T-002
- **Acceptance criterion:** enforcement and the refuse path are proven on a real Linux kernel (WSL + ubuntu CI).
- **Success criterion (EARS):**
  - **WHEN** the `#[cfg(target_os="linux")]` sandbox tests run on a capable kernel, **THEN** an out-of-workspace read and a network-socket attempt **SHALL** be denied while an in-workspace command **SHALL** succeed, and the refuse path **SHALL** return the defined error when a protection is required but unavailable.
- **Notes:** drive via the `cmd-fixture` bin or `/bin/sh -c`; gate to Linux; runs green in WSL and the ubuntu CI `check` job.
