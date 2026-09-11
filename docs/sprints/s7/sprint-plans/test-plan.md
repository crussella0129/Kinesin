Finalized - DO NOT EDIT

# Sprint 7 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) | out-of-workspace read denied (Landlock) | T-002 / WHEN capable kernel THEN confined to workspace | `sandbox_denies_out_of_workspace_read` |
| [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) | network denied (seccomp) | T-002 / network syscalls denied | `sandbox_denies_network_socket` |
| [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) | in-workspace work still succeeds | T-003 / in-workspace command succeeds | `sandbox_allows_in_workspace_work` |
| [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) | mandatory: refuse when unavailable | T-002 / WHEN unavailable THEN defined error, no unconfined spawn | `sandbox_refuses_when_unavailable` |
| [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) | gate green with new deps; non-Linux unchanged | T-001 / deps cfg(linux); deny+audit pass | `supply_chain_green_with_sandbox_deps`, `non_linux_build_unchanged` |

## Unit Tests
Sandbox behavior is a process/kernel effect, so it is exercised through
integration tests (below), not pure unit tests. The parent-side helpers (ruleset
build, ABI query, BPF compile) may carry small `#[cfg(target_os="linux")]` unit
assertions where they are pure (e.g. the availability check returns the refuse
signal on a forced-unavailable input).

## Integration Tests
### Linux sandbox enforcement (`tests/sandbox_linux.rs`, `#[cfg(target_os = "linux")]`)
- **Intents:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- `sandbox_denies_out_of_workspace_read`: a command reading a path outside the workspace root fails (Landlock).
- `sandbox_denies_network_socket`: a command opening a network socket fails (seccomp EPERM).
- `sandbox_allows_in_workspace_work`: a command reading/writing inside the workspace succeeds under the sandbox.
- `sandbox_refuses_when_unavailable`: with the sandbox forced unavailable, `CommandRunner` returns the defined `sandbox_unavailable` error and spawns nothing.

### Supply-chain gate & portability
- `supply_chain_green_with_sandbox_deps`: `cargo deny check` + `cargo audit` pass with landlock/seccompiler added.
- `non_linux_build_unchanged`: the existing windows+macos build/test path compiles without the sandbox deps (the deps are `cfg(target_os="linux")`); verified by the unchanged windows CI job.

## End-to-End Tests
- **Status:** possible (Linux). The ubuntu CI `check` job runs the
  `#[cfg(target_os="linux")]` sandbox tests on a Landlock-capable kernel — the
  authoritative enforcement proof; WSL provides the same locally. The windows +
  supply-chain jobs remaining green is the portability proof.
