# Sprint 14 implementation adjustment after failed live attempt 1

The first frozen two-request attempt failed. Initial run
`97241cfd-8824-424b-95ee-9b2797a958ec` wrote two files and started a real preview,
but browser operation found missing search/filter controls and inert Add to Cart.
Its completion review is `model_finished` sequence 13, `data.workflow.reason =
completion_review`; sequence 15 claimed the requested features without any
intervening tool observation. Follow-up run
`a96761d6-565a-48e3-9171-1301c45f9b34` only called `start_preview` for `.` and
received `preview_already_running`. It performed no file edits, yet its final
answer claimed Clear Cart and item-count changes. These remain failed outcomes.

The initial run used three model turns, four tool calls and 23.138 seconds;
the follow-up used three model turns, one tool call and 16.081 seconds. Each
used one completion review and zero repair nudges. Both submitted prompts match
the frozen card. Zero corrective operator prompts does not constitute success
when the requested application behavior is absent.

## Agreed bounded refinement

Within T-115, replace immediate freeform execution for eligible new-version
runs with a generic planning-and-execution sequence:

1. Request a constrained JSON plan containing one to six small milestones.
   Each has bounded task/verification text and a kind: `read`, `change`, `run`,
   `preview` or `answer`. Planning has no tool dispatch and must cover the
   original user request. Preserve that request while executing the plan.
2. Prompt the model to perform each milestone through its already granted
   tools. To advance a non-answer milestone, require an actual successful
   operation of its corresponding kind during that milestone. Prose, a failed
   operation, or a receipt from an earlier milestone cannot be its witness.
3. A missing witness consumes the same shared two-nudge repair allowance;
   exhaustion stops honestly. `answer` steps need no mutation. The planner
   cannot grant tools or authorize effects that the user/workspace did not.
4. Keep the separate completion review at most once, after the last milestone.
   All planning, execution, repair and review calls consume the unchanged
   20-turn / 30-tool / 240-second / 2,400-output ceilings and existing byte limits.
   Bound the plan and preserve the existing 12-entry / 4,096-byte receipt summary.
5. Record deterministic planning, stage-advancement and recovery metadata so
   inference time and internal intervention remain measurable. Use core 5 and
   retain exact core-4 behavior for the already recorded attempt, alongside
   earlier replay compatibility. Never execute partial plans or tool-shaped prose.

This is task decomposition and operation witnessing, not independent verification.
A read does not prove understanding; a write does not prove valid code; a command
may check the wrong thing; and a planner may omit or misclassify requirements.
The frozen browser criteria remain the success gate, and model-chosen verification
cannot upgrade unchecked acceptance. A failed attempt cannot be repaired through
operator coaching and relabeled as a pass.

## Why this step precedes adding browser automation to Kinesin

The observed failures include omitted features and outputting changes as prose,
before runtime diagnostics alone could establish correctness. Sprint 13 showed
that decomposition helped the same small model perform real file operations.
Automating generic, bounded decomposition directly targets that manual work with
existing capabilities. A browser-observation capability could later supply useful
runtime feedback, but is a broader integration and would not itself establish
requirement completeness. Continue independent browser operation now; reconsider
additional capability only if the next retained attempt demonstrates that need.

## Review and verification impact

The implementation owner agreed this refinement after the failed attempt; the
intent consequences record it explicitly. Locked plans are not silently rewritten.
The existing T-115/T-117 verification categories must include bounded plan parsing,
stage-specific witnesses, missing-witness exhaustion, answer-only behavior,
unchanged grants/budgets and core-4/core-5 replay. The next attempt uses a fresh
empty workspace and separately frozen source/profile identity, with the same two
natural prompts and independent criteria. Official tests still wait for a live pass.

## Refinement after failed attempt 2

Run `de003881-f1e5-49bd-9187-8d9faae9e6dd` stopped honestly without a candidate
after two repair nudges. The model proposed file-creation placeholders instead of
feature-complete milestones, guessed `run` for a preview operation, and parroted
long stage instructions. It first consumed a repair by repeating the directory
step without acting; later two actual empty responses exhausted recovery around
a successful JS write. No fabricated tool-response text caused this stop. The
[retained trace](../sprint-tests/attempts/attempt-2-runs.json) records each decision;
the incomplete application remains a failed attempt with no preview.

The implementation owner agreed a core-6 refinement while preserving exact
core-4/core-5 behavior for earlier captures:

- Recommend two to six compact, feature/outcome milestones for larger work that cover
  the original requirements. Group setup with useful work; do not spend stages
  merely creating empty files/folders or reporting the final answer.
  The schema/parser still allows one to six steps for a simple single operation.
- Include the available granted tool names in planning reference data so the
  planner can distinguish `preview` from `run`, without granting new authority.
  Use concise phase-specific execution instructions to reduce instruction echo.
- An actual empty provider response after a successful matching operation in
  the current active stage may advance that stage on its recorded operation
  witness. Record an explicit completion basis. It is never an answer candidate,
  never asserts verification, and cannot satisfy final completion review. An
  empty reply without a witness still consumes the shared two-repair allowance.
- Preserve all existing run/byte/tool ceilings, the one final completion review,
  failed attempts and the independent two-prompt browser gate. The next attempt
  starts empty with the same profile and natural prompts, newly frozen source
  identity, and no operator app patches.

Operation witnessing still cannot establish that a requirement was implemented
correctly. The unchanged browser gate is essential, and official verification
must cover the new empty-with-witness transition and legacy replay after a live
pass. This adjustment does not claim the incomplete attempt was successful.

## Refinement after failed attempt 3 and diagnostic operation

Attempt 3 started a real preview but exhausted repair because the model had
classified that stage as `run`. Its browser outcome regressed to an empty
catalog, with inline JavaScript/handlers blocked by CSP and unsupported writes
to static JSON endpoints. Actual preview startup therefore established neither
the stage's assumed command effect nor the requested application behavior.
Additional diagnostic operation also failed: one staged run rejected inspection
already performed during an earlier stage; a separate core-4 probe encountered
file-path errors. Those probes demonstrated no functional progress and do not
replace a fresh scored attempt or erase the three retained failures.

The implementation owner proposed, and an independent bounded design review
accepted, a smaller core-7 refinement within T-115:

- Remove mandatory model-generated planning and per-stage witness gates for new
  runs. Use short generic work instructions, the existing two repair nudges and
  one final completion review under unchanged run/byte/tool budgets. Preserve
  exact core-4/core-5/core-6 behavior for old captures.
- Generic instructions retain subdirectory context and require observing a
  prerequisite tool result before a dependent action. Tools 5 adds a bounded
  `not_found` hint in `src/tools.rs`: include the file's subdirectory and, if
  listing is granted, inspect the parent. This addresses the unscored core-4
  probe's wrong-root file batch without performing a listing or granting one.
  Current identity is core 7 / tools 5 / adapter 4; historical core-6/tools-4
  behavior and earlier supported versions remain available for replay.
- Extend T-115's implementation touches to `src/preview.rs`. Its already bounded
  index read can return actionable compatibility warnings for executable inline
  JavaScript, inline handlers/styles and the static server's GET/HEAD-only
  capability. Preview startup with warnings still returns a successful real URL;
  warnings are feedback, not another mandatory stage or failure gate.
- Repeating `start_preview` for the existing directory rescans the current
  bounded index so a repair can receive fresh feedback. Retain output limits,
  owned worker/deadline/cancellation handling and the existing server lifetime.
  Diagnostics do not execute app code, scan unrelated files, weaken CSP or
  expand tool grants.
- State the observation's scope: an index compatibility scan cannot prove
  JavaScript execution or application correctness, and absence of warnings is
  not a behavioral pass. The independently observed two-prompt browser gate,
  profile ceilings and prohibition on operator app patches remain unchanged.
  Core-7 completion review refers to returned preview warnings and rechecking
  the same preview path after repairs, without promoting that review to proof.

This applies the [reference study](reference-loop-principles.md)'s principle of
using observer data to select the next action, rather than confusing a planning
phase or an operation receipt with product progress. Relaxing individual stage
aliases or permitting selected old receipts would retain a brittle gate without
addressing the missing behavior. Richer app/runtime observation remains a future
option if the next retained live result shows that the smaller feedback is
insufficient; no general semantic progress oracle is claimed here.

Locked plans remain unchanged. After live confidence, T-117's existing focused
verification categories must cover core-7 admission/recovery/review limits,
legacy replay and the preview warning/refresh boundaries. No task completion,
test report or successful gate is recorded by this refinement.

## Model-profile comparison after failed attempt 4

All four scored attempts with the same 7B baseline model profile failed, across
the recorded harness revisions. Attempt 4 produced no application files. A
separate first-action JSON diagnostic only proposed directory creation; no
proposed action was executed. That probe is neither an official test nor useful
product delivery, and it does not upgrade any scored attempt.

Attempt 5 changes the model profile while retaining attempt 4's frozen core-7
source/binary, adapter, natural prompts, ceilings and independent browser gate.
The download target is official Qwen3-8B Q4_K_M (5.03 GB), model commit
`6a569868d07d3bd59e8b97fb001bf8c0b254bb20`, expected GGUF SHA-256
`d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785`.
Use temperature 0.7 and a generic `/no_think` profile-instruction suffix, guided
by the [official model card](https://huggingface.co/Qwen/Qwen3-8B-GGUF).
Record the complete effective sampling/profile settings and verify the downloaded
artifact before a separate pre-dispatch freeze; the card's other recommendations
are not silently claimed as configured.

This is a model-profile comparison, including sampling/mode differences, not a
causal demonstration of harness improvement or a broad model ranking. It keeps
the same two user prompts in a new empty app workspace, introduces no app-specific
coaching and changes no adapter code. Preserve all baseline failures and the
unexecuted diagnostic separately. No task completion or official verification
is claimed before a successful live gate.

## Attempt 6: reasoning-mode and output-budget comparison

Attempt 5's non-thinking profile failed independent browser inspection: Add to
Cart was inert and search was absent. Attempt 6 retains the same core-7 harness
and Qwen3-8B model weights, removes the generic `/no_think` suffix to allow
reasoning, changes temperature to 0.6 and increases per-response output from
2,400 to 4,096 tokens so reasoning can leave room for an action. All other
resource caps, the exact two user prompts, required features and zero-correction
operator rubric remain unchanged. The evidence owner must freeze this distinct
profile and source/binary identity before launch in a fresh empty app workspace.

This is an explicit profile/output-budget comparison, not a directly comparable
harness-only effect. It does not alter production defaults, the original frozen
baseline card, AC3's behavior/intervention criteria or the retained attempt-5
failure. No new harness scope, task completion or official tests are introduced;
official verification still follows a successful live gate.

## Attempt 7: bounded structured single-action protocol

Six scored attempts failed. Attempt 6 wrote only index.html and then claimed
unperformed external-file repairs; two generation-length replies had consumed
8,192 output tokens before its first actual action. Stronger receipt-based
stopping would make failure clearer but would not implement the app. Browser
facts also produced no repairs in the retained diagnostic runs. The earlier
structured probe demonstrated only a directory-creation proposal, not execution
or app competence.

The implementation owner accepted one independently reviewed T-115 refinement:
core 8 / adapter 5, with tools 5 unchanged. Eligible runs are freeform with only
compiled tools and at least one effectful grant. Each model response uses an
explicit constrained schema for exactly one declared tool action with its exact
argument shape, or an answer. Checked, read-only and MCP-only/mixed runs retain
native behavior. No mandatory plan or stage gates are reintroduced.

Only complete valid JSON under this declared protocol may translate into an
existing typed ToolCall. Unknown fields, malformed/partial responses and arbitrary
prose dispatch nothing. Parsing grants no authority: normal permission, path,
budget, journal and effect-arbitration checks still apply. Invalid-protocol
recovery shares the existing two-nudge ceiling; the one completion review and
other core-7 limits remain. Streaming must withhold raw action JSON and provisional
tool arguments from user-facing text. Preserve all previously supported replay
versions, their instructions/request fingerprints and failed attempt records.

Pre-dispatch inspection found that the action protocol adds instructions and
schemas to the actual model messages. Adapter 5 therefore also checks the
serialized transformed message array against the frozen history-byte cap before
HTTP dispatch. Runner and replay supply the same cap and preparation path.
Ordinary logical-history compaction remains; if the transformed messages still
exceed the cap, the run stops with `history_bytes_limit` instead of increasing
the allowance or adding recovery attempts. Native requests and older adapter
versions retain their previous request bytes.

Attempt 7 returns to the original Qwen2.5 7B baseline profile, temperature 0,
2,400 output tokens, exact user prompts and original ceilings. Freeze the new
source/binary and effective profile before a fresh empty-workspace dispatch.
This compares the new harness strategy with attempt 4 under restored baseline
model settings; one run cannot establish general causality or reliability.
Schema validity, tool counts and truthful stopping are not functional success.
Only the unchanged independent browser rubric can open the live confidence gate;
official checks and task-completion claims remain deferred. Locked plans stay
unchanged, with this explicit refinement carrying scope/provenance.

## Attempt 8: counterpart profile on the same structured protocol

Attempt 7 failed: two answers, zero tool calls, zero files and no preview after
the one completion review; no follow-up was sent. Schema-valid answers did not
produce useful actions on the baseline Qwen2.5 profile. Retain that failure.

One counterpart comparison keeps the same core-8/adapter-5/tools-5 source and
binary, returning to attempt 5's Qwen3-8B non-thinking profile: temperature 0.7,
generic `/no_think` and 2,400 output tokens. All other original ceilings, exact
user prompts and independent browser/intervention criteria remain binding.
Freeze the complete effective profile and unchanged implementation identities
before dispatch in a fresh empty app. This compares the protocol strategy with
attempt 5 under matching model settings; it cannot establish general reliability
or an isolated effect from one stochastic outcome.

No production defaults, gates, source changes or official tests are introduced.
If this counterpart also fails, the next design step must address actual
observation and repair strategy rather than continue blind prompt variations.

## Rejected structured experiment and default rollback

Attempt 8 also failed: run `77579e77-d3d7-4394-b783-ffcfdec09a81` produced two
schema-valid answers, zero tools, zero files and no preview after its completion
review. No follow-up was sent. With the same model/profile, native attempt 5 had
written files and displayed products, although cart behavior still failed. The
structured experiment therefore showed an observed capability regression in
these runs, not progress toward the usable workflow. Preserve both failures.

Independent review accepts restoring the exact prior new-run defaults:
core 7 / adapter 4 / tools 5, with runner structured actions disabled. No new
semantic version is needed for returning to that existing version set. Keep
experimental core-8/adapter-5 captures admitted explicitly and reconstruct their
structured requests only for core 8, retaining the adapter helpers, strict
decoder and encoded-message cap for that historical path. Older replay behavior
and all effect authority remain unchanged. This is a rejected experiment and
rollback decision, not a claim that core 7 satisfies the user's goal.

The usability gate remains unmet and T-115–T-117 remain queued/incomplete; no
official tests or task-completion evidence follow from this rollback. The next
design obligation is to make actual independent observations drive bounded
repair and completion decisions. Further blind prompt/profile variants are not
the next strategy. Any design must preserve the independent outcome gate, finite
budgets and normal permission checks; no new observation capability is claimed
here. Locked plans remain unchanged.

The rollback is implemented: new-run initialization selects core 7, the default
model builder selects adapter 4, and live runner options disable structured
actions. Replay explicitly admits the experimental 8/5/5 set and enables its
structured request shape only for core 8. Independent source review found no
remaining version/eligibility coupling. `cargo fmt --all`,
`cargo build --locked --bin kinesin` (10.67 seconds) and `git diff --check`
completed successfully after the rollback. These are implementation checks,
not a live usability pass or the deferred official unit/integration verification.
Both comparison sessions exited normally; no model server remained running.
