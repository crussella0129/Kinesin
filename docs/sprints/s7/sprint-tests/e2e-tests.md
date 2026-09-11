# Sprint 7 End-to-End Tests

- **Intent:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- **Tested head:** `94e3855466fa71d34e6136b1a7a562f6d34c5bb6`
- **Status:** possible (Linux CI). 

## Authoritative enforcement in CI
The `#[cfg(target_os="linux")]` `sandbox_linux` tests run in the ubuntu `check`
job on a Landlock-capable kernel — the authoritative proof that Landlock denies
out-of-workspace reads and seccomp denies network syscalls on a real Linux, and
that a legitimate in-workspace command still runs. WSL2 (kernel 6.6) provides the
same enforcement locally and was used to iterate.

The `supply-chain` job (INT-0013) additionally re-checks the tree with the new
`landlock`/`seccompiler` dependencies: `cargo deny check` + `cargo audit` green.
The windows `check` job proves the sandbox is cleanly `cfg`-compiled-out (behavior
unchanged off Linux).

## Fault-injection note
The mandatory refuse path (`sandbox_unavailable` when Landlock/seccomp cannot be
established) is asserted by `sandbox_is_mandatory` on any kernel and is the live
behavior on a kernel lacking Landlock; on the capable WSL/CI kernels the
complementary enforced path is exercised. It is not fault-injected on a capable
kernel (no safe hook to force unavailability without an env-readable disable
switch in the security path, which is deliberately avoided).
