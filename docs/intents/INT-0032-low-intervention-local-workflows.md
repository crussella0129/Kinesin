# INT-0032 — Useful local work with less operator correction

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0032
- **State:** active
- **Work evidence:** [T-115–T-117 plan](../sprints/s14/sprint-plans/build-plan.md); [T-118 failed-sprint assessment](../sprints/s14/failure-report.md); [T-119–T-124 follow-on plan](../sprints/s15/sprint-plans/build-plan.md); [Sprint 15 failed qualification](../sprints/s15/failure-report.md); [Sprint 16 failed qualification](../sprints/s16/failure-report.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Make Kinesin useful enough that the operator spends less time babysitting it
than writing the code themselves. Automatically recover from common local-model
workflow failures and review premature completion using actual tool receipts.
Demonstrate the result on a fresh small app and a natural follow-up without
manual code corrections, tool-forcing prompts or context resets. This extends
INT-0031's operator-steered delivery; it does not claim general autonomous coding
competence or complete INT-0024's broader evaluation program.

## Acceptance criteria
- AC1: For freeform work with compiled effectful grants, reject observable
  tool-protocol-looking prose and allow at most two automatic repair nudges for
  that failure, empty replies or generation truncation. Separately, allow one
  completion review per run comparing the original request with summaries of
  actual successful/failed tool receipts, bounded to 12 entries and 4,096 encoded
  JSON bytes with no raw payloads. Do not execute prose, invent receipts
  or force writes for questions. Review remains unchecked and cannot establish
  arbitrary app correctness; exhausted rejection retains no terminal candidate.
- AC2: Recovery consumes the existing time, turn, tool-call, request/history and
  output budgets, plus an explicit finite recovery ceiling. Exhaustion ends
  truthfully. Grants, checked-task semantics, journal ownership and replay remain
  intact; older captures retain their recorded behavior. Error guidance does
  not reveal outside/private files or grant missing effects.
- AC2a: Generic tool guidance preserves exact-edit refusal and advises rereading;
  nonzero command exit is an explicit error with useful bounded stderr. Fresh
  local profiles use context 16,384, output 2,400 tokens, 20 model turns,
  30 tool calls and 240 seconds. Existing operator profiles remain unchanged.
- AC3: Before a live attempt, freeze the initial app request, natural follow-up,
  independent outcome checks and resource ceilings in a workload card. In a new
  isolated workspace, the real model/harness creates and previews a storefront
  with catalog/search, cart quantity/removal/totals, synthetic checkout and the
  requested follow-up. The accepted attempt requires zero corrective operator
  prompts, code patches, tool-forcing instructions or context resets; initial
  task and planned natural follow-up are the only task messages.
- AC4: Retain failed attempts as well as the final result. Record model/profile
  and source identity, prompts, tool events, wall time, active operator time,
  automatic recoveries and manual interventions. Browser/HTTP effects establish
  functional success independently of model completion claims. Report one
  workload honestly; do not claim a measured human-authoring speedup without
  a measured baseline. Official focused unit/integration verification and
  independent critique follow a successful live confidence gate.

## Rationale
Sprint 13 proved the harness can perform effects, but the operator repeatedly
had to supply exact edits, prohibit fake tool responses and clear context.
That operating cost fails the user's usefulness goal even when the eventual
storefront works. The next improvement must reduce intervention directly.

## Alternatives
Count tool invocations or passing unit tests as usefulness (wrong outcome);
hide manual patches in a successful demo (misleading); execute tool-shaped
prose (changes authority/protocol); embed storefront-specific code in recovery
(does not improve the general harness); remove all ceilings (unbounded work).

## Consequences
Recovery adds bounded inference cost and may still fail on the selected model.
Receipt-grounded completion review is workflow assistance, not an independent
correctness verifier; freeform results remain unchecked. Preserve failure data
and report limits plainly. The live zero-correction rubric is a concrete proxy
for reduced babysitting, not a universal productivity or reliability benchmark.
Broader workload comparisons and human-effort baselines remain INT-0024/T-103.

The first live attempt showed that a receipt-only completion review could repeat
unsupported feature claims. A bounded constrained plan of at most six generic
milestones may therefore guide execution before the one final completion review.
Milestone kinds distinguish reading, changing files, running a command, previewing
and answering. Non-answer progress requires a successful matching operation from
that stage; missing witnesses share the existing two-repair ceiling. Planning and
all stages consume the original run budgets and confer no authority. These
witnesses do not prove app correctness or planner completeness: independent
browser outcomes remain mandatory. Core-5 behavior retains the failed core-4
attempt's exact replay semantics. The [recorded implementation refinement](../sprints/s14/sprint-research/implementation-adjustment.md)
preserves the failed evidence and the unchanged two-prompt live gate.

The second live attempt stopped without a candidate after file-scaffold planning
and actual empty provider responses. Core 6 may use concise requirement-bearing
milestones and expose granted tool names to planning. An empty response after an
actual matching stage operation may advance on that witness alone, with an
explicit completion basis; it cannot become a candidate or replace final review.
Empty replies without a witness retain the shared two-repair limit. Earlier
core-4/core-5 captures retain their behavior, and browser acceptance is unchanged.

Attempt 3 and diagnostic operation showed that mandatory stages can false-stop
valid work without proving useful behavior. Core 7 removes enforced planning and
stage witnesses for new runs, while preserving core-4/core-5/core-6 replay.
Short generic work instructions retain two repair nudges and one final review.
They preserve file subdirectory context and observe prerequisite results before
dependent tools. Tools 5 adds bounded missing-file guidance to include the
subdirectory or list the parent only if already granted; it performs no listing
and confers no new capability. Adapter 4 and historical version behavior remain
unchanged. Final review directs attention to actual preview warnings and a fresh
same-path observation after repairs.
T-115 also extends to preview index compatibility warnings: successful startup
returns its real URL, and repeated preview calls rescan current bounded index
bytes. Inline JavaScript/handler/style warnings and static GET/HEAD-only guidance
provide actionable observations without weakening CSP, grants, ownership or
budgets. These warnings are not browser proof; no warnings cannot establish a
pass. The three failed attempts and failed diagnostic probes remain failures,
and the frozen independent live gate remains binding.

After four failed scored attempts on the same 7B model profile, a separately
frozen Qwen3-8B profile comparison may use unchanged core-7 source/binary,
adapter, user prompts and resource ceilings. Model weights, sampling and the
generic thinking-mode instruction are explicit profile differences, not proof
of causal harness improvement. Preserve the baseline failures and unexecuted
first-action diagnostic; the independent browser gate and post-live official
verification order remain unchanged. This bounded comparison does not complete
INT-0024's broader model evaluation program.

After the non-thinking comparison failed, a separately frozen attempt may keep
core 7 and the same Qwen3 weights while allowing reasoning at temperature 0.6
with 4,096 output tokens instead of 2,400. This explicit profile/output-budget
change preserves all other caps, exact user prompts, required features and zero
operator-correction acceptance. It changes no production default and supports
no isolated harness-effect claim; retain the failed comparison and original
baseline. Official checks remain after live confidence.

After six failed attempts, core 8 / adapter 5 may use a bounded structured
single-action protocol for freeform runs whose grants are all compiled tools
and include an effectful operation; tools 5 stays unchanged. A complete valid
declared JSON tool-or-answer response may become a typed proposal, never an
automatic permission or interpretation of arbitrary prose. Existing dispatch
checks remain authoritative. Partial/malformed/unknown-field responses cause no
effect; protocol repairs share the two-nudge ceiling and the one final review.
Raw action JSON/provisional arguments must not leak through text streaming.
Checked, read-only and all MCP-containing runs retain native behavior, earlier
versions retain replay, and mandatory planning stages remain absent. Attempt 7
restores the original Qwen2.5 profile and 2,400-output baseline; schema validity
is not usability, and only the unchanged independent live gate can establish
the requested result. The earlier one-action proposal and six failed attempts
remain retained; no broader success or task completion is implied.

Attempt 7's schema-valid answers produced no actions. One bounded counterpart
comparison may keep core-8/adapter-5 source and binary fixed while restoring
attempt 5's Qwen3 non-thinking profile, temperature 0.7 and 2,400 output tokens.
Original prompts, other ceilings and the independent browser/intervention gate
remain unchanged. This changes no defaults and cannot prove broad reliability.
Preserve all failures; another failure requires an observation/repair strategy
decision rather than further blind prompt variations.

Attempt 8 also produced only two answers and no effects. Reject the structured
experiment as the default and restore exact core-7/adapter-4/tools-5 new-run
semantics. Keep core-8/adapter-5 captures replayable through an explicitly
historical version path; preserving failed evidence does not require retaining
the failed strategy for new runs. Native attempt 5's file/product creation is
evidence of a sampled capability regression, not evidence that native mode is
usable. The acceptance gate remains unmet, T-115–T-117 remain queued/incomplete,
and official checks remain deferred. The next design obligation is bounded
repair/completion driven by actual independent observations, not further blind
prompt/profile variations. No new observation capability or success is claimed.

The user-requested [reference study](../sprints/s14/sprint-research/reference-loop-principles.md)
reinforces that a work-step witness is evidence of activity, not semantic
progress. Any future forward/backward/unchanged/unknown classification must use
comparable criterion observations bound to artifact hashes and observer identity,
independently of model completion claims. Changed bytes alone are not forward
progress; missing, stale or incomparable observations remain unknown. Such
classification cannot upgrade unchecked acceptance or replace the independent
live outcome gate.

For subsequent usability sprints, the user's standing preference is disposable
live operation and repair first, followed by focused official unit/integration
checks once the working flow is convincing. Avoid repeated test cycles that
delay operating the product. Compilation and read-only inspection may support
the live repair work; this preference does not remove final verification.

Sprint 16 retained the unchanged usefulness gate. Its unaided, filename-assisted
and full-source-assisted repairs all produced zero tool calls and no preview.
The planned conditional listing was not implemented; no full storefront or
same-session follow-up ran and official checks remain deferred. The desired
outcome is still less time babysitting than writing the code directly. A more
detailed model explanation did not constitute an improvement in applied work.
State remains active and unrealized; future action-selection research must
preserve the actual browser and zero-correction criteria.

## Transition history
- 2026-09-20: remained active/unrealized after Sprint 15's bounded diagnostics.
  Two small file-task passes did not transfer to either app-repair request; the
  unchanged storefront/follow-up was not run and official checks remain deferred.
  Retain failed evidence and unverified source; T-125 plans a narrower discovery
  investigation rather than weakening this intent's usability criterion.
- 2026-09-20: owner requested Sprint 14 closure as failed after eight failed
  live attempts. Intent remains active and unrealized; T-115–T-117 return to
  backlog. T-118 archives failure evidence and unverified source, with no
  product-completion claim. Sprint 15 must research deeper causal mechanisms
  and replan mitigations before further implementation.
- 2026-09-20: created as proposed after the user rejected an operator-intensive
  result as insufficiently useful and requested less time babysitting than
  writing the code themselves.
- 2026-09-20: proposed → planned for core-4 bounded recovery, tools-4 error
  guidance, adapter-4 descriptions and the frozen two-prompt live workload.
- 2026-09-20: planned → active after the independent plan critic accepted the
  scope and finalize-plan.sh locked both plans; formal checks remain post-live.
- 2026-09-20: recorded the user's reiterated live-first preference for later
  usability sprints; current locked plans already follow this order.
- 2026-09-20: refined active implementation after failed live attempt 1 to add
  at most six bounded, witnessed execution milestones under core 5. This changes
  neither effect authority nor independent acceptance; prior captures and failed
  attempts remain retained, and all existing run/recovery ceilings still apply.
- 2026-09-20: refined active implementation after failed live attempt 2 with
  core-6 feature-based planning and witnessed-stage handling of actual empty
  replies. No acceptance criterion, operator correction allowance or resource
  ceiling changes; the failed one-prompt attempt remains visible.
- 2026-09-20: independently reviewed and accepted the core-7 refinement after
  failed attempt 3 and diagnostic operation: remove new-run stage gates and add
  fresh bounded preview compatibility feedback under T-115. Preserve prior
  replay, all ceilings, failed evidence and the independent live acceptance
  criteria. This records a design decision, not implementation completion.
- 2026-09-20: recorded a separately frozen model-profile comparison after four
  failed baseline attempts, with core-7 source/binary and the live rubric held
  fixed. No scope, acceptance, adapter or resource-ceiling change; no successful
  result is implied by downloading a model or proposing one JSON action.
- 2026-09-20: recorded attempt 6's reasoning-mode/4,096-output comparison after
  attempt 5 failed browser behavior. Same core-7 harness and Qwen3 weights; all
  other caps and behavior/intervention criteria remain fixed. Production
  defaults and original baseline evidence remain unchanged.
- 2026-09-20: independently reviewed and accepted the bounded core-8/adapter-5
  structured single-action experiment within T-115 after six failed attempts.
  Attempt 7 restores the original model/profile and ceilings, preserves normal
  authority and all legacy replay, and retains the independent outcome gate.
  This is an explicit implementation refinement, not completion evidence.
- 2026-09-20: recorded attempt 7's zero-action failure and one unchanged-harness
  counterpart comparison using attempt 5's model profile. The full live rubric
  and post-live official verification order remain binding; no success claimed.
- 2026-09-20: independently accepted rejection of the structured experiment after
  attempt 8 also made no effects; restore exact core-7/adapter-4/tools-5 defaults
  while preserving experimental capture replay. Usability remains unmet and the
  next design obligation is independent observation-driven repair/completion.
