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
