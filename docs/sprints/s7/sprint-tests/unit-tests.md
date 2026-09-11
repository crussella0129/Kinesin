# Sprint 7 Unit Tests

- **Intent:** [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md)
- **Tested head:** `94e3855466fa71d34e6136b1a7a562f6d34c5bb6`

Not applicable as a separate layer. The sandbox is a process/kernel effect
(Landlock filesystem confinement + a seccomp syscall filter applied to a spawned
child), so it is verified through integration tests that spawn a real command
under `CommandRunner` (see [integration-tests.md](integration-tests.md)) plus the
ubuntu CI job. The parent-side builders (`build_ruleset`, `build_bpf`,
`resolve_binary`) compile under `cargo clippy --all-targets -D warnings` on Linux.
