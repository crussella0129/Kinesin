# Test Critique — Sprint 7

Adversarial read-only screen of the sprint 7 sandbox-enforcement evidence against
INT-0012 (Linux tier).

## Concerns

### C-001: the mandatory refuse path isn't fault-injected on a capable kernel
- **Where:** `e2e-tests.md` / `sandbox_is_mandatory`.
- **Failure mode:** negative-path.
- **Why it matters:** on WSL/CI (Landlock-capable), the `sandbox_unavailable` refuse branch never fires; the test asserts the enforce-or-refuse dichotomy, not a forced refusal.
- **Suggested response:** accept-with-rationale. The refuse branch is the live behavior on any kernel lacking Landlock (where `sandbox_is_mandatory` asserts it), and forcing it on a capable kernel would require an env-readable "disable sandbox" switch in the security path — a deliberate non-goal (an attacker-influencable disable is worse than an untested-here branch). The branch is small and its inputs (`build_ruleset`/`build_bpf` `Err`, `RulesetStatus::NotEnforced`) map directly to the defined error.

### C-002: seccomp is a network denylist, not a full syscall allowlist
- **Where:** `src/tools.rs` `build_bpf`.
- **Failure mode:** weak-assertion.
- **Why it matters:** the filter confines network egress, not general syscall misuse.
- **Suggested response:** accept-with-rationale (deliberate MVP scope, carried from the plan critique). The intent's guarantee is filesystem confinement (Landlock — proven) + no network (seccomp denylist — proven); a strict syscall allowlist is brittle across arbitrary allow-listed commands and is a future hardening.

### C-003: enforcement tests skip on a non-capable kernel
- **Where:** `integration-tests.md` (each enforcement test).
- **Failure mode:** e2e-drift (screened).
- **Why it matters:** a vacuous pass on a kernel without Landlock could mask a broken sandbox.
- **Suggested response:** defer-with-rationale. The ubuntu CI runner and WSL 6.6 both enforce Landlock, so the enforcement branch actually ran (the artifacts confirm no "skip" was printed); the skip exists only so the mandatory-refuse kernels don't spuriously fail. CI is the authoritative capable kernel.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — deny-read, deny-network, allow-in-workspace, and mandatory each map to a named executed test; portability maps to the windows job + full-suite runs.
- **Assertion weakness (enforcement):** none — the deny/allow tests assert real kernel outcomes (`READ_DENIED`/exit 21, `SOCKET_DENIED`/exit 22, `READ_OK`) via a real spawned process, not a mock.
- **Stub leakage / integration drift:** none — real `cmd-fixture`, real Landlock/seccomp, real `CommandRunner` path.
- **Flake risk:** none — kernel enforcement is deterministic; the socket probe is a local `socket()` denial, no network dependency.
- **Evidence drift:** none — artifacts name tested head `94e3855`, carry the WSL confirmations, and identify the CI jobs + the Test-evidence link on INT-0012.

## Confidence
proceed-with-caveats
