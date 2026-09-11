# Sprint 7 Research Report — Command-execution sandboxing (INT-0012)

## Intents Reviewed
- [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) — selected; roadmap theme A, sequence #2 after INT-0013 (done). To be **re-scoped in the plan phase to the Linux tier** (Landlock+seccomp); the Windows AppContainer tier splits to a new follow-on intent (**INT-0019**), because the two are distinct OS-specific efforts and Windows is large on its own.
- INT-0019 (to be created in plan) — Windows command sandboxing (AppContainer/LPAC + the existing Job Object).

## 1. Sprint Goal
Confine `run_command` with OS-level isolation beyond cap-std + argv + env-scrub.
This sprint delivers the **Linux tier**: a Landlock filesystem ruleset and a
seccomp syscall filter applied to the child in a `pre_exec` closure, plus a
**mandatory-isolation mode** that refuses to launch when a required protection is
unavailable (rather than running unconfined). Windows AppContainer is split to
INT-0019. WSL2 (kernel 6.6) gives local Linux iteration; the ubuntu CI job is the
authoritative Landlock/seccomp enforcement check.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| src/tools.rs:1013 `CommandRunner::execute` | high | The spawn path: builds `tokio::process::Command`, `env_clear` + PATH, `group_spawn()` (command-group). The sandbox hooks in here: build ruleset/filter in the parent, apply in a `pre_exec` closure before exec. Preserve tree-kill/output/timeout guarantees. |
| src/tools.rs:994 `CommandRunner` struct | high | Holds `root` + `allowed`; add the isolation policy (mandatory vs. best-effort) and the workspace root the Landlock ruleset grants. |
| Cargo.toml (target.cfg(unix)/linux deps) | high | Add `landlock` + `seccompiler` under `[target.'cfg(target_os="linux")'.dependencies]`; both crates are Linux-only. |
| deny.toml | high | New deps must pass the INT-0013 gate — re-run `cargo deny check` and extend the license allow-list if they introduce a new license. |
| .github/workflows/ci.yml | medium | The ubuntu `check` job runs the Landlock/seccomp enforcement tests (Ubuntu runner supports Landlock). |
| docs/security.md §"Future external tools and hostile code" | medium | Already specifies the requirement: Landlock ABI varies, "a mandatory isolation mode must refuse launch when a required protection is unavailable," port restrictions alone don't select a host. |

## 3. External Sources
- [rust-landlock crate](https://landlock.io/rust-landlock/landlock/) — `Ruleset` builder + `restrict_self()` applies the ruleset to the calling thread and every child it spawns; ABI is queryable so the code can refuse when the kernel is too old. Applied inside `pre_exec`, it confines only the child.
- [seccompiler](https://docs.rs/seccompiler) — compiles a seccomp-BPF filter (expressed in Rust) and installs it; used in `pre_exec` to deny network/dangerous syscalls.
- [Running Rust binaries without root using sandboxing (2026)](https://oneuptime.com/blog/post/2026-01-07-rust-sandboxing-seccomp-landlock/view) — current pattern (`seccompiler = 0.4`, `landlock = 0.3/0.4`): seccomp syscall filtering + Landlock filesystem restriction + capability dropping.
- Sprint 5 survey (sandboxing spectrum) — Landlock+seccomp is the unprivileged first tier chosen here; microVM/gVisor/WASM remain heavier alternatives for a future untrusted-code tier.

## 4. Risks, Unknowns, Dependencies
- **Risk (`pre_exec` safety):** the closure runs post-fork/pre-exec and must be async-signal-safe. Mitigation: build the Landlock ruleset and compile the seccomp BPF **in the parent**; in `pre_exec` only call the apply syscalls (`restrict_self`, seccomp install) — no allocation.
- **Risk (Landlock ruleset correctness):** too tight breaks legitimate commands (dynamic linker needs `/usr`,`/lib`,`/etc`,`/proc` reads; the workspace needs RW). Mitigation: grant read-execute on standard system prefixes + read-write on the workspace root, deny the rest; iterate in WSL + CI against a real command (e.g. the `cmd-fixture` bin).
- **Risk (seccomp allowlist brittleness):** a strict syscall allowlist can kill normal programs. Mitigation: start with a **targeted denylist** of network syscalls (`socket`,`connect`,`bind`,…) returning EPERM — enough to enforce "no network egress" without enumerating every benign syscall; broaden later.
- **Unknown (WSL Landlock):** WSL2 6.6 may or may not enable the Landlock LSM (`/sys/kernel/security/lsm` was empty). If WSL lacks it, seccomp + the refuse-on-unavailable path are tested in WSL and Landlock enforcement is verified in the ubuntu CI runner (which supports it). Not a blocker.
- **Dependency:** adding `landlock`/`seccompiler` must keep the INT-0013 supply-chain gate green (`cargo deny check`).
- **Deferred:** Windows AppContainer/LPAC → INT-0019 (created in plan); microVM/WASM tier stays a future option.

## 5. Recommended Approach
**Primary — Linux Landlock+seccomp in `CommandRunner`, applied via `pre_exec`, with mandatory-isolation refusal.**
1. Add `landlock` + `seccompiler` as `cfg(target_os="linux")` deps; re-green `deny.toml`.
2. Extend `CommandRunner` with an isolation policy; on Linux, query the Landlock ABI, build a ruleset (RX on system prefixes, RW on the workspace root, deny else) and compile a seccomp filter (deny network syscalls). If a required protection is unavailable and isolation is mandatory → refuse to spawn with a defined `Error` outcome; never silently run unconfined.
3. In a `pre_exec` closure, apply `restrict_self()` + install the seccomp filter, then exec. Keep the existing tree-kill/output/timeout/env-scrub guarantees intact.
4. Tests (`#[cfg(target_os="linux")]`, run in WSL + ubuntu CI): a sandboxed command cannot read a file outside the workspace root (Landlock) and cannot open a network socket (seccomp); a normal in-workspace command still succeeds; the refuse path returns a defined error when isolation is required but unavailable.

**Alternative considered:** a full seccomp allowlist (rejected as brittle for arbitrary allow-listed commands — start with a network denylist); a microVM/gVisor tier (heavier, deferred). Windows in the same sprint (rejected — distinct large effort → INT-0019).

## Artifacts
- No code pre-authored; the CommandRunner sandbox, the two Linux deps, and the Linux enforcement tests are the build-phase deliverables.
