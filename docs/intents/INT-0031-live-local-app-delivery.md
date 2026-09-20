# INT-0031 — Deliver and operate a local app through Kinesin

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0031
- **State:** realized
- **Work evidence:** [T-112–T-114 sprint 13 plan](../sprints/s13/sprint-plans/build-plan.md)
- **Completion evidence:** [T-112–T-114](../work/completed-tasks.md#t-112-sprint-13)
- **Code evidence:** [confined preview](../../src/preview.rs), [owned dispatch](../../src/runner.rs), [session cleanup](../../src/cli.rs), [versioned replay](../../src/replay.rs)
- **Test evidence:** [accepted report](../sprints/s13/sprint-tests/test-report.md), [live browser and tool evidence](../sprints/s13/sprint-tests/e2e-tests.md), [focused integration results](../sprints/s13/sprint-tests/integration-tests.md)
- **Documentation evidence:** [preview use](../getting-started.md), [preview contract](../loop-and-tools.md#local-website-previews)

## Intent
Use the real local Kinesin assistant to build and operate a small storefront in
an isolated disposable workspace. Drive its actual file, model and server
workflow through failures and repairs until the served app works in a browser.
Only then run the sprint's official unit and integration verification. This is
one concrete usability workload; the wider evaluation program remains INT-0024.

## Acceptance criteria
- AC1: A dedicated workspace, private profile and retained activity record
  identify the initial state, model/configuration, prompts, resource ceilings,
  generated files and observed failures. Routine destructive app experiments
  cannot overwrite the Kinesin source or the user's normal workspace/settings
  through the granted file tools. This is a scoped workspace, not a claim of
  operating-system isolation for arbitrary executables.
- AC2: Kinesin itself writes the storefront through actual granted tool calls.
  The delivered app has a product catalog, filtering or search, a cart with
  quantity/removal and totals, and a checkout confirmation using synthetic data.
  Follow-up requests change the app through the same assistant workflow.
- AC3: Kinesin initiates a locally owned static preview server through a granted
  `start_preview` tool, and the resulting
  loopback port serves the app in the internal browser. The operator exercises
  catalog, cart and checkout, verifies the requested follow-up, and records the
  actual launch/stop lifecycle. Preview serves only the selected workspace
  directory, rejects traversal/symlink escape and exposes no arbitrary command
  execution. The grant is local-CLI-only: service-mode configuration rejects
  preview authority to avoid shared multi-owner preview URLs. Do not substitute a manually authored app or
  label a manually launched server as a harness capability.
- AC4: Retain and fix concrete workflow failures encountered while operating
  the app, repeating the affected live path until it succeeds. Preserve existing
  authority, workspace/private-state separation and bounded process ownership.
  Record model limitations separately from runtime defects. Official unit and
  integration tests, formatting, Clippy and independent review follow live
  confidence; they do not replace the observed browser outcome.

## Rationale
The user wants evidence that the harness can complete a useful task, rather
than another round of isolated tests. Sprint 12 established launch, bounded
session memory and file edits; it did not prove app delivery or server use.

## Alternatives
Have the outer agent build the app (does not evaluate Kinesin); repeat only file
fixtures (misses the requested workflow); add unrestricted command authority to
normal personal settings (unnecessary expansion); build the entire INT-0024
evaluation system first (delays the concrete live exercise).

## Consequences
The app is disposable and uses no real purchases, credentials or customer data.
Use `target/storefront-lab/{app,control,evidence}` as disjoint app, private
profile/runtime, and observation directories. A first-party Rust static
preview avoids weakening command cleanup or sandbox restrictions. It is
owned by the CLI's run resources and bounded in lifetime; it remains available
across local conversational turns and closes on session/process shutdown.
Keep generated artifacts and local runtime state out of product source. Any
server capability must preserve existing authorization and process boundaries.
This sprint establishes one recorded operating scenario, not general model
competence, comprehensive sandboxing or a statistically measured benchmark.
The observed model repeatedly returned tool-result-like prose and needed
precise operator-guided corrections. Real tool events and independently
observed app behavior remain the outcome evidence; follow-up model-quality
work stays with INT-0024/T-103.

## Transition history
- 2026-09-20: created as proposed for the user's live storefront, local port and
  browser exercise, with official unit/integration tests deferred until after
  confidence from operating the app.
- 2026-09-20: proposed → planned; the observed command-server lifecycle gap is
  addressed by a confined local-CLI static preview tool. The user authorized
  the live exercise and necessary repairs; T-112–T-114 retain live-first order.
- 2026-09-20: planned → active after the planning critique resolved ownership,
  lifetime and touched-path gaps, and finalize-plan.sh locked both plans.
- 2026-09-20: active → realized after harness-authored file/tool evidence,
  catalog/cart/checkout and follow-up browser operation, owned preview shutdown,
  271 post-live passing checks, formatting/Clippy and accepted independent
  critique. Completion commits T-112–T-114 and final source identities are
  retained; model steering and native-Windows-only scope remain explicit.
