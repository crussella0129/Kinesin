# Sprint 10 canonical verification record

The project's canonical runner is `.github/workflows/ci.yml`; no separate suite
runner script is present. This record preserves each suite confirmation for
formal TEST review. The report will attach to INT-0021 and INT-0022 Test evidence;
their realized transitions and the one dev-to-main PR are gated by Loop.

An independent final documentation scan checked 702 local link targets across
176 Markdown files with zero broken links. The installed Book validator passed
with all 27 intent chapters; the new verification record still awaited its
formal TEST commit at that checkpoint.

## Authoritative CI

- Tested head: `f3e3d0ff066eccbccf0d8e4e592031d9715c2b05`.
- Run: [34704378309](https://github.com/crussella0129/Kinesin/actions/runs/34704378309).
- Conclusion: success; all three jobs completed successfully on 2026-09-12.
- Windows: format, all-target/all-feature clippy with warnings denied, locked
  offline tests: 303 passed, zero failed, nine ignored.
- Ubuntu: same format/clippy/test commands: 315 passed, zero failed, nine ignored.
- Supply chain: `cargo deny check` passed advisories/bans/licenses/sources;
  `cargo audit` scanned 249 locked dependencies without vulnerabilities.

| Suite | Windows passed | Ubuntu passed | Ignored per OS |
| --- | ---: | ---: | ---: |
| Library units | 164 | 168 | 0 |
| adversarial_runtime | 3 | 3 | 0 |
| cli_inspect | 10 | 10 | 0 |
| command_tool | 12 | 12 | 0 |
| live_comparisons | 1 | 1 | 1 |
| live_evaluation | 2 | 2 | 5 |
| mcp | 20 | 20 | 1 |
| model_protocol | 17 | 17 | 0 |
| process_recovery | 3 | 3 | 1 |
| redteam | 2 | 2 | 0 |
| replay | 16 | 16 | 0 |
| runner_journal | 9 | 9 | 0 |
| runner_tools | 24 | 24 | 0 |
| sandbox_linux | 0 (platform gated) | 9 | 0 |
| service | 16 | 16 | 0 |
| service_load | 0 | 0 | 1 |
| settlement | 3 | 3 | 0 |
| windows_signal | 1 | 0 (platform gated) | 0 |
| **Total** | **303** | **315** | **9** |

No Linux enforcement test is skipped on the supported Ubuntu runner. The MCP
ignored worker executes in an isolated subprocess through its ordinary parent
test; the other ignored cases are opt-in historical live/load/chaos tests. Two
selected live-evaluation cases were separately executed against nighthawk and
remain separate from these offline totals.

## Local confirmations and post-CI delta

On the same runtime repair source, native Windows executed:

```text
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
```

All succeeded: 303 passed, zero failed, nine ignored. The all-target invocation
also built/executed the zero-test fixture/example targets. Local log:
`target/s10-final-windows.log`. CI log: `target/s10-ci-build.log`.

Commit `1a38395` adds diagnostic text to the existing CLI command assertion and
final documentation; it changes no runtime source or assertion condition. After
that edit, format, the exact CLI command test, and all-target/all-feature clippy
passed. The follow-on ledger commit is `334f179`. The CI record above is explicitly
for f3e3d0f; it is not relabeled as a run on a later commit. Final PR checks will
verify the submitted head again.

## Failed attempts and limits

- WSL with fixture binaries on `/mnt/c` passed 168 library and three adversarial
  tests, then failed one of ten CLI tests. A focused diagnostic exited 101 and
  recorded EACCES for the correct confined ELF while unconfined execution worked.
  The remaining mounted full suite did not run.
- A persistent native Linux-cache attempt exited 135 during compilation with
  SIGBUS/EIO after host disk exhaustion; zero tests ran and clippy was not reached.
  A later WSL startup probe failed. No WSL-wide restart or recovery was attempted.
  After space was freed, both distributions were observed stopped and a fresh
  `wsl -d Ubuntu -- /bin/true` exited zero. This confirms startup recovered, not
  that the interrupted native build or its tests passed.
- A local Windows rebuild failed with disk-full error 112. Removing the exact
  sprint-created duplicate Linux build directory freed about 5 GiB; its affected
  test/clippy rerun passed. Source, logs and the persistent native cache remain.

These failures are not accepted as local full Linux proof. Successful native
Ubuntu CI plus earlier focused real-kernel checks provide that platform evidence.
The assertion diagnostics and environment limitations are also recorded in the
[integration review](integration-review.md).

## Scoped manual confirmations

The two executed `--ignored --exact` live cases and disconnected CLI/replay
check are described in [remote-deployment.md](remote-deployment.md), including
runtime/model hashes, physical-host provenance, SSH/listener checks, actual
session request hashes and paired cache samples. Model and tunnel cleanup was
verified. Cache reuse in both extension modes establishes no causal flag benefit;
full-history continuity, concurrent slots and general deployment guarantees stay
proposed under INT-0026/0027.
