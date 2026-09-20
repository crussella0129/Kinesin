# INT-0032 — Useful local work with less operator correction

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0032
- **State:** planned
- **Work evidence:** [T-115–T-117 plan](../sprints/s14/sprint-plans/build-plan.md)
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

## Transition history
- 2026-09-20: created as proposed after the user rejected an operator-intensive
  result as insufficiently useful and requested less time babysitting than
  writing the code themselves.
- 2026-09-20: proposed → planned for core-4 bounded recovery, tools-4 error
  guidance, adapter-4 descriptions and the frozen two-prompt live workload.
