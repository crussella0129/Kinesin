# Sprint 7 Integration Tests (Linux sandbox enforcement)

- **Intent:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- **Tested head:** `94e3855466fa71d34e6136b1a7a562f6d34c5bb6`
- **Runners:** WSL2 kernel 6.6 (Landlock+seccomp enforced) locally; the ubuntu CI `check` job authoritatively.
- **Result:** all green — the enforcement branch ran for real (no "skip" printed).

## `tests/sandbox_linux.rs` (`#![cfg(target_os = "linux")]`)
| Test | EARS clause | Result (WSL) |
|------|-------------|--------------|
| `sandbox_denies_out_of_workspace_read` | Landlock denies a read outside the workspace root | ok — `READ_DENIED`, exit 21 |
| `sandbox_denies_network_socket` | seccomp denies network syscalls | ok — `SOCKET_DENIED`, exit 22 |
| `sandbox_allows_in_workspace_work` | in-workspace work still succeeds | ok — `READ_OK`, exit 0 |
| `sandbox_is_mandatory` | a command is enforced-Ok or refused, never unconfined | ok |

## Portability & no-regression
- `tests/command_tool.rs` (8 tests) pass **under the sandbox** in WSL — the fixture
  in `target/` is exec'd via the resolved-binary grant, workspace RW works, and no
  network is needed; the mandatory sandbox does not break legitimate commands.
- Full suite green on **WSL** (Linux, sandbox enforced) and **Windows** (sandbox
  `cfg`-compiled-out; 145 lib + all integration suites pass).
- `cargo clippy --all-targets -D warnings` clean on Linux and Windows; `cargo fmt --check` clean.

## Confirmation
```
# WSL (Landlock+seccomp enforced):
test sandbox_denies_out_of_workspace_read ... ok
test sandbox_denies_network_socket ... ok
test sandbox_allows_in_workspace_work ... ok
test sandbox_is_mandatory ... ok
test result: ok. 4 passed; 0 failed
# command_tool under sandbox: ok. 8 passed; 0 failed
```
