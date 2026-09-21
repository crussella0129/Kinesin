Finalized - DO NOT EDIT

# Sprint 14 Build Plan

Implementation-owner design agreed; independent critique precedes locking.
Official tests follow live confidence.

## Intents
- [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) —
  planned; AC1–AC4 including AC2a. INT-0024 remains the wider evaluation backlog.

## Schema Tree
- Less operator correction
  - T-115: bounded receipt-grounded recovery and compatible replay
  - T-116: frozen fresh-app and natural-follow-up live workload
  - T-117: focused post-live regressions and independent acceptance

## Execution Sequence

### T-115: Recover common freeform workflow failures without operator patches
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- **Touches:** src/recovery.rs; src/lib.rs; src/core.rs; src/runner.rs;
  src/replay.rs; src/tools.rs; src/model.rs; src/onboarding.rs; src/config.rs;
  corresponding focused tests after the live gate; operator documentation
- **Depends on:** none
- **Acceptance criterion:** AC1 recovery; AC2/AC2a bounded authorized behavior and fresh defaults
- **Success criterion (EARS):**
  - **WHEN** eligible freeform work returns tool-protocol-looking prose or a
    completion answer, **THEN** the harness **SHALL** reject observable protocol
    failures with at most two repair nudges and separately perform at most one
    receipt-grounded completion review, without executing prose, forcing writes
    for questions or minting checked acceptance.
  - **WHEN** a recoverable empty/truncated model reply or tool/edit error occurs,
    **THEN** the harness **SHALL** offer bounded generic retry/reread guidance
    and continue through valid model/tool protocol, preserving exact-edit checks.
  - **WHEN** recovery or existing execution budgets are exhausted or the run is
    cancelled, **THEN** the harness **SHALL** stop truthfully without additional
    unauthorized dispatch or retaining rejected terminal candidates; checked
    and no-tool behavior stays within its contract.
  - **WHEN** new and prior captures are replayed, **THEN** replay **SHALL**
    reproduce their versioned transitions/request hashes without live effects.
  - **WHEN** a command exits nonzero or an exact edit cannot match, **THEN** the
    tool **SHALL** report failure with bounded useful stderr/reread guidance,
    preserving exact matching, denied effects and private-file boundaries.
  - **WHEN** a fresh local profile is created, **THEN** it **SHALL** use context
    16,384, output 2,400, 20 model turns, 30 tool calls and 240 seconds; loading
    an existing profile **SHALL** preserve its operator-specified values.
- **Notes:** core 4 enables recovery only for freeform compiled effectful grants;
  receipt summaries retain at most 12 receipts and 4,096 encoded JSON bytes,
  survive compaction and
  contain no arbitrary tool payloads. Deterministic workflow metadata is optional
  in model_finished. Tools 4/adapter 4 preserve old capture semantics. All existing
  budgets remain binding. No app-specific magic or prose-to-call conversion.

### T-116: Complete a fresh app and natural follow-up with zero corrective messages
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- **Touches:** docs/sprints/s14/sprint-research/live-workload.md;
  isolated app/control/evidence workspace; docs/sprints/s14/sprint-tests/e2e-tests.md;
  retained sanitized workload fixtures and attempts
- **Depends on:** T-115 implementation; workload card frozen before scored attempts
- **Acceptance criterion:** AC3 fresh live outcome; AC4 independent results and effort
- **Success criterion (EARS):**
  - **WHEN** the frozen initial request and planned natural follow-up run in a
    new workspace, **THEN** Kinesin **SHALL** produce and preview the requested
    storefront and follow-up with zero manual code patches, corrective messages,
    tool-forcing instructions or context resets in the accepted attempt.
  - **WHEN** independent browser/HTTP checks inspect the result, **THEN**
    catalog/search, cart quantity/removal/totals, synthetic checkout and the
    follow-up behavior **SHALL** match the frozen card's observable criteria.
  - **WHEN** any live attempt finishes or is abandoned, **THEN** the record
    **SHALL** retain prompts, model/source/profile, limits, outcomes, wall time,
    active operator time, automatic recoveries and manual-intervention counts.
- **Notes:** failed attempts are never replaced by the final passing story;
  generic product repairs may occur between clearly labeled fresh attempts.
  Do not claim a measured human-coding speedup without a human baseline.

### T-117: Verify bounded recovery after the live workflow is useful
- **Intent:** [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md)
- **Touches:** tests for affected recovery/runner/replay paths; docs/sprints/s14;
  INT-0032 evidence; docs/work ledgers
- **Depends on:** T-116 zero-correction live confidence gate
- **Acceptance criterion:** AC2 regressions and AC4 live-first formal verification
- **Success criterion (EARS):**
  - **WHEN** the frozen live workload passes, **THEN** focused official unit and
    integration tests, formatting, Clippy and independent critique **SHALL**
    verify the final recovery implementation and mapped outcomes before closure.
- **Notes:** source inspection and compilation during implementation are allowed;
  official unit/integration testing stays after the live gate as requested.
