Finalized - DO NOT EDIT

# Sprint 14 Test Plan

Draft. Formal unit/integration checks follow a zero-correction live workload.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
| --- | --- | --- | --- |
| [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) | AC1 completion/prose recovery | T-115 / premature completion | recovery_uses_actual_receipts; prose_never_dispatches |
| INT-0032 | AC1 empty/truncated/error recovery | T-115 / recoverable failure | bounded_failure_recovery; exact_edit_recovery_guidance |
| INT-0032 | AC2 limits, cancellation and authority | T-115 / exhausted/cancelled | recovery_budget_and_authority_boundaries |
| INT-0032 | AC2 new/legacy replay | T-115 / replay | recovery_capture_and_legacy_replay |
| INT-0032 | AC2a honest useful tool errors | T-115 / command/edit failure | command_failure_and_reread_guidance |
| INT-0032 | AC2a new defaults/preserved existing profiles | T-115 / fresh profile | fresh_workflow_defaults_preserve_existing |
| INT-0032 | AC3 zero-correction app/follow-up | T-116 / frozen requests | fresh_app_without_corrections |
| INT-0032 | AC3 independent functional outcome | T-116 / browser checks | fresh_storefront_browser_and_followup |
| INT-0032 | AC4 retained failures and effort | T-116 / attempt finishes | live_attempt_effort_record |
| INT-0032 | AC4 checks after live confidence | T-117 / workload passes | post_live_verification_record |

## Unit Tests
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- `recovery_uses_actual_receipts`: T-115 first clause; distinguish actual
  successful effects, denied/errors and unsupported prose-only claims; review
  does not convert freeform work into independently checked acceptance. Receipt
  bounds of 12 entries/4,096 encoded JSON bytes survive compaction without
  retaining arbitrary payloads;
  two repair nudges and one separate review remain independent ceilings.
- `bounded_failure_recovery`: T-115 second clause; empty/truncated outputs receive
  finite recovery, valid protocol actions may continue, partial calls never run.
- `exact_edit_recovery_guidance`: T-115 second clause; a missing/ambiguous match
  retains denial and useful reread guidance without broad replacement or leakage.
- `recovery_budget_and_authority_boundaries`: T-115 third clause; exact retry
  ceiling, exhausted turn/history/request/time budgets, cancellation and unchanged
  checked/no-tool contracts. Failed recovery cannot replenish any budget.
- `command_failure_and_reread_guidance`: T-115 fifth clause; nonzero exit is a
  top-level error, bounded stderr survives stdout noise, edit refusal suggests
  rereading without returning outside/private data or weakening exact matching.
- `fresh_workflow_defaults_preserve_existing`: T-115 sixth clause; new-profile
  values match the promised ceilings; preexisting operator values are unchanged.

## Integration Tests
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- `prose_never_dispatches`: T-115 first clause; actual runner/journal retain
  fabricated tool-result text as model prose, with no corresponding effect.
- `recovery_capture_and_legacy_replay`: T-115 fourth clause; capture actual new
  recovery sequence and compare request hashes; replay after deleting workspace;
  old captures keep old terminal/denial semantics with no model/tool call.
- `post_live_verification_record`: T-117; final source identity, gate timestamp,
  focused suite outcomes, format/Clippy and accepted independent critique.

## End-to-End Tests
- **Status:** possible
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- `fresh_app_without_corrections`: T-116 first clause; frozen card run in empty
  workspace with only initial request and natural follow-up; assert zero manual
  code patches, corrective prompts, tool-forcing instructions and context resets.
- `fresh_storefront_browser_and_followup`: T-116 second clause; independently
  operate catalog/search, quantity/removal/totals, checkout and requested follow-up
  using visible browser behavior and actual HTTP; no model self-grading.
- `live_attempt_effort_record`: T-116 third clause; retain every attempt and
  configuration change with timing/intervention counts, including failures and
  uncertainty. One successful run is not a general reliability measurement.
- The live confidence gate requires all three scenarios above. Only afterward
  run focused affected Rust suites, `cargo fmt --all -- --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`, and critic.
