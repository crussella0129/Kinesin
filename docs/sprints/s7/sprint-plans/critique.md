# Plan Critique — Sprint 7

Adversarial read-only screen of `build-plan.md` and `test-plan.md` against the
research report and INT-0012 (re-scoped to the Linux tier).

## Concerns

### C-001: seccomp uses a network denylist, not a syscall allowlist
- **Where:** `build-plan.md` T-002.
- **Quote:** "seccomp-bpf filter denying network syscalls (`socket`,`connect`,…)".
- **Failure mode:** weak-assertion.
- **Why it matters:** a denylist confines network egress but is not a general syscall-confinement (a program could still do other dangerous local syscalls the filter doesn't list).
- **Suggested response:** accept-with-rationale (deliberate MVP scope). The intent's network-egress guarantee is exactly what the denylist enforces, and a strict allowlist is brittle across arbitrary allow-listed commands (it kills benign programs). Filesystem confinement is Landlock's job (covered). A tighter syscall allowlist is a future hardening, noted; the sprint proves the network-denial and fs-confinement guarantees it claims.

### C-002: local (WSL) Landlock enforcement may be unavailable
- **Where:** research report §4 / `test-plan.md` E2E.
- **Failure mode:** flake-risk / e2e (screened).
- **Why it matters:** `/sys/kernel/security/lsm` was empty in WSL, so the Landlock-enforcement tests might only truly enforce in the ubuntu CI runner.
- **Suggested response:** defer-with-rationale. The tests gate on capability: on a Landlock-capable kernel they assert enforcement, and the ubuntu CI runner is Landlock-capable and authoritative. WSL still covers build, seccomp, the in-workspace path, and the refuse branch. The design's mandatory refuse-if-unavailable means a kernel without Landlock never runs unconfined — so a WSL lacking Landlock exercises exactly the refuse path, which is itself a tested clause.

### C-003: `pre_exec` runs unsafe code post-fork
- **Where:** T-002 `pre_exec` closure.
- **Failure mode:** hidden-dep (screened).
- **Why it matters:** a post-fork/pre-exec closure must be async-signal-safe; allocation or a panic there is UB.
- **Suggested response:** accept-with-rationale — the plan mandates building the ruleset and compiling the BPF **in the parent** and doing apply-only syscalls in the closure (no allocation), which is the standard-safe pattern for `restrict_self` + seccomp install.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — T-001/T-002/T-003 each carry measurable `WHEN…THEN…SHALL` clauses.
- **Plan-test mismatch:** none — every clause maps to a named test (`sandbox_denies_out_of_workspace_read`, `sandbox_denies_network_socket`, `sandbox_allows_in_workspace_work`, `sandbox_refuses_when_unavailable`, `supply_chain_green_with_sandbox_deps`, `non_linux_build_unchanged`), and each test traces to a clause.
- **Missing risk coverage:** none — pre_exec safety, ruleset correctness, seccomp brittleness, WSL-Landlock uncertainty, and gate-green-with-new-deps all land on a task, test, or explicit acceptance.
- **Intent drift:** none — INT-0012 re-scoped to Linux with the change in transition history; Windows split to INT-0019 (a tracked intent, roadmap + SUMMARY); acceptance matches the sprint.
- **Granularity:** none — deps / implementation / tests are distinct.
- **E2E status drift:** none — `possible` via the ubuntu CI job; not skipped.

## Confidence
proceed-with-caveats
