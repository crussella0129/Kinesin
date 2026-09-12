# Sprint 10 independent integration review

The review compared the repair set with the initial `cab59aa` checkout and the
locked intent/test contracts. Findings were repaired within T-011's explicit
integration regression clause; the original plan pair was not rewritten.

| Finding | Repair and executed evidence | Disposition |
| --- | --- | --- |
| Unix leader reaping could release a numeric PGID before group cleanup | `waitid(WNOWAIT)` observes exit while retaining the direct child; only group termination precedes Tokio reaping. Two Linux tests inspect the kernel-owned child status and cancelled/retried observation, alongside 12 command tests. | Fixed; Windows native error paths independently reviewed, without claiming forced API-failure injection. |
| Service verifier parent could overlap a Linux runtime read grant | Resolved verifier storage now participates in the same command runtime/private-path exclusion. A Linux negative config regression first accepted the unsafe placement, then rejected it after the fix; positive sibling/no-command cases remain valid. All 29 Linux config tests passed. | Fixed; synthetic paths only, no real verifier disclosed. |
| Many tiny MCP frames could amplify SDK response tasks despite byte caps | A 4,096-frame per-server-session ceiling is charged before SDK decoding, including partial/empty frames and cancelled reads. Four transport tests prove the ceiling; a 5,000-ping fixture with unread responses hits bounded deadline teardown. | Fixed; SDK sends may defer protocol completion until deadline/cleanup, so immediate read-fault propagation is not claimed. |
| x32 syscall numbering could bypass x86_64 deny lists | An initial mandatory BPF guard rejects foreign architectures, tagged x32 numbers and legacy 512–547. A production-filter interpreter verifies boundaries independently of host ABI support; all six real alternate-ABI probes require EPERM. | P1 fixed and independently re-reviewed; Linux sandbox units 3/3, sandbox integration 9/9 and command tests 12/12 passed. |
| Cache false omitted the extension and did not create an uncached baseline | The live test warms the server, alternates six pairs, checks actual request hashes/token reuse and reports omitted versus emitted flag semantics. | Corrected measurement: reuse in both modes, no causal flag-speedup claim. See [remote evidence](remote-deployment.md). |

The independent review found no further actionable cross-boundary regression in
startup schema persistence/replay ordering, the two grant gates, optional token
totals, continued input framing or encoded command bounds. This is a bounded
code review, not an assertion that all hostile behavior is confined.

Residuals remain explicit in the current [threat model](../../../threat-model.md):
Linux metadata/same-user process limitations, trusted MCP executable non-escape,
Windows AppContainer absence, source-replacement/lease semantics, comprehensive
descriptor/credential inventory, and the proposed continuity/deployment work.
Final whole-suite, CI and formal TEST-critic evidence belong in the test report.

The final documentation audit found 27 historical research links that still
used repository-root paths relative to their document directory. Only their
destinations were corrected. It also added the omitted explicit disposition for
INT-0008's historical research-survey criterion and replaced a stale roadmap
condition after nighthawk became available. All original criteria, 27 chapters,
20 OWASP risk rows and native/build ownership/cadence were independently checked;
proposed capabilities retain their limits.

A full WSL run with command fixtures built on `/mnt/c` failed the CLI command
observation: the correctly resolved ELF returned EACCES during confined spawn,
although it executed outside confinement. The diagnostic case exited 101 and
was not accepted as a passing Linux run. The test assertion now includes its
synthetic journal observation and CLI stderr. The final verification uses a
persistent Linux-native target directory and records explicit command status
files, while retaining all mandatory sandbox checks. That attempt failed during
compilation with SIGBUS/EIO (exit 135) after host storage exhaustion; no tests ran
and clippy was not reached. No runtime denial was relaxed to accommodate the
Windows-mounted build directory, and no WSL-wide restart or recovery was performed.
The successful native Ubuntu CI run supplies complete Linux verification.
After space was available and Ubuntu was observed stopped, a fresh `/bin/true`
invocation started successfully with exit zero; the interrupted build remains
failed evidence, without a host-wide WSL restart.

The host Windows assertion-only rebuild initially reported disk-full error 112.
Removing only the duplicate sprint-created `target/linux-s10` directory freed
approximately 5 GiB; the affected test and all-target/all-feature clippy then
passed. The full Windows suite had already passed locally and in CI. Diagnostic
logs remain locally ignored; none of these failed attempts are counted as passes.
