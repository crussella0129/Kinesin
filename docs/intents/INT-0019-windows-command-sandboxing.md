# INT-0019 — Windows command sandboxing (AppContainer/LPAC)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0019
- **State:** proposed
- **Work evidence:** [T-101 backlog](../work/tasks.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Bring `run_command` OS-level isolation to **Windows**, the platform parity for the
Linux Landlock+seccomp tier (INT-0012): run a granted command inside an
AppContainer/LPAC profile (filesystem + network capability restriction) alongside
the existing Job Object (process-group lifecycle/resource control). Mandatory,
secure-by-default like INT-0012: refuse to launch when the required isolation
cannot be established. Non-goals: a general container/VM runtime; replacing the
capability model; matching Linux syscall-filter granularity exactly (the OS
primitives differ).

## Acceptance criteria
- A granted command on Windows runs inside an AppContainer/LPAC profile plus the
  Job Object; a denied file operation (outside the workspace) and a denied
  network operation each fail inside the sandbox, proven on Windows (local + CI).
- Isolation is mandatory: when the AppContainer profile cannot be established,
  `CommandRunner` refuses with a defined error and never spawns an unconfined child.
- A normal in-workspace command still succeeds under the sandbox.
- Existing guarantees hold (tree-kill via the Job Object, output/timeout bounds,
  env scrubbing, no credential inheritance); `cfg(windows)`-gated so other builds
  are unchanged.

## Rationale
Kinesin's primary development host is Windows, so Windows is a first-class
deployment target, not an afterthought — the Linux-only sandbox (INT-0012) leaves
the largest trust surface unconfined on the platform the operator actually uses.
security.md already names AppContainer/LPAC + Job Objects as the Windows path.

## Alternatives
Job Object alone (current after INT-0003; lifecycle/resource control but no
file/network sandbox — insufficient). Restricted tokens / integrity levels
(coarser than AppContainer). A microVM/WSL-backed sandbox (heavier; a future
untrusted-code tier). Leaving Windows unconfined (rejected — it is the operator's
own platform).

## Consequences
Windows-specific code via `windows-sys` (AppContainer profile creation,
capability SIDs, `STARTUPINFOEX`/attribute lists) that likely cannot use the
plain `Command::group_spawn` path unchanged — a custom creation path is probable;
its own on-Windows test/CI story; the AppContainer file/network model does not map
one-to-one to Landlock/seccomp, so parity is behavioral, not mechanical.

## Transition history
- 2026-09-11: created as `proposed` (sprint 7; split from INT-0012, theme A — security hardening); Windows parity for the Linux sandbox.
