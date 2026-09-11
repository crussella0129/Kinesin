# Sprint 6 End-to-End Tests

- **Intent:** [INT-0013](../../../intents/INT-0013-supply-chain-security.md)
- **Tested head:** `62abee0e94eb819828f1986795a1faee0c0bca34`
- **Status:** possible (CI).

## `supply_chain_job_green_at_checkpoint`
The authoritative end-to-end proof that the gate executes and blocks is the new
`supply-chain` CI job running on the checkpoint PR: it installs cargo-deny +
cargo-audit and runs `cargo deny check` + `cargo audit` on GitHub's runner. A
green result proves the gate runs in CI on the committed tree; a policy violation
or a newly-published advisory against a pinned dependency would turn it red and
block the check. The existing windows+ubuntu `check` (fmt/clippy/test) jobs must
remain green.

This is the same non-unit CI verification sprint 1's cross-OS matrix used: the
workflow content is asserted offline (see integration-tests.md) and the job's
green status at the checkpoint is the live confirmation.

## Fault-injection note
"CI fails on a violation" is verified structurally (deny-by-default `licenses`/
`sources` + blocking, non-`continue-on-error` steps that exit non-zero on a
finding) rather than by injecting a real vulnerable/banned crate, which would
pollute the lockfile. This is the standard way a CI policy gate is verified.
