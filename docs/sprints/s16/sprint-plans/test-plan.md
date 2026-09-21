Finalized - DO NOT EDIT

# Sprint 16 Test Plan

Approved by the user on 2026-09-21. Live operation and bounded repair happen first. Official
unit/integration suites and Clippy run only after T-128 passes the unchanged full
workload. Formatting, compilation and read-only preparation may support earlier
live operation. No request or official check has run under this proposed plan.

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
| --- | --- | --- | --- |
| INT-0033 | AC1, AC4, AC5 | T-125 E1 | diagnostic_freeze_and_budget_audit |
| INT-0033 | AC4–AC6 | T-125 E2 | diagnostic_branch_and_effect_oracle |
| INT-0033; INT-0032 | AC6; AC2 | T-126 E1 | grounding_admission_and_disabled_compatibility |
| INT-0033; INT-0032 | AC2, AC6; AC2 | T-126 E2 | grounding_accounting_and_settlement |
| INT-0033 | AC2, AC6 | T-126 E3 | grounding_observation_and_context_bounds |
| INT-0033; INT-0032 | AC2, AC3, AC6; AC2 | T-126 E4 | grounding_capture_replay_and_reference_origin |
| INT-0033; INT-0032 | AC4–AC6; AC3, AC4 | T-127 E1 | two_normal_request_repairs |
| INT-0033 | AC4, AC5 | T-127 E2 | diagnostic_behavior_and_effort_comparison |
| INT-0032; INT-0033 | AC3, AC4; AC4, AC5 | T-128 E1 | paper_harbor_same_session_live |
| INT-0032; INT-0033 | AC4; AC4, AC5 | T-128 E2 | full_attempt_retention_and_retry_gate |
| INT-0033; INT-0032 | AC1–AC6; AC2, AC4 | T-129 E1 | final_focused_verification_and_review |
| INT-0033; INT-0032 | AC4, AC5; AC4 | T-129 E2 | task_and_intent_disposition_audit |

The linked authorities are [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
and [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md).
Named live checks below are concrete independent operator observations, not a
new automated test framework or permission to exceed request budgets.

## Live Preparation and Diagnostics

- **diagnostic_freeze_and_budget_audit** — T-125 E1: before A, archive original
  and held-out seed bytes, every conditional prompt, actual reproduced seed
  defect/control observations and oracle arithmetic. Freeze source/model/runtime/
  profile/binary and accepted UTF-8/LF bytes; inspect actual first requests.
  Four slots shared with T-127, consumed on submission, no replacements.
- **diagnostic_branch_and_effect_oracle** — T-125 E2: independently apply the
  A→unaided-held-out, A-fail/B-pass→implementation, or A/B-fail→source-study
  branches. Retain list/read/edit/result records and actual previews. Original
  repaired cart must show 2+5=7/count 2, then +2=9/count 3; both search queries,
  clearing search and catalog prices remain correct. Held-out removal must clear
  both units/full line value while controls remain correct. Missing behavior is
  unqualified. Source-only wins stop before implementation/full workload.
- **two_normal_request_repairs** — T-127 E1: either A and unaided held-out slot 2,
  or post-T-126 original and held-out slots 3/4, pass in fresh sessions without
  operator context/repairs. For grounded runs count the initialization separately
  from model-selected tools. Observe actual owned preview while the session lives,
  then verify shutdown. An assisted earlier pass never replaces a normal one.
- **diagnostic_behavior_and_effort_comparison** — T-127 E2: bind every behavior
  to observer/time/current artifact hashes; compare actual effects and criteria,
  retaining failures. Record model/tool turns, origin, request/model time, token
  counts when supplied, retries and operator interventions. Time spent preparing
  the lab is human effort; missing active-time measurement stays unknown.

## Unit Tests — only after the full live gate

Tests are focused behavioral regressions, not exhaustive permutations. Existing
coverage is reused where it proves the same invariant. T-126 checks are required
only if its conditional implementation actually occurred; skipping the feature
does not waive verification of retained S14/S15 source.

- **grounding_admission_and_disabled_compatibility** — T-126 E1,
  INT-0033 AC6/INT-0032 AC2: absent/none serializes exactly as before and causes
  no listing; explicit eligible selection freezes; missing list grant, checked,
  service/foreign-owner, native-protocol or MCP-containing requests reject before
  any filesystem/model effect. Data in model/session content cannot enable it.
- **grounding_accounting_and_settlement** — T-126 E2, INT-0033 AC2/AC6:
  one harness-origin effect precedes model dispatch, charges one tool call and
  no fictitious model turn; exhausted tool/time budgets prevent new dispatch;
  cancellation/deadline/journal failure during listing keeps worker ownership
  until settlement and prohibits a later model call. No retry is synthesized.
- **grounding_observation_and_context_bounds** — T-126 E3,
  INT-0033 AC2/AC6: empty, truncated, non-UTF-8/error and adversarial-name results
  remain actual observations, not instructions or proof of absence. Exact result
  cap and serialized history/request limits include framing; overflow stops
  before HTTP. No source content, recursion or user-prompt alteration occurs.
- **grounding_reference_origin_and_eviction** — component of T-126 E4,
  INT-0033 AC2/AC3/AC6: typed harness/model origin cannot come from model claims;
  missing/conflicting result stays unknown; omitted facts remain explicit; old
  v1 encodings stay unchanged; mixed v1/v2 references, failed/cancelled runs,
  whole-record eviction, metadata exclusion and reset respect existing bounds.

## Integration Tests — only after the full live gate

- **grounding_capture_replay_and_reference_origin** — T-126 E4,
  INT-0033 AC2/AC3/AC6, INT-0032 AC2: run through actual admitted filesystem/tool/
  journal path with safe synthetic files; verify before-first-model observation
  and exact request fingerprint. Private capture replays without filesystem/model
  access; metadata has no directory names/body. Tampered origin/grant/selector/
  dispatch/counter/result/tuple fails replay. Capture 6/core 9 combinations and
  origin-aware reference versions work with eligible adapter selection; old
  capture 3/4/5 reject new shapes and retain their frozen requests.
- **legacy_protocol_and_partial_work_regressions** — part of T-129 E1,
  INT-0033 AC1–AC3, INT-0032 AC2: retained native/ordered/old structured captures,
  complete/invalid/stream-truncated action handling, fail-closed eligibility,
  reducer bounds/conflicts, failed-run facts, CLI admission and same-session
  reference/reset paths. Include still-retained S14 recovery/tool behavior.
  Do not call old sprint tasks complete merely because new related checks pass;
  map what was retained/replaced and its actual evidence.

## End-to-End Tests

- **Status:** possible with the local model, isolated apps and browser, but gated
  by T-127; no model reliability or result is presumed.
- **paper_harbor_same_session_live** — T-128 E1, INT-0032 AC3/AC4 and INT-0033
  AC4/AC5: exact retained initial/follow-up text, original caps and zero
  corrective interventions. Observe six products, search/category filters,
  multiple additions, quantity increase/decrease, removal, independently correct
  totals and reload persistence. Valid synthetic name/email checkout confirms
  correct amount and empties cart; empty-cart/invalid-email cannot order. Only
  after the initial gate, send the exact follow-up in the same session; summed
  item count and Clear Cart/reset/reload work while catalog, quantities and
  checkout still work. Preview survives between requests and closes with session.
  Archive before/after files, source/reference/tuple identities and browser/HTTP
  observations separately from model claims.
- **full_attempt_retention_and_retry_gate** — T-128 E2: maximum two fresh full
  attempts; failed initial gate means no follow-up. A second attempt requires a
  documented in-scope generic fix, new binary/source freeze and unchanged task/
  profile/oracle. No second attempt after success is required; source-only or
  failed repair qualification means zero full attempts.

## Final Verification and Disposition

- **final_focused_verification_and_review** — T-129 E1: after actual full pass,
  add/run the affected Rust unit/integration groups above and existing relevant
  replay/CLI/runner/preview/recovery groups. Run cargo fmt and cargo clippy with
  warnings addressed. Independent final critic maps every executed EARS to evidence.
  New material changes require affected live re-observation. Stop broadening once
  those checks pass without an unresolved concern. No official tests on failed gate.
- **task_and_intent_disposition_audit** — T-129 E2: T-119–T-121 and T-115–T-117
  keep their historical failure/incompleteness; list retained/replaced/deferred
  portions with current evidence. T-123/T-124 were not run in S15; new work is
  recorded here. Explicitly reschedule skipped T-126, preserve every failed
  attempt and only realize intents if all required criteria are demonstrated.

If any prerequisite fails, record the failed/inconclusive result and leave later
checks not run. A clean diagnosis or honest status cannot substitute for a useful
app. No new model budget, weaker criterion or broader context subsystem is implied.
