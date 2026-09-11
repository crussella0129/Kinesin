# INT-0012 — Command-execution sandboxing (OS-level isolation)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0012
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Confine `run_command` (INT-0003) with an OS-level sandbox, not just cap-std path
scoping, argv-only spawn, and environment scrubbing. On Linux, apply Landlock
(filesystem) plus a seccomp-bpf syscall filter; on Windows, an AppContainer/LPAC
profile plus the existing Job Object. Add a **mandatory-isolation mode** that
refuses to launch a command when a required protection is unavailable (kernel
ABI too old, capability missing) rather than silently degrading. The default
grant should deny network egress from a command unless explicitly allowed.
Non-goals: a general container runtime; sandboxing the harness itself; replacing
the capability model (this is defense-in-depth beneath it).

## Acceptance criteria
- On a supporting Linux kernel, a granted command runs under Landlock + seccomp:
  a denied filesystem path and a denied syscall/network attempt fail inside the
  sandbox, proven by tests on the actual platform.
- Mandatory-isolation mode refuses to launch with a defined error when a required
  protection is unavailable; a best-effort mode is a separate, explicit choice.
- On Windows, a granted command runs under an AppContainer/LPAC profile plus the
  Job Object; a denied file/network operation is refused, proven on-platform.
- The sandbox never weakens existing guarantees (tree-kill, output/timeout bounds,
  env scrubbing, no credential inheritance) and adds no path for a command to
  reach the private state tree.

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
Platform-specific code and CI coverage on both OSes; kernel-ABI variance means
capability detection and a refuse-on-missing path; a network-egress policy to
design; interacts with INT-0005 (MCP execution) and any future code tool.

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme A — security hardening); hardens [INT-0003] and gates future code/MCP execution.
