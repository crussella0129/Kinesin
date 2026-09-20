# INT-0033 — Evidence-driven work recovery

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0033
- **State:** planned
- **Work evidence:** [T-119–T-124 build plan](../sprints/s15/sprint-plans/build-plan.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Make the local assistant require less babysitting than writing the code directly.
Qualify its ability to select and execute actions before adding another workflow
layer. Keep recorded operations, their outcomes and model claims distinct so
repair and subsequent requests start from what happened, including partial work
left by a failed run. This follows INT-0030's realized conversation memory and
supports INT-0032's still-unmet zero-correction app workload.

This intent does not declare a general semantic progress oracle. An executed
operation is activity; an answer is a claim; a working feature requires an
independent observation. Truthful stopping alone is not useful task completion.

## Acceptance criteria
- AC1: Qualify action selection using pinned runtime/model/wire identities and
  disposable, matched workloads. A new structured protocol preserves both tool
  and answer choices, aligns its grammar prefix with its instructions and uses
  explicit versioning. Partial/invalid actions never execute; legacy capture
  bytes and replay behavior remain reproducible. No global serializer-order
  switch or change to operator profiles is an acceptable shortcut.
- AC2: A bounded machine-owned summary distinguishes recorded authorized
  operations, their outcomes and model claims. A model answer cannot manufacture
  effects or erase recorded errors. Missing terminal observations have unknown
  outcomes; recorded errors have unverified repair status. Unknown behavior remains unverified; serving
  a page, writing bytes or passing a static scan never means its features work.
  Even a successful operation does not necessarily change bytes. Failed
  operations may have partial effects and must not be reported as proof
  of no change. Recovery shares the original authority and all existing budgets.
- AC3: Local session reference memory can retain bounded facts about partial or
  failed runs with run provenance, separately from assistant claims. Existing
  byte/turn admission, explicit eviction, reset and capture/privacy boundaries
  remain intact. Retained facts confer no grants and do not replace rereading
  files or revalidating a preview. Earlier session captures retain their meaning.
- AC4: Diagnostic and full-workload attempts retain failures and operator effort.
  Comparisons state the changed variable, prediction and falsifier; behavior
  observations identify the criterion, observer and artifact revision. Forward,
  backward, unchanged and unknown judgments require comparable observations;
  missing or stale observations never count as passes. Improvement on a tiny
  diagnostic does not satisfy INT-0032's unchanged storefront/follow-up criteria.
- AC5: Implementation follows bounded disposable live operation and repair.
  Official focused unit/integration checks and final lint/review follow the
  successful live confidence gate. Exhausted diagnostic or repair budgets lead
  to an explicit failed/inconclusive decision, not unrecorded profile roulette
  or a weaker acceptance criterion. No general reliability or measured human
  productivity claim is made without corresponding evidence.

## Rationale
Sprint 14's eight failures exposed more than generated application bugs. Its
structured grammar and instruction disagreed about the opening discriminator;
completion review could accept another unsupported assertion; receipts measured
operations rather than behavior; session memory retained successful answer prose
while omitting failed runs' real effects. Several experimental comparisons also
changed multiple variables. These defects make it difficult to know whether a
repair helped or merely changed the failure. Their causal contribution must be
distinguished from the code-level facts themselves.

The original request survives compaction, and most failed runs did not exhaust
their budgets. More context, a larger model, rigid operation stages or another
self-review prompt therefore have no established claim to solve the whole
problem. Independent observations must control outcome judgments; source-bound
facts must control the remembered account of what the harness did.

## Alternatives
Forcing the tool branch would conceal action-selection failure and break ordinary
answers. Mandatory read/write/run stages already caused false stops. Treating
the model's plan or answer as the authority for completion repeats the failed
design. A broad browser subsystem is deferred until a bounded repair contrast
establishes that supplying a real observation helps; it would not alone repair
action selection or missing features. A broad model/token sweep is also deferred.

## Consequences
Versioned wire and reference-state changes have replay and admission costs.
Facts must stay compact and exclude arbitrary file/tool payloads. Derive them
from durable events rather than a new model-authored work graph. General outcome
satisfaction, automatic replanning and semantic stalled states are deferred;
report exact existing terminal reasons. The first
implementation may honestly expose that behavior is unverified without making
the model more capable. That is useful diagnostic infrastructure, but the
parent usability intent remains unrealized until its live workload passes.
General persistent sessions remain INT-0026; wider benchmarking remains INT-0024.

## Transition history
- 2026-09-20: proposed → planned after the user explicitly approved Sprint 15's
  reviewed mitigation plans. Preserve the six-request diagnostic ceiling,
  conditional two-attempt live gate and official-verification-after-live order.
- 2026-09-20: created as `proposed` from Sprint 14's failed closeout and the
  owner's request for deeper causal mitigations in Sprint 15. Concrete plans
  remain subject to review and approval; no implementation or usability claimed.
