# Sprint 13 unit and static verification

Verified on native Windows on 2026-09-20, after the approximately 04:52 UTC [live confidence gate](e2e-tests.md#live-confidence-gate-passed). The implementation operator reported the final commands and results to the independent reviewer before this record was written at 05:00 UTC. No unit checks were substituted for the prior browser exercise.

The tested working tree was based on `6bb16fb17dec84e7d269d88f666a4af9ee4ebc76`; [tested-source.json](tested-source.json) records SHA-256 identities of the final changed Rust sources and affected test files. All eleven hashes match the committed blobs in T-113 implementation commit `d9547f8865b0703919f3ba3aea1f75abd7432932`. No remote CI result is implied.

| Command / component | Result |
| --- | --- |
| `cargo test --locked --lib --test runner_tools --test runner_journal --test managed_cli --test replay` — library component | 206 passed. Integration components are recorded separately. |
| `cargo fmt --all`, followed by `cargo fmt --all -- --check` | Passed. |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed on final source/test changes. The final replay-file adjustment was formatting only. |

For INT-0031 AC3/AC4 and T-113's negative EARS clauses, the new `preview_grants_reject_service_configuration_and_checked_authority` case in the preview suite directly asserts rejection of service-mode preview grants and checked authority. The other preview cases exercise real HTTP, file and lifecycle boundaries; they are mapped in [integration-tests.md](integration-tests.md).

An initial MSVC/PDB link attempt failed because free disk space reached zero. The operator removed verified generated PDB/incremental build artifacts, reclaimed about 14 GB and retried successfully. No product change or persistent compiler option was introduced to hide that environmental failure.

Coverage is native Windows only. Linux runs, remote CI, a full unrelated suite and statistical model-quality evaluation were not performed in this sprint.
