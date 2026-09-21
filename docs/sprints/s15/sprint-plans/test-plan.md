Finalized - DO NOT EDIT

# Sprint 15 Test Plan

User approved on 2026-09-20. Disposable live operation comes first; official unit/integration checks run only after T-123 passes.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
| --- | --- | --- | --- |
| [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md) | AC1 wire prefix | T-119 E1 | ordered_wire_prefix_and_budget; causal_order_pair |
| INT-0033; INT-0032 | AC1; AC2 protocol selection | T-119 E2 | protocol_selector_preserves_native_boundaries |
| INT-0033; INT-0032 | AC1; AC2 replay/invalid actions | T-119 E3 | legacy_and_ordered_capture_replay; invalid_actions_never_dispatch |
| INT-0033 | AC1 complete protocol output only | T-119 E4 | structured_stream_does_not_leak_arguments |
| INT-0033 | AC2 observed facts | T-120 E1 | effect_facts_are_not_claims; durable_effect_after_cancel |
| INT-0033 | AC2 bounded/private facts | T-120 E2 | effect_summary_bounds_and_incomplete_history |
| INT-0033 | AC2/AC3 partial-run retention | T-121 E1 | partial_run_reference_survives_failure |
| INT-0033 | AC3 admission/privacy/reset | T-121 E2 | reference_admission_privacy_and_reset |
| INT-0033 | AC2/AC3 honest terminal view | T-121 E3 | response_completion_is_not_work_verification |
| INT-0033; INT-0032 | AC3; AC2 frozen reference semantics | T-121 E4 | old_and_new_session_capture_identity |
| INT-0033 | AC1/AC4/AC5 diagnostic discipline | T-122 E1 | causal_order_pair; action_path_qualification; repair_feedback_contrast; diagnostic_budget_ledger |
| INT-0033 | AC4/AC5 causal interpretation | T-122 E2 | diagnostic_outcomes_and_effort |
| [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md); INT-0033 | AC3/AC4; AC4/AC5 live usefulness | T-123 E1 | zero_correction_storefront_and_followup |
| INT-0032; INT-0033 | AC4; AC4 honest progress | T-123 E2 | artifact_bound_progress_comparison |
| INT-0033; INT-0032 | AC5 and final AC1–AC4 regressions; AC2/AC4 | T-124 E1 | post_live_official_verification |

## Unit Tests
- **Intents:** INT-0033 AC1–AC3; INT-0032 AC2. Run only after T-123.
- `ordered_wire_prefix_and_budget` (T-119 E1): inspect actual encoded request
  bytes, not a re-sorted Value; both kind-first branches remain possible, tool
  name precedes arguments, unrelated bytes pass through, exact byte cap/hash
  reflects the sent body. No tokenizer/grammar acceptance claim from JSON alone.
- `protocol_selector_preserves_native_boundaries` (T-119 E2): omitted selector,
  eligible opt-in, checked/read-only/no-tool/MCP cases and forbidden combinations;
  verify capture tuple and unchanged native semantics, with no silent fallback.
- `effect_facts_are_not_claims` (T-120 E1): synthetic journal sequence distinguishes
  planned/denied/dispatched/result states, success/error/missing results and
  out-of-order/duplicate inputs. Model text cannot create or clear facts. Neither
  success nor error implies a specific byte change or behavior result.
  Conflicting, duplicate and noncontiguous inputs cannot turn an error into a
  successful result; MCP/unknown-tool activity remains visible as unsupported.
- `effect_summary_bounds_and_incomplete_history` (T-120 E2): exactly-at/over
  event/record/encoded-byte limits, escaping-heavy metadata, large identifiers,
  partial pages and missing results; whole-record omission and explicit incomplete
  state; no raw file content, command output or arbitrary tool/model payload.
  Exercise a short nonfinal page capped by bytes, scan through run_finished, and
  distinguish a known omission count from an unknown unscanned tail at the cap.
- `response_completion_is_not_work_verification` (T-121 E3): freeform final view
  and /status explicitly unverified, exact failure/cancel/stop reasons retained,
  no-tool answers still valid, checked verdict labels preserved.

## Integration Tests
- **Intents:** INT-0033 AC1–AC3; INT-0032 AC2. Run only after T-123.
- `legacy_and_ordered_capture_replay` (T-119 E3): retained versions, including
  native 7/4/5 and historical 8/5/5, plus ordered 8/6/5, reproduce their respective
  prepared hashes and transitions; offline replay has no model/tool side effects;
  unsupported tuples fail admission. Include old milestone captures still supported.
  Assert complete capture/core/adapter/tools tuples: old-order lab 4/8/5/5,
  ordered no-reference 4/8/6/5, native reference 5/7/4/5 and ordered reference
  5/8/6/5, plus retained legacy tuples. Reject new reference shapes under capture
  3/4 and reject 5/8/5/5; native without references stays 4/7/4/5.
  Historical adapter-5 captures with valid legacy context remain replayable;
  only new lab-control calls require empty context.
- `invalid_actions_never_dispatch` (T-119 E3): a controlled model transport returns
  truncated JSON, duplicate/unknown fields, unavailable tools and valid-but-denied
  actions; actual journal/files show no unauthorized or partial dispatch. Include
  cancellation/budget exhaustion around repair; do not replenish run budgets.
- `structured_stream_does_not_leak_arguments` (T-119 E4): fragmented and truncated
  streamed actions emit no raw protocol JSON or argument deltas; only complete
  validated answers render as text, with complete authorized actions dispatching
  exactly once. Existing native streaming behavior stays unchanged.
- `durable_effect_after_cancel` (T-120 E1): record a real successful effect, then
  cancellation before core observation; terminal summary still includes its
  durable result. A planned effect without a result remains unknown.
- `partial_run_reference_survives_failure` (T-121 E1): run writes then fails;
  subsequent interactive request receives actual bounded reference/terminal reason
  separately from model claims. An absent answer is accepted only with valid
  reference data; a model claim with no journal effects cannot manufacture facts.
- `reference_admission_privacy_and_reset` (T-121 E2): 16-turn/8-KiB and tighter
  profile admission, deterministic clipping/eviction notices, /new and /clear;
  whole-record/reference omission preserves typed IDs/resource identities instead
  of clipping facts into plausible but false paths or malformed JSON;
  evict answerless turns when their sole reference can no longer fit;
  metadata-only capture contains no reference bodies, opt-in capture does; fresh
  grants deny an operation even when prior facts claim it was once authorized.
  Include changed/missing referenced files, owner mismatch and untrusted text.
- `old_and_new_session_capture_identity` (T-121 E4): old optional-field omission
  reproduces byte-identical context/request hashes; new versioned references are
  frozen and hashed, unsupported shapes fail closed, replay requires no current
  file/preview access and retains old checked acceptance semantics.
- `post_live_official_verification` (T-124 E1): record live-gate and check timestamps,
  final source identity, focused Rust results, `cargo fmt --all -- --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`, and accepted
  independent critique. Include retained S14 recovery/tool/profile behavior in
  affected coverage before claiming the candidate verified. Recheck only behavior
  affected by a later material fix; report if the frozen live budget is exhausted
  and requires explicit replan rather than silently granting more attempts.

## End-to-End Tests
- **Status:** possible
- **Intents:** INT-0033 AC1/AC4/AC5; INT-0032 AC3/AC4.
- `causal_order_pair` (T-119 E1, T-122 E1): diagnostic calls 1/2, same tiny
  create/read/amend request, correct bytes in independent copies, old versus
  corrected schema order only. Retain actual wire/grammar evidence and both
  results. A parsed tool proposal without execution is a failure of the workload.
  First wire bodies may differ only in schema order; record the first branch/action
  separately and do not require later output-dependent histories to be identical.
- `action_path_qualification` (T-122 E1): calls 3/4 use the documented held-out
  native/ordered contrast or native create/amend fallback. Freeze which branch
  was selected and why before dispatch. Require correct bytes; report protocol
  comparison separately from same-session memory qualification.
  If both held-out paths fail, call 2's earlier success cannot qualify a path.
  Both create and amend must pass in the native fallback branch. Paired arms
  have independent empty sessions as well as identical initial files; only the
  explicitly named fallback carries prior context.
- `repair_feedback_contrast` (T-122 E1): calls 5/6 on identical one-defect copies;
  only the second receives authentic hash-bound browser observations. Check the
  seeded failure before repair, actual effects and failing interaction afterward,
  plus an unaffected control. If a control no longer works, this is a regression,
  not a successful repair. Assisted-only success cannot unlock T-123 without a
  separately planned automatic-observation capability.
- `diagnostic_budget_ledger` (T-122 E1): at most six top-level harness requests,
  exact branch/stop decisions, no replacement slot for a failed/abandoned request,
  original caps and correct old/new tuples; no model/profile sweeps. Record all
  repairs inside their parent request and operator work outside it.
- `diagnostic_outcomes_and_effort` (T-122 E2): every row includes actual effect or
  browser evidence, changed variable, prediction/falsifier, supported/falsified/
  inconclusive result, elapsed/model/operator time and manual intervention count.
  Unavailable measurements stay unknown; no fabricated human-authoring baseline.
- `zero_correction_storefront_and_followup` (T-123 E1): only after small unaided
  repair qualifies, at most two fresh empty-workspace attempts with original
  prompts/ceilings. Independently operate six products, search/category filtering,
  cart quantity/removal/totals, reload persistence, valid/invalid synthetic checkout,
  Clear Cart and summed item count after follow-up. Verify preserved original
  behavior, server continuity and shutdown. Zero corrective messages, code patches,
  tool forcing or resets in the accepted attempt; every failure retained.
- `artifact_bound_progress_comparison` (T-123 E2): criterion/observer/revision/receipt
  records distinguish gained and lost behavior. File changes invalidate prior
  relevant observations until observed again; missing/stale/incompatible records
  stay unknown. Equal criterion states count unchanged only after the action is
  known to have run. These are independent assessment records, not a new product
  semantic oracle or a way to upgrade freeform acceptance to checked.

The full live gate—not the diagnostic pair, effect history, truthful status or
passing formal checks—establishes INT-0032's usefulness outcome. If qualification
fails, record the failed sprint decision and leave official suites not run. A
future automatic browser observer requires its own scoped research/plan under
INT-0033 and INT-0032; it is not preauthorized by this test plan.

