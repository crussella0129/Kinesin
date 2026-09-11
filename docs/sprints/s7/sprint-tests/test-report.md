# Sprint 7 Test Report — Command sandboxing, Linux tier (INT-0012)

- **Tested head:** `94e3855466fa71d34e6136b1a7a562f6d34c5bb6`
- **Toolchain:** pinned 1.96.0. Verified on **WSL2 kernel 6.6** (Landlock+seccomp enforced) and **Windows** (sandbox cfg-compiled-out), plus the ubuntu CI `check` job (authoritative Landlock).
- **Result:** all green. Enforcement ran for real in WSL (no skip); full suite green on both platforms; clippy + fmt clean on both.
- **Critic verdict:** `proceed-with-caveats` (see [critique.md](critique.md)).

## Intent acceptance → evidence
| INT-0012 acceptance criterion | Verification | Result |
|-------------------------------|--------------|--------|
| out-of-workspace read denied (Landlock) | `sandbox_denies_out_of_workspace_read` | ok (WSL real; CI authoritative) |
| network denied (seccomp) | `sandbox_denies_network_socket` | ok |
| mandatory: refuse when unavailable, never unconfined | `sandbox_is_mandatory` + the `sandbox_unavailable` refuse path | ok |
| in-workspace command still succeeds | `sandbox_allows_in_workspace_work` + `command_tool` (8) under the sandbox | ok |
| existing guarantees preserved; cfg(linux)-gated, non-Linux unchanged | full suite green WSL + Windows; windows CI job | ok |
| supply-chain gate green with new deps | `cargo deny check` + `cargo audit` (the `supply-chain` CI job) | ok |

Details: [integration-tests.md](integration-tests.md), [e2e-tests.md](e2e-tests.md), [unit-tests.md](unit-tests.md).

## Caveats carried forward (from the critic)
- **C-001 (accepted):** the mandatory-refuse branch is not fault-injected on a Landlock-capable kernel (no safe force-disable in the security path); it is the live behavior where Landlock is absent and is dichotomy-asserted otherwise.
- **C-002 (accepted):** seccomp is a network denylist, not a full syscall allowlist — the proven guarantees are filesystem confinement (Landlock) + no network; a stricter allowlist is a future hardening.
- **C-003 (deferred):** enforcement tests skip on a non-capable kernel; WSL 6.6 and the ubuntu runner both enforce, so the enforcement branch genuinely ran.

## Verdict
Test phase satisfied for INT-0012 (Linux tier): Landlock filesystem confinement and seccomp network denial are proven on a real Linux kernel, the sandbox is mandatory (refuse-if-unavailable), legitimate commands still run, and the change is cleanly cfg-gated. Windows parity remains INT-0019. Proceed to loop.
