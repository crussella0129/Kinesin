# Sprint 13 Build Plan

Live construction and repair precede official unit/integration verification.

## Intents
- [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md) — planned;
  AC1–AC4, one isolated live workload. INT-0024 remains broader backlog scope.

## Schema Tree
- Harness-delivered storefront
  - T-112: isolated workspace, recorded real-model construction and follow-up
  - T-113: repair observed blockers and operate the served app in-browser
  - T-114: focused official verification after the live confidence gate

## Execution Sequence

### T-112: Drive the real assistant to build and revise an isolated storefront
- **Intent:** [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md)
- **Touches:** target/storefront-lab/{app,control,evidence}, disjoint from product source;
  docs/sprints/s13/sprint-tests/e2e-tests.md; retained sanitized workload artifacts
- **Depends on:** none
- **Acceptance criterion:** AC1 provenance/isolation; AC2 actual app authorship
- **Success criterion (EARS):**
  - **WHEN** the live workload begins, **THEN** its record **SHALL** identify
    initial workspace state, profile/model versions, prompts and ceilings, with
    product source and ordinary user configuration outside the app workspace.
  - **WHEN** Kinesin receives the storefront and follow-up requests, **THEN**
    actual tool calls **SHALL** create and revise files providing catalog,
    filtering/search, cart quantity/removal/totals and synthetic confirmation.
- **Notes:** retain unsuccessful attempts; the outer agent may prepare the
  environment and repair Kinesin but shall not substitute handwritten app code.

### T-113: Repair observed operating failures and deliver the app on loopback
- **Intent:** [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md)
- **Touches:** src/preview.rs; src/lib.rs; src/tools.rs; src/config.rs;
  src/runner.rs; src/cli.rs;
  src/model.rs; src/onboarding.rs if exposing the preview grant by default;
  operator documentation;
  docs/sprints/s13/sprint-tests/e2e-tests.md
- **Depends on:** T-112 initial real-model attempt and failure evidence
- **Acceptance criterion:** AC3 server/browser lifecycle; AC4 focused repairs
- **Success criterion (EARS):**
  - **WHEN** Kinesin is asked to serve its generated app, **THEN** its granted
    `start_preview { path }` tool **SHALL** initiate an owned Rust static server
    on loopback, serve the selected app directory through a local port and
    remain available across local conversation turns until bounded cleanup
    through CLI session/run-resource shutdown.
  - **WHEN** the browser operates the served app, **THEN** catalog filtering,
    cart quantities/removal/totals, synthetic checkout and the follow-up change
    **SHALL** work with independently recorded browser observations.
  - **WHEN** a concrete runtime blocker is reproduced, **THEN** its repair
    **SHALL** make the affected live path succeed without weakening workspace,
    private-state, authorization or owned-process boundaries.
  - **WHEN** preview is ungranted or a requested path escapes its capability
    root, **THEN** preview handling **SHALL** deny the effect without serving
    outside files; no arbitrary executable or public listener is introduced.
  - **WHEN** service-mode configuration grants preview authority, **THEN**
    configuration validation **SHALL** reject it before serving requests.
- **Notes:** existing command completion kills descendants and Linux denies
  bind; preserve both contracts. The preview uses existing Rust dependencies.
  This is a local CLI capability; multi-owner service previews are out of scope.
  Record model-quality limitations separately from runtime defects.

### T-114: Verify the finished workflow after live confidence
- **Intent:** [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md)
- **Touches:** focused tests for actual repaired paths; docs/sprints/s13;
  docs/work/tasks.md; docs/work/completed-tasks.md; intent completion evidence
- **Depends on:** T-112, T-113 live confidence gate
- **Acceptance criterion:** AC4 verification ordering and retained evidence
- **Success criterion (EARS):**
  - **WHEN** the live confidence gate has passed, **THEN** official focused
    unit/integration checks, formatting, Clippy and independent review **SHALL**
    run against the final repair, with failures resolved before sprint closure.
- **Notes:** gate requires observed catalog/cart/checkout, requested follow-up
  and owned server lifecycle. Do not run official tests before this gate.
