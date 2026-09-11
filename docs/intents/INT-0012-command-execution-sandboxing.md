# INT-0012 — Command-execution sandboxing (Linux: Landlock + seccomp)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0012
- **State:** active
- **Work evidence:** [T-002 build plan](../sprints/s7/sprint-plans/build-plan.md#t-002-apply-landlock--seccomp-to-the-child-srctoolsrs)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** [sprint 7 test report](../sprints/s7/sprint-tests/test-report.md)
- **Documentation evidence:** none

## Intent
Confine `run_command` (INT-0003) on **Linux** with an OS-level sandbox, not just
cap-std path scoping, argv-only spawn, and environment scrubbing: a Landlock
filesystem ruleset (workspace root read-write; standard system prefixes
read-execute; deny the rest) plus a seccomp-bpf filter denying network syscalls,
applied to the child in a `pre_exec` closure. It is **mandatory (secure by
default)**: if Landlock or seccomp is unavailable (kernel ABI too old, capability
missing), the command **refuses to launch** rather than running unconfined.
Windows AppContainer/LPAC sandboxing is a distinct OS-specific effort tracked
separately as **INT-0019**. Non-goals: a general container runtime; sandboxing
the harness itself; replacing the capability model (this is defense-in-depth
beneath it); a configurable best-effort/disabled mode (a later knob).

## Acceptance criteria
- On a supporting Linux kernel, a granted command runs under Landlock + seccomp:
  a read outside the workspace root and a network-socket attempt each fail inside
  the sandbox, proven by tests on the actual platform (WSL + the Ubuntu CI job).
- Isolation is mandatory: when Landlock or seccomp is unavailable, `CommandRunner`
  refuses with a defined error and never spawns an unconfined child.
- A normal command operating inside the workspace still succeeds under the sandbox.
- The sandbox never weakens existing guarantees (tree-kill, output/timeout bounds,
  env scrubbing, no credential inheritance), adds no path to the private state
  tree, and is `cfg(target_os="linux")`-gated so non-Linux builds are unchanged.

## Rationale
`run_command` is the project's largest trust surface; security.md already names
Landlock/seccomp/AppContainer/Job Objects as the required future work, and the
2026 SotA (Sandlock's Landlock+seccomp split enforcement; the microVM/gVisor
spectrum) confirms unprivileged kernel primitives as the pragmatic first tier.
This also gates any future arbitrary-code or MCP-hosted tool execution.

## Alternatives
Keep cap-std + argv + env-scrub only (current; insufficient for untrusted code).
gVisor or a Firecracker microVM (stronger hardware/kernel boundary, higher
overhead and deployment complexity) — reserve for an untrusted-code-execution
tier above allow-listed commands. WASM/WASI sandboxing (strong, but only for code
compiled to WASM, not arbitrary native tools).

## Consequences
Linux-specific `unsafe` code (`pre_exec`) and two Linux-only deps (landlock,
seccompiler) that must pass the INT-0013 supply-chain gate; kernel-ABI variance
means capability detection + a refuse-on-missing path; a mandatory network-egress
denial (a configurable mode is deferred); interacts with INT-0005 (MCP execution)
and any future code tool. Windows parity lives in INT-0019.

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme A — security hardening); hardens [INT-0003] and gates future code/MCP execution.
- 2026-09-11: re-scoped to the **Linux tier** (Landlock+seccomp) and selected for sprint 7; Windows AppContainer/LPAC split to the new follow-on **INT-0019**. Title changed to name the Linux mechanism. Mandatory secure-by-default (refuse-if-unavailable); a configurable best-effort mode is deferred.
- 2026-09-11: `proposed → planned`; linked to the sprint 7 build plan (T-001 Linux deps + gate, T-002 CommandRunner Landlock+seccomp via pre_exec, T-003 Linux enforcement tests).
