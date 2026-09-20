# Sprint 12 Research Report

## Intents Reviewed
- [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md) — created;
  planned bounded usable session memory and local workflow.
- [INT-0026](../../../intents/INT-0026-session-context-continuity.md) — selected
  for boundaries; broader token admission and persistence remain proposed.
- [INT-0029](../../../intents/INT-0029-interactive-entry-and-workspaces.md) —
  selected; existing realized launch and workspace tools remain the baseline.

## 1. Sprint Goal
Deliver usable multi-turn local assistant sessions, keeping the existing
launcher and file-edit tools. Implement first, then perform verification as
the user explicitly requested.

## 2. Existing Code Survey
| File | Relevance | Notes |
| --- | --- | --- |
| src/cli.rs | high | Only the preceding candidate survives a turn. |
| src/cli/presentation.rs | high | Session help and progress presentation. |
| src/policy.rs | high | Frozen prior-answer authority and source metadata. |
| src/core.rs | high | Initial messages shared with replay. |
| src/runner.rs | high | Actual model options and run completion. |
| src/replay.rs | high | Frozen inputs and source validation. |
| src/config.rs | high | Fresh local authorization and byte limits. |
| src/model.rs | high | Encoded requests and compiled tool schemas. |
| src/onboarding.rs | high | Existing local setup and writable personal profile. |
| src/storage.rs | medium | Durable result lookup; metadata excludes prompts. |
| README.md | medium | Product entry instructions. |
| docs/managed-model-entry.md | medium | Existing native runtime provenance. |

## 3. External Sources
None required: this change uses existing internal interfaces and installed
runtime. No library/API changes are planned.

## 4. Risks, Unknowns, Dependencies
- Prior-answer provenance cannot be silently changed into mixed user/model text.
- Long answers currently make subsequent authorization fail indefinitely.
- Default model windows are small; bounded bytes are not exact token counts.
- Replay and live request construction must agree; legacy empty-context
  captures must remain unchanged. Metadata capture must not gain prompt bodies.
- The native runtime/model must be located before the final live walkthrough.

## 5. Recommended Approach
Add a distinct optional typed context to frozen authority, with shared initial
message construction and source hashing. Maintain a bounded recent-turn deque
in the CLI. Trim oldest turns to actual history/request byte limits, retain
successful history after failures, expose context status and reset. Re-read
files through existing tools before edits. Do not add persistent session storage
or pretend the broader INT-0026 criteria have been completed.

## Artifacts
- [Build plan](../sprint-plans/build-plan.md)
- [Deferred test plan](../sprint-plans/test-plan.md)

## Orchestration
Codex has no EnterPlanMode/ExitPlanMode or TaskCreate tools in this host.
The user has authorized the implementation and clarified scope; maintain the
Book ledger and use bounded independent review without another approval prompt.
The user's explicit testing-after-implementation instruction overrides the
skill's tests-before-each-task ordering; no tests run before implementation.
