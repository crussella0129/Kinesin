# Sprint 14 Failure Report

## Affected Intents

- [INT-0032](../../intents/INT-0032-low-intervention-local-workflows.md) remains
  active and unrealized. AC3's usable fresh app and natural follow-up failed;
  AC4's live-first official verification condition was never reached. AC1/AC2
  implementation exists but is not formally verified. AC2a defaults/guidance
  compiled, without evidence that they delivered the usability outcome.

## What Failed

All eight scored fresh attempts failed the independent live gate. None supplied
a usable initial app and the required follow-up without corrective steering.
The [full scorecard](sprint-tests/e2e-tests.md) retains every attempt, exact
prompts, source/model/profile identities, effects, failures and effort.

| Attempt | Actual outcome |
| --- | --- |
| 1 | Six products visible; cart inert, search absent; follow-up claimed unwritten changes. |
| 2 | Scaffold only; empty replies exhausted recovery; no preview. |
| 3 | Preview existed, products did not render; a run/preview milestone mismatch caused a false stop. |
| 4 | Code in prose, no writes; both preview calls failed. |
| 5 | Newer model wrote files and showed products; cart inert, search absent; real CSP warning dismissed. |
| 6 | Reasoning/output-budget comparison rendered no products; claimed external-file repairs that never occurred. |
| 7 | Structured protocol, original model: two valid answers, zero actions. |
| 8 | Structured protocol, newer model: two valid answers, zero actions. |

Diagnostic continuations also failed to repair retained apps. They were assisted
investigations, not accepted workloads. Attempts 2–8 include a terminal CRLF in
the actual submission; [raw and normalized identities](sprint-tests/attempts/prompt-byte-audit.json)
are distinguished rather than represented as byte-identical prompt files.

## Root Cause

Established design defects and observations are separate from causal hypotheses:

1. **No independent completion condition for freeform development.** A model
   answer is an unchecked candidate. The additional completion review asks the
   same model to assess itself; after that review, promises and unsupported
   claims can still end the run. A loop termination event is not delivered work.
2. **The available feedback describes operations, not requested behavior.**
   Receipt summaries prove writes/startup occurred. The index CSP scan does not
   execute JavaScript or exercise search/cart/checkout. There is no requirement
   ledger with current, artifact-bound behavioral observations or invalidation.
3. **Feedback availability and repair ability are distinct.** Real CSP warnings
   and supplied browser facts did not cause reliable edits. Missing browser
   tooling alone cannot explain zero-action initial runs or prove that adding
   an observer will make this model capable of repair.
4. **The attempted state machine used the wrong transition witnesses.**
   Mandatory operation-kind milestones false-stopped legitimate activity while
   successful operations could still leave the app broken. Strict response
   syntax also failed to establish action selection or task competence.
5. **The experiment design did not isolate all causal variables.** Harness
   revisions, model weights, sampling/mode and output allowance changed across
   the sequence. Matched comparisons are informative but single stochastic
   samples do not establish model rankings, reliability or a universal cause.

Still unresolved: model/runtime/tool-template compatibility, constrained-schema
branch bias, task decomposition and usable output headroom, whether authentic
feedback changes repair behavior, and session carry-forward of claims versus
actual effects. These require bounded discriminating probes. They must not be
declared proven model incapacity or solved by another generic prompt change.

## Required Re-architecture

Sprint 15 must research action initiation, repair competence, truthful stopping
and evidence continuity separately before selecting one intervention. Preserve
the original objective: a harness that costs less operator effort than writing
the code directly. Do not lower the storefront acceptance gate.

Freeze small live diagnostic contrasts, measure real effects and retain negative
results. Any future supervisor must separate attempted action, effect receipt,
behavior observation and completion; bind observations to current artifacts;
invalidate stale results; share parent permissions/budgets; and stop boundedly
on no demonstrated progress. Prove an observation actually helps before building
a broad browser subsystem. A model-authored check is not independent acceptance.

The selected mitigation needs a stated causal prediction and falsifier. Live
operation/repair remains first; official unit/integration verification follows
useful live behavior. Research may conclude that the selected configuration
cannot meet the goal within the agreed limits.

## Evidence

- [Eight failed runs and independent observations](sprint-tests/e2e-tests.md)
- [Experiment refinements and native-protocol rollback](sprint-research/implementation-adjustment.md)
- [Bounded Animus reference study](sprint-research/reference-loop-principles.md)
- [Failure assessment, not a passing test report](sprint-tests/test-report.md)
- [Independent design critiques](sprint-plans/critique.md)

## State at Failure

- T-115: incomplete; unverified source retained as a reversible research checkpoint.
- T-116: attempted eight times; acceptance failed; not a completed delivery task.
- T-117: not started because its live prerequisite failed.
- T-115–T-117 return to backlog for explicit disposition in Sprint 15 planning.
- T-118 is an owner-requested post-plan closure task: archive the failed
  experiments, findings and source disposition. Its completion certifies this
  assessment only, not any of T-115–T-117's product criteria.
- New runs were restored to core 7 / adapter 4 / tools 5. Experimental core 8 /
  adapter 5 captures remain supported for replay. No new version claims a pass.
- Formatting, compilation and whitespace inspection passed before closure;
  official Sprint 14 unit/integration tests and Clippy were not run. No passing
  test critique or release claim is implied. The installed assistant was not
  replaced with this unverified implementation.
- All scored sessions exited. Generated labs/model files remain ignored under
  target; only bounded provenance and public generated fixtures are archived.

The user explicitly requested failed closure and a new research/plan cycle on
2026-09-20. This report is the failure handoff to that cycle.

Independent closure review by live_sprint_setup accepted this assessment: no
product task is falsely completed, missing feedback is not asserted as a sole
cause, and unperformed official verification remains explicit. Follow-on schema
ordering research must distinguish a confirmed compatibility mismatch from its
unproven causal contribution to the answer-only runs.
