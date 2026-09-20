# Sprint 13 Research Report

## Intents Reviewed
- [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md) — created;
  planned isolated harness-authored storefront and live server/browser delivery.
- [INT-0024](../../../intents/INT-0024-harness-evaluation.md) — selected for
  workload provenance and independently observed results; broader evaluation
  cards and statistical comparisons remain proposed under T-103.
- [INT-0029](../../../intents/INT-0029-interactive-entry-and-workspaces.md) —
  selected; realized interactive launch and workspace tools are the baseline.
- [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md) — selected;
  realized bounded session history supports follow-up requests.

## 1. Sprint Goal
Make Kinesin produce and operate a small useful storefront in a disposable
workspace with a real local model, loopback server and internal browser. Repair
failures found by driving this workflow. Defer official unit/integration tests
until the live app, follow-up change and server lifecycle are convincing.

## 2. Existing Code Survey
| File | Relevance | Notes |
| --- | --- | --- |
| src/onboarding.rs | high | Personal profile grants file tools; no command grant by default. |
| src/cli.rs | high | Interactive prompts, session entry and fresh run preparation. |
| src/cli/session.rs | high | Bounded process-local context for follow-up edits. |
| src/config.rs | high | Workspace grants, command allowlist and private-path separation. |
| src/tools.rs | high | Actual file/command effects and sandbox validation. |
| src/process.rs | high | Owned process lifecycle; relevant to live server lifetime. |
| src/runner.rs | high | Run resources own bounded tool/server lifetime. |
| Cargo.toml | medium | Native Rust harness and existing HTTP/runtime dependencies. |
| docs/live-evaluation.md | high | Prior real-model failures and provenance conventions. |
| docs/sprints/s12/sprint-tests/e2e-tests.md | medium | Previous native memory/file-edit evidence. |

## 3. External Sources
None: initial work uses existing interfaces and the installed local runtime.
Inspect external primary documentation only if a concrete repair requires it.

## 4. Risks, Unknowns, Dependencies
- Default personal capabilities cannot launch a server. Existing run_command
  terminates descendants at completion and Linux sandboxing denies bind, so
  leaving a generic command running would weaken its contract. Add a granted
  first-party static preview instead of changing that lifecycle.
- The local model may emit code or tool syntax as prose, truncate long files,
  or need smaller follow-up requests. Retain such failures and distinguish
  model quality from harness/tool defects.
- Initial live request created the public directory, then printed HTML rather
  than calling write_file. Root is feeding that observed failure back through
  Kinesin; no app files were handwritten by the outer agent.
- The disposable app must remain separate from product source and private
  runtime/profile; stale listeners must not be mistaken for new output.
  Docker is unavailable: make no operating-system isolation claim. The initial
  profile's node allowlist is not a substitute for static preview confinement.
- Dependency downloads are avoidable for a small app. Prefer an installed
  runtime and keep network exposure on loopback.
- Service-mode owners must not share a preview URL or server authority. Reject
  service configurations granting preview; this sprint covers the local CLI.

## 5. Recommended Approach
Use disjoint `target/storefront-lab/{app,control,evidence}` directories,
preserving usual configuration. Record provenance and drive Kinesin to write the
storefront, start its server, and make an additional requested change. Inspect
the served result independently in the internal browser. Capture failed paths,
repair concrete runtime blockers, rebuild as necessary, and replay the affected
user path. Add a granted `start_preview { path }` tool with a capability-rooted
Rust static loopback server, no new dependencies, owned by CLI RunResources
across conversational turns and closed on session shutdown. Reject preview
grants in service mode. Preserve command cleanup and sandbox restrictions. Once the live
confidence gate passes, run focused regressions and the required sprint review.
Alternative considered: an outer-agent-authored demo would be faster but would
not establish the requested harness capability.

## Artifacts
- [New workload intent](../../../intents/INT-0031-live-local-app-delivery.md)
- [Build plan](../sprint-plans/build-plan.md)
- [Deferred official test plan](../sprint-plans/test-plan.md)
- Runtime transcripts, app snapshot and browser observations will be linked
  from the E2E evidence when they exist; no successful outcome is claimed here.

## Orchestration
Codex has no EnterPlanMode/ExitPlanMode or TaskCreate tools in this host. The
user has explicitly authorized this live exercise and repairs; use the Book
ledger and bounded independent review without another permission prompt.
The user's explicit sequencing overrides tests-before-task ordering: initial
live use and repair precede official unit/integration tests. The initial server
blocker bounds the planned repair; the planning reviewer checks the lock gate.
