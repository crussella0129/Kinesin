# Sprint 16 Build Plan — proposal

Awaiting user approval. This host exposes no EnterPlanMode/ExitPlanMode tools;
these are reviewable scratch proposals and implementation remains unchanged.
Canonical plans stay empty until approval, then receive the required final
critic and helper lock. No Sprint 16 model request has been made.

## Intents
- [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md) — active;
  AC1–AC6. Preserve truthful evidence, bounded execution and conditional grounding.
- [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) — active;
  AC2–AC4. Make the assistant require less babysitting than writing the code;
  preserve the unchanged full workload and live-first verification order.

## Schema Tree
- Qualify discovery and useful repair before claiming autonomous app work
  - T-125: freeze and run a bounded grounding decision
  - T-126: conditionally supply one real root listing in the harness
  - T-127: qualify normal requests on original and held-out repairs
  - T-128: operate the unchanged storefront and same-session follow-up
  - T-129: verify useful implementation and reconcile retained backlog

## Fixed Live Budget and Decision Tree

There are at most **four diagnostic top-level submissions total**, shared across
T-125/T-127. Submitted failures, cancellations and transport failures consume
their reserved slot. No replacement, budget reset or profile sweep is allowed.
Freeze all branches' fixtures/prompts/oracles before the first call. Source may
change only for conditional T-126; refreeze its binary/source and first wire
before subsequent qualification. All runs use fresh independent app copies and
empty sessions. Keep the S15 Qwen2.5-Coder-7B Q4_K_M/b6500/seed-0/temp-0 profile,
explicit ordered adapter 6, and original 16,384 context/2,400 output/20 model
turns/30 tools/240 seconds/32,768 history/8,192 tool-result limits, two repair
nudges and one completion review. Same runtime ownership/alias and grants.

1. **A, slot 1:** exact S15 unaided repair prompt/seed, without browser feedback,
   filename additions or grounding. If it passes, skip B/C and T-126; slot 2
   becomes an unaided held-out repair under T-127. Two passes unlock T-128.
2. **B, slot 2 only after A fails:** same base task and seed, appending a frozen
   authentic bounded root filename observation. Include all collected names,
   whole-name/truncation/error semantics, and no source/diagnosis/tool imperative.
   This is upfront operator-assisted diagnosis. If B passes, implement T-126;
   slots 3/4 then requalify the actual harness with normal base prompts, original
   fixture then held-out fixture. Both must pass. Stop early if slot 3 fails.
3. **C, slot 3 only after B fails:** same B input plus complete unmodified
   observed source for the three small seed files, with hashes/provenance and no
   diagnosis. If it fails, stop. If it passes, slot 4 uses the same source-
   assistance policy on the held-out fixture. Stop with findings either way;
   this branch cannot unlock T-126, T-127 or full workload delivery.

Source assistance must fit unchanged limits in full; no selective excerpt of the
bug. If preparation cannot provide the frozen observation, stop and record it;
do not invent evidence or increase budgets. Slot 4's source-assisted outcome
measures limited transfer, not autonomous discovery. A/B/C contrasts include
framing/token/salience effects; no unique-cause or reliability claim is justified.

The held-out fixture is frozen before A: a small app in different relative files
with a seeded removal defect, where removing a row containing two units must
remove both units and their full value. Catalog/search and unrelated cart actions
are controls. Independently reproduce both seed defects and controls in a browser
before dispatch; retain exact fixture bytes, hashes, expected arithmetic and
observer identity. No held-out files, prompts or solutions enter earlier grants.

The original fixture keeps S15's independently scored 2+5=7 and then +2=9
cart totals, item counts and search/price controls. Every pass requires a real
returned preview and browser operation; bytes or a model claim alone cannot pass.

### T-125: Freeze and diagnose workspace discovery versus source use
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** docs/sprints/s16/sprint-research/diagnostic-card.md; bounded
  sprint-tests/diagnostics artifacts; ignored target/s16-live preparation glue
- **Depends on:** approved plan; existing S15 implementation compiles; no new
  feature implementation is required for A/B/C
- **Acceptance criterion:** AC1, AC4, AC5; AC6 motivates the conditional decision
- **Success criterion (EARS):**
  - **E1 WHEN** a diagnostic is submitted, **THEN** its record **SHALL** freeze
    accepted prompt bytes/newlines, fixture/control hashes, model/runtime/profile,
    source/binary, actual first request identity, prediction and falsifier before
    inference, with a monotonically consumed shared four-slot ledger.
  - **E2 WHEN** A/B/C settles, **THEN** the decision **SHALL** follow the tree
    above, score real operations plus defect/control behavior, retain every
    failure and unknown, and distinguish manual observation assistance from a
    normal user request. Unsupported answers or missing preview **SHALL NOT** pass.
- **Notes:** Preserve authentic directory/source observations outside app grants.
  Use a no-inference capability reader or direct fixture observation with explicit
  observer identity; never counterfeit a harness tool event. Source assistance
  is frozen data, not a replacement implementation. No code changes to make the
  diagnostic result positive. This task's evidence completion can coexist with
  a failed sprint; it does not complete prior product tasks.

### T-126: Conditionally add one bounded harness-origin directory observation
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** src/config.rs; src/policy.rs; src/core.rs; src/runner.rs;
  src/replay.rs; src/storage.rs; src/effects.rs; src/session.rs;
  src/cli.rs; src/cli/session.rs; docs/cli.md; src/tools.rs only for executor reuse
- **Depends on:** A fails and B passes under T-125. Otherwise leave this task
  explicitly unexecuted; no guessed source-context mechanism substitutes for it.
- **Acceptance criterion:** AC2, AC3, AC5, AC6; INT-0032 AC2
- **Success criterion (EARS):**
  - **E1 WHEN** trusted workspace config explicitly selects `workspace_grounding =
    "list_root"`, **THEN** admission **SHALL** freeze that selection and require
    eligible local freeform ordered compiled-tool execution with an effective
    list_files grant and no MCP tools. Ineligible selection **SHALL** reject;
    absent/none **SHALL** preserve existing bytes and behavior with no read.
  - **E2 WHEN** an admitted grounded run starts, **THEN** it **SHALL** execute
    exactly one actual list_files(".") before its first model request through
    the existing capability and bounded executor; charge one existing tool call,
    actual deadline and result/history/request bytes; cap the result at
    min(2,048, configured max_tool_result_bytes); retain worker ownership through
    settlement and never dispatch a model after cancellation/deadline/journal stop.
  - **E3 WHEN** listing settles, **THEN** the model context **SHALL** include only
    the actual bounded observation with explicit harness origin, root scope,
    untrusted-data and incomplete/error labels. The user request remains intact.
    A normal listing error may be observed by the model; it **SHALL NOT** mean
    empty workspace. Actual context overflow **SHALL** stop before model dispatch
    with existing context-limit semantics, without silently changing limits.
  - **E4 WHEN** that operation is journaled, summarized, carried into later
    reference or replayed, **THEN** origin and real outcomes **SHALL** remain
    distinct from model initiative and behavioral progress, bounded by existing
    fact/session limits; metadata capture **SHALL NOT** retain listing names/body,
    private replay **SHALL** reproduce recorded observations without live I/O,
    and all historical captures/default encodings **SHALL** retain their meaning.
- **Notes:** This is a single optional initialization effect, not a mandatory
  workflow graph, model-generated proposal, shell command, automatic source read
  or forced tool branch. Keep answer choice, instructions, adapter 6 wire ordering
  and checked acceptance unchanged. Do not put filesystem I/O into CLI preflight.
  Recheck actual framed context after observation; preflight cannot know filenames.
  No retry or wider scan is authorized. Root listing does not assert completeness
  of a repository or ownership/security beyond the existing filesystem capability.

Use distinct harness-origin planned/finished events and effect identity, not
`model_turns - 1` or a synthetic ModelReply. Derive accounting in the core and
replay consistently; include actual dispatch/control/result fingerprints. The
operation-fact reducer must recognize those events, preserve harness origin even
when bounded records are omitted and never label them model-selected actions.

Reserve **capture 6 / core 9 / adapter 6 / tools 5** for grounded ordered runs.
Use typed origin-aware effect summary/reference version 2 only when needed;
historical version 1 shapes retain exact serialization. A later input containing
version-2 references also needs capture 6/core 9, including when grounding is
disabled; native adapter 4 or ordered adapter 6 remains frozen by its normal
protocol selection. New grounding is itself allowed only on the eligible ordered
path above. Reject new fields/shapes in capture 3/4/5 and unsupported tuples;
retain all old admitted tuples, including 4/8/5/5. New capture/core does not change
tools/checker/parser/output-contract versions. Validate any mixed version-1/2
session history explicitly; whole-record/reference eviction and /new remain intact.

### T-127: Qualify normal requests on two distinct repairs
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md);
  [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- **Touches:** docs/sprints/s16/sprint-tests/diagnostics; diagnostic decision card;
  disposable app/control directories
- **Depends on:** A passes (no T-126), or A fails/B passes/T-126 compiles
- **Acceptance criterion:** INT-0033 AC4–AC6; INT-0032 AC3, AC4
- **Success criterion (EARS):**
  - **E1 WHEN** the selected normal-request path is scored, **THEN** original
    repair and distinct held-out repair **SHALL** both pass actual preview/browser
    defect and control criteria with zero operator-supplied context, corrections,
    patches, tool forcing or resets; all requests consume the original four-slot
    allocation. Failure on either **SHALL** block T-128 and T-129.
  - **E2 WHEN** outcomes are compared, **THEN** the report **SHALL** distinguish
    automatic initial observations, model-selected inspection and edits, behavior
    observations, failures and unknowns; retain costs/interventions and artifact
    identities, without calling a single successful listing useful progress.
- **Notes:** In branch A, A plus the unaided held-out slot 2 supply the two
  observations. In branch B, fresh normal-prompt slots 3/4 must both pass after
  the implementation; manually assisted B cannot replace either. No second
  implementation revision/replacement qualification slot after failure.

### T-128: Deliver the unchanged storefront and natural follow-up
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md);
  [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** docs/sprints/s16/sprint-research/live-workload.md;
  sprint-tests/e2e-tests.md; bounded retained attempts; disposable app workspace
- **Depends on:** T-127 passes both normal-request repairs
- **Acceptance criterion:** INT-0032 AC3, AC4; INT-0033 AC4, AC5
- **Success criterion (EARS):**
  - **E1 WHEN** up to two fresh full attempts are operated, **THEN** an accepted
    attempt **SHALL** pass the exact S14 Paper Harbor initial task and natural
    same-session Clear Cart/item-count follow-up, original search/filter/cart/
    totals/persistence/checkout validation and preview-lifetime criteria, with
    zero corrective messages, code patches, tool-forcing requests or resets.
  - **E2 WHEN** a full attempt fails, **THEN** it **SHALL** remain recorded; a
    second attempt **SHALL** require a stated in-scope generic harness fix to a
    recorded defect, unchanged prompts/profile/criteria/caps and a fresh app.
    Exhaustion or an out-of-scope cause **SHALL** stop with failed/inconclusive
    usefulness; no passing initial gate means no follow-up submission.
- **Notes:** Copy the exact prompt bytes and full criteria from the retained
  [S15 workload](../../s15/sprint-research/live-workload.md), which preserves S14.
  Record actual CLI delimiter bytes. Keep the selected diagnostic path and caps;
  automatic listing is charged within the original 30-tool limit, not added to it.
  No second attempt is needed after a pass. Initial empty app contains no seed,
  hint, evidence file or prepared storefront. Do not inject observed failures.

### T-129: Verify the useful implementation and resolve retained work
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md);
  [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- **Touches:** focused Rust unit/integration tests; docs/sprints/s16/sprint-tests;
  docs/work; intent evidence; CLI documentation
- **Depends on:** T-128 full live confidence pass
- **Acceptance criterion:** INT-0033 AC1–AC6; INT-0032 AC2, AC4
- **Success criterion (EARS):**
  - **E1 WHEN** the unchanged full workload passes, **THEN** focused official
    unit/integration checks, formatting, Clippy and independent final review
    **SHALL** cover the final observed-context path if used and retained S14/S15
    protocol/recovery/facts/session/replay/authority boundaries. Material repairs
    **SHALL** receive affected live re-verification without erasing past failures.
  - **E2 WHEN** task/intent completion is recorded, **THEN** prior failed tasks
    **SHALL** receive explicit retained/replaced/deferred disposition, and no
    earlier failure, skipped conditional task or unperformed check **SHALL** be
    retroactively marked successful. Unobserved general reliability and human
    productivity **SHALL NOT** be claimed.

## Deferrals and Approval Boundary

Automatic source retrieval, recursive exploration, generic browser tools,
requirement/state graphs, new semantic completion states, model/token sweeps,
persistent sessions and OS sandbox changes are deferred. Source-assisted wins
require a new retrieval design, not silently selecting files in T-126.
Grounding/default promotion and installing the candidate are not authorized by
this proposal. On failure, preserve source as unverified and close honestly.

Only a clean reviewed and user-approved plan can authorize these new four slots
and conditional implementation. Keep Sprint 16 local while PR #15 awaits separate
human disposition. After approval, write canonical plans, keep active intents
active, run the mandatory canonical critic and finalize-plan helper, then queue
the tasks in order. Skipped conditional work must be explicitly rescheduled or
deferred before closeout, never fabricated as a completed implementation.
