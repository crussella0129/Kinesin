# Sprint 13 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
| --- | --- | --- | --- | --- |
| [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md) | AC1 scoped workspace and provenance | T-112 / live_workspace_provenance; retained prompts, tool events and manifest | pass | Completion and Test evidence linked |
| INT-0031 | AC2 Kinesin-authored storefront and follow-up | T-112 / live_harness_authorship; T-113 / live_storefront_browser | pass | Actual file effects and browser outcomes linked |
| INT-0031 | AC3 owned confined local preview and lifecycle | T-113 / live_server_lifecycle; preview_capability_integration; preview_denies_outside_paths; preview_service_config_rejected | pass | Code and executed negative-path evidence linked |
| INT-0031 | AC4 live repair, preserved boundaries and verification order | T-113 / live_failure_recovery, repaired_boundary_units, repaired_server_integration; T-114 / deferred_verification_record | pass | This report and accepted critic linked |

## Summary
- Unit tests: 206 passed, 0 failed, 206 total.
- Integration tests: 65 passed, 0 failed, 65 total across runner_tools,
  runner_journal, managed_cli, replay and preview. Repeated preview execution
  is not counted twice: 271 distinct passing cases overall.
- E2E: all five named live scenarios passed after retained failures and
  operator-guided repairs. The initial confidence gate preceded official
  testing; the final rebuilt copy was reconfirmed, including a $25 checkout.
- Formatting and all-target/all-feature Clippy with warnings denied passed.
- Final independent [critique](critique.md): clean.
- CI status: not run for this local sprint; no remote checkpoint authorized.

## CI Confirmation
- **Tested implementation SHA:** `d9547f8865b0703919f3ba3aea1f75abd7432932`.
  Commands ran against the final working tree before its task commit;
  [tested-source.json](tested-source.json) matches all eleven changed Rust
  source/test files in that commit. The unchanged baseline is recorded there.
- **CI run:** none. Hosted CI exists, but was not run or represented as green.
- **Local conclusion:** success on native Windows only.
- **Confirmations:** [unit/static records](unit-tests.md),
  [integration records and exact named assertions](integration-tests.md),
  [live tool/browser evidence](e2e-tests.md), and the accepted critique above.

## Failures
Earlier model prose-only claims, generated JavaScript and checkout defects,
exact-edit misses, output limits and the foreground-server workflow wall are
retained in E2E evidence. The server lifecycle and replay/deadline repairs were
implemented before final verification. Initial linker failures were caused by
disk exhaustion; deleting verified generated build artifacts allowed successful
reruns. There are no unresolved failures in the final selected checks.

## Technical Debt Identified
- [INT-0024](../../../intents/INT-0024-harness-evaluation.md), existing T-103:
  include prose-only tool claims, follow-up repair/context reset and app-generation
  failures in reproducible model-quality workloads. This sprint does not claim
  general unattended reliability or complete the wider evaluation program.

## Coverage Observations
The model authored and revised the app through real tools, with substantial
operator steering disclosed. Actual browser interactions and HTTP responses
prove the basic storefront outcome. Static previews are local CLI only and
do not execute arbitrary backend code. Linux execution, remote CI, production
design quality and a full unrelated suite were outside this pass.
