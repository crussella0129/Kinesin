# Sprint 14 Research Report

## Intents Reviewed
- [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) —
  created; planned reduction of operator correction with bounded recovery and
  a frozen zero-correction live workload.
- [INT-0024](../../../intents/INT-0024-harness-evaluation.md) — revised to record
  intervention cost; broader multi-profile evaluation remains proposed/T-103.
- [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md) — selected;
  realized preview/file capabilities and retained failures are the baseline.
- [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md) — selected;
  realized bounded memory remains unchanged in scope.

## 1. Sprint Goal
Reduce the user's babysitting burden: let the harness recover from common
model/tool failures and complete a new storefront plus a natural follow-up
without manual patches or tool-forcing prompts. Preserve actual outcome and
effort evidence, then run focused official checks after live confidence.

## 2. Existing Code Survey
| File | Relevance | Notes |
| --- | --- | --- |
| src/core.rs | high | Answer commits or empty/incomplete stops; owns pure transitions. |
| src/runner.rs | high | Actual tool receipts, budgets, journal and model dispatch. |
| src/replay.rs | high | Reconstructs transitions and old-version semantics. |
| src/tools.rs | high | Exact-edit failure and structured tool error observations. |
| src/model.rs | high | Tool protocol and compiled tool descriptions. |
| src/onboarding.rs | medium | Generic local operator instructions. |
| src/cli/session.rs | medium | Bounded reference history; reset was manual in prior demo. |
| docs/sprints/s13/sprint-tests/e2e-tests.md | high | Prose-only claims, truncation, edit misses and operator steering. |
| docs/sprints/s13/sprint-tests/test-report.md | high | Explicit distinction between effects and unattended usefulness. |

## 3. External Sources
None: implementation uses existing state-machine/tool interfaces and the
installed local model. No new library or backend API is needed.

## 4. Risks, Unknowns, Dependencies
- Completion self-review is another model observation, not proof of correct
  code. Require actual tool receipts and independent final browser behavior.
- Generic recovery may consume time or nudge ordinary answers unnecessarily;
  keep it bounded, scoped to freeform tool work and separate from checked mode.
- Truncation may discard a partly generated tool call. Never execute partial
  calls or tool-shaped prose; request a smaller valid action through the protocol.
- Error hints must support reread/retry without weakening exact matching,
  disclosing private paths or granting unauthorized effects.
- Changing pure transitions affects replay. Version semantics explicitly and
  preserve existing captures, request fingerprints and denial interpretation.
- One accepted workload is narrow evidence. Freeze prompts before scoring,
  retain unsuccessful attempts and report operator effort honestly.

## 5. Recommended Approach
Add a finite automatic recovery/completion-review policy grounded in the
current request and recorded tool outcomes. Cover fake tool-result prose,
premature completion, empty/truncated replies and recoverable edit/tool errors
using generic hints; preserve authority and all existing budgets. Keep failure
terminal when retries or budgets are exhausted. Pair new transition semantics
with old-capture replay compatibility. Run the frozen workload in a fresh
workspace, using only its initial request and natural follow-up. Observe the
browser outcome; failed attempts remain in the denominator. Perform formal
regressions/critic only after a zero-correction live success. Core 4 applies
only to freeform work with compiled effectful grants: two repair nudges plus
one separate completion review, bounded receipt summaries and deterministic
optional workflow metadata on model_finished. Preserve old core-3/adapter-3
replay. Tools 4 reports nonzero command errors with stderr priority and useful
exact-edit reread hints; adapter 4 clarifies preview external-JS/CSP needs.
Fresh local-profile budgets become 16k context/2,400 output/20 turns/30 tools/
240 seconds without rewriting existing profiles.
Alternative: more operator-authored patches would repeat sprint 13's usability
failure rather than improve the product.

## Artifacts
- [Draft build plan](../sprint-plans/build-plan.md)
- [Deferred official test plan](../sprint-plans/test-plan.md)
- [Draft live workload card](live-workload.md)

## Orchestration
The user explicitly authorizes progress and retains live-first testing order.
Codex has no EnterPlanMode/ExitPlanMode or TaskCreate tools in this host; use
unlocked Book drafts and independent critic review without another permission
question. Final recovery details and workload limits must be agreed by the
implementation owner before locking. No official tests are run during drafting.
