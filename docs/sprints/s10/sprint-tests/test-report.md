# Sprint 10 Test Report

## Intent Verification

| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
| --- | --- | --- | --- | --- |
| [INT-0021](../../../intents/INT-0021-harness-contract-review.md) | Intent-only baseline precedes code; all original criteria assessed | T-001/A; intent-first review, four criterion-level audit chapters, explicit historical INT-0008 survey disposition | Pass; review chronology and original evidence retained | Test evidence links this report |
| INT-0021 | Foundational authority, acceptance, owner/admission/cancellation/settlement/replay contracts have named code/test evidence | T-001/A, T-011/A; core/assurance audits plus canonical core, scheduler, service, settlement, storage and replay regressions | Pass within documented boundaries | Same |
| INT-0021 | Missing outcome ownership and proposed ambiguities | T-001/A; INT-0023..0027 and amendments/supersession history | Pass; proposals are not reported implemented | Same |
| INT-0021 | Roadmap, complete risk mapping, native inventory, owner/cadence | T-009/B; assurance review of all 20 LLM/ASI rows and supply-chain inventory | Pass | Same |
| INT-0021 | Book, links and independent critique | T-011/B; installed Book validator, 702-target link scan, accepted plan and final TEST critiques | Pass for TEST boundary; Loop owns the final checkpoint | Same; realization waits for Loop |
| [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md) | Honest independent usage dimensions and complete totals | T-002/A–B; partial-usage wire tests, mixed-usage runner tests, inspect dimension tests | Pass, including missing/error/zero/overflow cases | Test evidence links this report |
| INT-0022 | Actual continuation framing is bounded and cache checks use real sessions | T-003/B,D; `continuation_initial_context_is_bounded`, `actual_session_cache_request_contract`, actual-session manual measurement | Pass; reuse observed, no causal flag-speedup claim | Same |
| INT-0022 | Freeform compaction protects checked evidence and current replay semantics | T-003/A,C; `freeform_reads_compact_and_replay`, checked evidence survival, compatibility and continuation-origin replay tests | Pass; unsupported old semantics refuse explicitly | Same |
| INT-0022 | Commands bound encoded output and own their process group/job | T-004/A–B; all 12 command tests plus Unix nonreaping-observation units and command journaling | Pass, including pre-cancel, timeout, leader exit and drop | Same |
| INT-0022 | Mandatory Linux enforcement and denied outside/private/network/escape effects | T-005/A–B; nine real kernel sandbox cases, forced-refusal/filter units, private config/verifier regressions | Pass in native Ubuntu CI; incomplete WSL attempts remain failed evidence | Same |
| INT-0022 | MCP framing/discovery, environment and awaited process lifecycle are bounded | T-006/A–B; transport units and protocol/count/page/schema/environment/teardown fixture cases | Pass; tiny-frame SDK sends can settle through bounded deadline cleanup | Same |
| INT-0022 | MCP preparation follows admission and freezes definitions before model dispatch or settles without effects | T-007/A,C; controller/service spawn markers, startup cancellation/deadline, frozen capture tampering and journal-error recovery | Pass | Same |
| INT-0022 | Frozen definitions and run grants independently gate live/replay dispatch | T-007/B; unapproved-call, preprepared-authority, altered/ungranted schema and effect-free replay tests | Pass | Same |
| INT-0022 | Confidential non-loopback origins with unchanged redirect/proxy rules | T-008/A; `remote_plaintext_is_rejected`, loopback/address-family matrix, all 17 model protocol tests | Pass; actual SSH-forwarded two-host checked serving also passed | Same |
| INT-0022 | Blocking reviewed dependency policy and adversarial/platform gates | T-009/A, T-011/A; deny/audit, exact-exception negative policy, canonical Windows/Ubuntu CI | Pass; isolated duplicate-policy removal exited 2 | Same |
| INT-0022 | Atomic no-replace move destination publication | T-010/A; four publication/collision/competition/partial-cleanup units | Pass; stale-source fencing and two-name atomicity are not claimed | Same |
| INT-0021/0022 | Reviewed integrated evidence and one dev-to-main checkpoint | T-011/B; result chapters, final clean TEST critic, tracked Book/phase checks | TEST evidence complete; final PR/head and realization gates execute in Loop | Completion evidence is the subsequent sprint metadata |

Every locked EARS clause has its named executed-case mapping in
[unit-tests.md](unit-tests.md). Cross-layer assertions are recorded in
[integration-tests.md](integration-tests.md) and
[e2e-tests.md](e2e-tests.md). The independent reviewer inspected actual tests,
the retained CI log and manual result JSON, then rechecked final evidence changes
before issuing the [clean critique](critique.md).

## Summary

- Unit tests: Windows 164 passed / 0 failed / 164 executed;
  Ubuntu 168 passed / 0 failed / 168 executed.
- Integration tests: Windows 139 passed / 0 failed / 139 executed;
  Ubuntu 147 passed / 0 failed / 147 executed.
- Total ordinary suite: Windows 303 passed; Ubuntu 315 passed; nine ignored per
  platform. The isolated MCP worker is invoked by its passing parent test.
- E2E: CLI/service/MCP flows pass within the above totals; two additional selected
  manual live cases passed, plus the disconnected checked CLI/export/replay probe.
- CI status: green at the tested repair head. Later diagnostic/documentation
  commits are identified separately and the submitted PR head is checked in Loop.

## CI Confirmation

- **Head SHA:** `f3e3d0ff066eccbccf0d8e4e592031d9715c2b05`.
- **CI run:** [34704378309](https://github.com/crussella0129/Kinesin/actions/runs/34704378309).
- **Conclusion:** success.
- **Confirmations:** canonical `.github/workflows/ci.yml` jobs
  `check (windows-latest)`, `check (ubuntu-latest)` and `supply-chain` all passed.
  Both platforms ran format, locked all-target/all-feature clippy with warnings
  denied and locked offline tests. Deny passed all four gates; audit found no
  vulnerabilities among 249 dependencies.

[verification-record.md](verification-record.md) retains per-suite counts,
commands, log locations and the exact post-CI delta. The additional local Windows
`cargo test --locked --all-targets` passed all 303 cases. Commit 1a38395 changes
only diagnostics on an existing test assertion plus documents; its affected
Windows test, format and complete clippy passed. Commit 334f179 records its ledger
evidence. Neither is falsely labeled as the CI-tested head above.

## Failures

All identified runtime contract regressions were repaired and retested. The
[integration review](integration-review.md) records the extra Unix PID, verifier
placement, MCP tiny-frame and x32 findings and their dispositions. Earlier green
tests are not a substitute for the new adversarial cases; actual red-to-green
observations are distinguished from source-derived findings.

Local failed attempts are retained: a confined WSL fixture on `/mnt/c` received
EACCES; a native-cache full build later failed during storage exhaustion with
SIGBUS/EIO before tests, and a Windows rebuild hit disk-full error 112. Removing
the sprint-created duplicate target freed space; the affected Windows rerun
passed and Ubuntu startup subsequently recovered. No incomplete local Linux run
is counted as a pass. Native Ubuntu CI supplies full platform verification.

## Technical Debt Identified

- [INT-0019](../../../intents/INT-0019-windows-command-sandboxing.md): Windows
  AppContainer isolation; Jobs currently own lifetime only.
- [INT-0010](../../../intents/INT-0010-cross-agent-write-coordination.md): stale
  edit fencing and shared leases; destination no-replace is the repaired subset.
- [INT-0023](../../../intents/INT-0023-lifecycle-data-stewardship.md) and
  [INT-0025](../../../intents/INT-0025-secrets-and-egress.md): restore/migration
  drills and comprehensive credential/descriptor/egress stewardship.
- [INT-0024](../../../intents/INT-0024-harness-evaluation.md),
  [INT-0026](../../../intents/INT-0026-session-context-continuity.md) and
  [INT-0027](../../../intents/INT-0027-encrypted-remote-deployment.md): broader
  coding evaluation, full session continuity/token admission, concurrent slots
  and deployment coverage beyond the measured configuration.
- Other proposed capabilities and dependency ordering remain in the current
  [roadmap](../../../roadmap.md), including delegated remote MCP and approvals.

## Coverage Observations

The real Nighthawk configuration passed checked acceptance and capture-v3 pure
replay over authenticated SSH forwarding, with a loopback-only model listener.
Actual session requests reused their shared prefix in both cache-extension modes.
The [remote evidence](remote-deployment.md) records exact provenance and samples,
including why this establishes no causal cache-flag benefit. The owned temporary
server and tunnel were stopped; disconnected failure and replay were verified.

Linux metadata/same-user limits, trusted MCP executable non-escape, Unix orphan
reaping, hard-link filesystem constraints and lightweight MCP schema validation
remain explicit. This repair sprint does not certify hostile native-code
confinement or realize every proposed harness feature.
