# Sprint 15 Build Plan — proposal awaiting approval

This is a reviewable scratch proposal, not the canonical or finalized build plan.
No Sprint 15 implementation is authorized by a lock on this file. The canonical
build-plan.md and test-plan.md remain empty until plan approval. This host has no
EnterPlanMode/ExitPlanMode tools; implementation source remains unchanged during
planning, and explicit user approval is retained as the equivalent boundary.

## Intents
- [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md) — proposed;
  AC1–AC5. Transition to planned after approval, before the required final critic.
- [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) — active;
  AC2 compatibility/authority, AC3 unchanged live usefulness, AC4 retained effort
  and post-live verification. Existing AC1/AC2a semantics are preserved, not
  silently replaced by a new recovery controller.

## Schema Tree
- Less babysitting, with independently demonstrated progress
  - T-119: qualify the action wire boundary without changing historical behavior
  - T-120: derive bounded recorded-operation facts from the existing journal
  - T-121: carry partial-run facts into local session reference and honest status
  - T-122: run one six-request causal decision sequence in disposable workspaces
  - T-123: attempt the unchanged storefront/follow-up only after qualification
  - T-124: run focused official verification only after the live confidence gate

## Execution Sequence

### T-119: Add an explicitly selected ordered action protocol
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** src/model.rs; src/config.rs; src/policy.rs; src/runner.rs; src/replay.rs;
  src/core.rs only if required to select existing core-8 semantics; docs/cli.md
- **Depends on:** none
- **Acceptance criterion:** AC1; INT-0032 AC2
- **Success criterion (EARS):**
  - **E1 WHEN** ordered structured mode prepares a request, **THEN** adapter 6
    **SHALL** serialize every action variant with properties ordered
    kind/name/arguments and every answer variant kind/text, retain both legal
    choices, use the unchanged action instruction and argument validation, and
    compute limits/fingerprints from the actual final wire bytes.
  - **E2 WHEN** a normal profile omits the new selector, **THEN** new runs
    **SHALL** retain native core-7/adapter-4/tools-5 semantics; explicitly selected
    ordered structured mode **SHALL** use core 8/adapter 6/tools 5 only for the
    existing eligible freeform compiled-tool work. Default checked, no-tool,
    read-only and MCP-containing runs **SHALL** keep native behavior; explicit
    ineligible opt-ins **SHALL** fail admission. No fallback may
    silently widen authority or change the selected protocol mid-run.
  - **E3 WHEN** a historical capture is prepared/replayed or an invalid action
    arrives, **THEN** the harness **SHALL** preserve historical adapter-5 wire
    bytes/transitions, reject unsupported version combinations, and execute no
    partial, malformed, extra-field or undeclared action.
  - **E4 WHEN** ordered structured generation streams or terminates mid-object,
    **THEN** presentation **SHALL** suppress raw protocol JSON and provisional
    arguments; only a complete validated answer may appear as answer text and
    only a complete valid authorized action may be dispatched.
- **Notes:** Use a small custom serde Serialize view at the final request boundary;
  pass other Value fields through unchanged and reorder only action variant
  properties. Serialize straight to bytes, never back through Value. No global
  preserve_order feature, new prompt, model download, branch forcing, streaming
  JSON leakage or new recovery budget. Use ModelConfig.action_protocol with an
  absent/default native value omitted from serialized authority and explicit
  structured_ordered_v1 selection. Resolve once before admission; policy/core
  initialization, runner options/preparation and journal versions use that same
  selection. Freeze the selector and admitted tuple in captures; replay derives
  behavior from recorded versions, not ambient configuration. The old-order
  diagnostic control is a lab-only route to historical
  adapter 5 under normal grants; do not expose it as a recommended product mode.
  Ordered mode remains opt-in even if this sprint passes.

### T-120: Derive bounded operation facts without inventing outcome judgments
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** new src/effects.rs; src/lib.rs; src/cli.rs bounded event_page reader;
  existing src/storage.rs Event contract read-only unless a narrow accessor is needed
- **Depends on:** none; implementation may parallel T-119 with separate file ownership
- **Acceptance criterion:** AC2
- **Success criterion (EARS):**
  - **E1 WHEN** durable tool_planned/tool_finished events are summarized,
    **THEN** the reducer **SHALL** retain run/effect identity, compiled tool,
    bounded resource descriptor, known dispatch state and recorded result
    classification independently of model prose, including failed/cancelled runs.
    A missing result **SHALL** be unknown; a successful result **SHALL NOT** claim
    changed bytes or correct behavior; errors **SHALL NOT** prove no partial effect.
    Duplicate, conflicting or out-of-order records **SHALL** mark the affected
    result unknown/incomplete rather than replacing an error with success.
  - **E2 WHEN** event or summary limits are reached or identifiers cannot fit,
    **THEN** the reducer **SHALL** deterministically omit whole records and expose
    incompleteness/omission counts within 256 events scanned, 12 retained effect
    records and 4,096 encoded JSON bytes. It **SHALL NOT** include file bodies,
    stdout/stderr, arbitrary tool result text or model-written receipts.
- **Notes:** Read existing owner-scoped durable events in pages of at most 64.
  A short page is not EOF because storage also caps page bytes; require a
  contiguous scan through run_finished to call the event history complete.
  At the scan cap, report a known omission count separately from an unknown
  unscanned tail. Label MCP/unknown-tool activity as unsupported/summarized only
  by safe event identity; count its presence instead of claiming zero activity.
  Derive terminal summaries after the run has actually ended, so a durable tool
  result immediately followed by cancellation is retained. No new RunOutcome,
  database schema, checked acceptance, semantic stalled status, requirement graph
  or per-run completion controller. Missing/truncated event history remains
  explicitly incomplete, never an assertion that no work happened.

### T-121: Retain partial-run facts separately from conversation claims
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** src/session.rs; src/cli/session.rs; src/cli.rs;
  src/cli/presentation.rs; src/policy.rs; src/replay.rs; docs/cli.md
- **Depends on:** T-120; coordinate shared replay/CLI paths after T-119
- **Acceptance criterion:** AC2, AC3; INT-0032 AC2
- **Success criterion (EARS):**
  - **E1 WHEN** a terminal interactive run has recorded tool activity,
    **THEN** the session **SHALL** retain its bounded operation reference and exact
    terminal reason even if its answer is absent or the run failed, while storing
    any assistant answer in a separate claim field. Unknown or incomplete activity
    **SHALL** remain labeled as such and recorded errors as repair-unverified.
  - **E2 WHEN** these references enter a later request or exceed admission limits,
    **THEN** the harness **SHALL** mark them historical reference data with no
    current authority/checked evidence, preserve the existing 16-turn/8-KiB and
    smaller profile-derived limits, omit/shorten deterministically with notice,
    and erase them on /new, /clear or process exit. Metadata-only capture **SHALL
    NOT** persist reference content; paths/grants require fresh validation.
  - **E3 WHEN** a freeform response completes or /status is requested,
    **THEN** the CLI **SHALL** distinguish response completion from unverified
    requested work and preserve exact stopped/failed/cancelled reasons; ordinary
    answers with zero tools **SHALL NOT** be recast as failures. Existing checked
    verdicts and durable lifecycle values **SHALL** retain their meanings.
  - **E4 WHEN** old and new session inputs are captured/replayed,
    **THEN** omitted optional reference fields **SHALL** preserve old encoding,
    the new reference shape **SHALL** be explicitly versioned/frozen/hashed, and
    historical request preparation **SHALL** reproduce without live effects.
- **Notes:** Add an optional typed run_reference to SessionTurn; an empty answer
  is valid only with a valid reference. Use a new capture version for the new
  input shape, admit exact tuples, retain legacy decoders, and omit absent fields.
  Freeze this compatibility matrix (capture/core/adapter/tools; all other version
  fields retain their current values): existing capture-3/4 tuples remain as-is;
  new ordered requests without run references use 4/8/6/5; new native references
  use 5/7/4/5; new ordered references use 5/8/6/5. Capture 3/4 rejects the new
  run_reference shape. The lab-only old-order control uses 4/8/5/5 with empty
  session input; no 5/8/5/5 combination is admitted. No-reference native requests
  still use 4/7/4/5. Record the full tuple for every live and replay case.
  Empty context is a restriction on new lab-control invocations only; do not
  reject otherwise valid historical adapter-5 captures with legacy context.
  Do not append fabricated assistant text to disguise failed effects as answers.
  Shorten ordinary prompt/answer text only. Drop whole typed effect records or
  references when needed; never clip IDs, resource identities or serialized JSON.
  If trimming removes the only reference from an answerless turn, evict the turn
  rather than retaining invalid empty content.
  No persistent session store or automatic resume is introduced.

### T-122: Identify the next failure boundary with a fixed diagnostic budget
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** docs/sprints/s15/sprint-research/diagnostic-card.md;
  docs/sprints/s15/sprint-tests/diagnostics/; ignored target/s15-live/ labs;
  minimal local launch/evidence glue if needed, not a new evaluation framework
- **Depends on:** T-119, T-120, T-121 compiled; card frozen before dispatch
- **Acceptance criterion:** AC1, AC4, AC5
- **Success criterion (EARS):**
  - **E1 WHEN** diagnostics start, **THEN** the operator record **SHALL** freeze
    exact prompts/fixtures, model/runtime hashes, source/binary/wire identities,
    sampling and limits, prediction and falsifier before each pair, and spend at
    most six user-request invocations through the real harness using the decision
    sequence below. Repairs consume the original request caps; aborted/failed
    invocations consume their slot and are retained.
  - **E2 WHEN** a diagnostic finishes, **THEN** its record **SHALL** score actual
    bytes or independently observed behavior, retain operator interventions and
    cost, and classify supported/falsified/inconclusive causal predictions without
    calling JSON validity, tool counts or honest failure task success.
- **Decision sequence:**
  1. No-inference inspection materializes actual adapter-5 and adapter-6 wire
     schemas against b6500 ordering semantics. Calls 1/2 use identical fresh tiny
     create/read/amend fixtures with old versus ordered structured mode; only the
     schema property order differs. Both answer/action branches remain available.
     Inspect the actual first request bodies before dispatch and reject unrelated
     differences; record first action/answer selection separately from eventual
     effects. Later model histories, receipts and request-derived call IDs may
     diverge naturally and must not be described as byte-identical paired inputs.
     Prediction: corrected ordering permits a useful action trajectory where the
     original does not. Neither/both succeeding does not establish that ordering
     caused attempts 7/8; never extrapolate from one pair to general reliability.
  2. If call 2 completes correct effects, calls 3/4 compare native versus ordered
     mode on one held-out tiny task with identical fresh fixtures. Prefer native
     on a tie; ordered is selected for further diagnostics only if it provides a
     concrete behavior advantage across these observations. For this branch,
     select only from successful held-out calls 3/4: native wins ties, ordered is
     selected when only call 4 passes, and neither passing means stop even if
     call 2 passed. If call 2 fails,
     calls 3/4 instead qualify native create and natural same-session amend with
     independently checked bytes. This fallback is qualification, not a paired
     causal comparison. Both native create and same-session amend must pass to
     qualify that fallback; otherwise stop.
  3. Calls 5/6 compare repair of the same single-defect static app on identical
     copies using the qualified path, without versus with an authentic bounded
     browser failure observation bound to file hashes. The base request and all
     caps remain fixed; the observation is the sole added input. Record it as
     diagnostic operator assistance, never a zero-correction acceptance attempt.
     Verify the original defect and an unaffected control interaction afterward.
     If neither arm repairs, stop and record the action/repair limitation. If only
     the feedback arm repairs, record an automatic-observation dependency and
     return to research/plan for that capability; do not silently implement a
     browser subsystem under this plan. If unaided repair succeeds, T-123 may run.
- **Fixed profile:** Qwen2.5-Coder-7B Q4_K_M baseline, temperature 0, b6500 runtime;
  retain all original S14 card ceilings (16,384 context, 2,400 output, 20 turns,
  30 tools, 240 seconds, 32,768 history bytes, 8,192 result bytes, two nudges and
  one review). Freeze effective seed if supported; disclose if not.
  All paired arms use independent empty sessions and identical initial reference
  state as well as fresh file copies; only the native create/amend fallback
  deliberately shares a session. Never carry T-121 reference facts from one
  comparison arm into the next. No model,
  token, temperature or prompt sweep and no reset of the six-call budget after
  a generic product repair. Compile/format/read-only inspections may support
  live work; official test suites stay deferred.

### T-123: Demonstrate useful work on the unchanged full workload
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md);
  [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- **Touches:** docs/sprints/s15/sprint-research/live-workload.md;
  docs/sprints/s15/sprint-tests/e2e-tests.md; retained bounded attempt evidence;
  disposable target/s15-live/ workspaces
- **Depends on:** T-122 qualifying real actions and unaided observed repair
- **Acceptance criterion:** INT-0032 AC3, AC4; INT-0033 AC4, AC5
- **Success criterion (EARS):**
  - **E1 WHEN** at most two fresh full-workload attempts are run, **THEN** the
    accepted attempt **SHALL** satisfy the original S14 initial storefront and
    same-session Clear Cart/item-count follow-up, every original independent
    browser/HTTP criterion, preview lifetime and zero corrective operator prompts,
    code patches, tool-forcing messages or context resets. The follow-up is sent
    only after the initial browser gate passes. All attempts remain in the record.
  - **E2 WHEN** the outcome is compared to retained attempts or a prior revision,
    **THEN** the report **SHALL** bind each observation to a criterion, observer,
    artifact hashes and actual receipt; report lost passes as regressions, gains
    separately, and missing/stale/incomparable evidence as unknown. Unchanged
    behavior requires comparable observations plus proof the attempted action
    ran; changed bytes alone **SHALL NOT** mean forward progress.
- **Notes:** Preserve exact effective prompt text and record CLI newline handling.
  Keep the selected diagnostic profile and original ceilings. An optional second
  full attempt is permitted only after a stated generic harness correction to a
  recorded defect, with new source identity and no criterion/profile changes.
  No mandatory second run after a pass. If either prerequisite is unmet or both
  full attempts fail, stop and record failed/inconclusive usefulness; do not start
  official unit/integration tests or count infrastructure completion as app success.

### T-124: Verify the final useful implementation and resolve the failed backlog
- **Intent:** [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md);
  [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- **Touches:** focused tests beside affected Rust modules and existing integration
  files; docs/sprints/s15/sprint-tests/; docs/work/; intent evidence and CLI docs
- **Depends on:** T-123 successful live confidence gate
- **Acceptance criterion:** INT-0033 AC1–AC5; INT-0032 AC2, AC4
- **Success criterion (EARS):**
  - **E1 WHEN** the full live gate passes, **THEN** focused official unit/integration
    checks, formatting, Clippy and an independent review **SHALL** cover the final
    protocol, facts, session/admission, replay and authority behavior before a
    successful sprint closure. A material fix after the live gate **SHALL** trigger
    re-verification of its affected behavior, without erasing prior attempts.
- **Notes:** Map T-115 to explicitly retained/replaced portions; T-116's failed
  attempts remain history and T-123 carries the new attempt; T-117 remains not-run
  until equivalent post-live verification actually occurs. Never mark the failed
  S14 tasks retroactively complete or realize INT-0024/INT-0030 again. The old
  unverified recovery/tool changes retained in S14 must be included in focused
  verification if still part of the final candidate. No installation, default
  promotion, merge or release is implied by this proposal.

## Explicit Deferrals and Failure Exit
General browser tools, arbitrary URL/JS execution, automatic requirement graphs,
recursive semantic supervisors, new completion acceptance states, persistent
sessions, model sweeps and Windows OS sandbox implementation remain out of scope.
If a diagnostic finds a necessary capability outside this scope, preserve the
decision and return to research/plan rather than patching the frozen plan through
prose. A sprint that cannot reach its live gate closes failed/inconclusive with
partial implementation clearly unverified. INT-0032 remains active/unrealized.

## Approval and Finalization
Before finalization, copy approved proposals to canonical plans, transition
INT-0033 to planned with Work evidence, preserve INT-0032 active, run the required
canonical-plan critic, address concerns, update metadata, and invoke the installed
finalize-plan.sh. Preliminary scratch review does not replace that gate. The
backlog gains only the approved T-119–T-124 schedule; source stays unchanged until
finalization. Sprint 15 work remains local while the failed S14 draft checkpoint
awaits its separate human disposition.
