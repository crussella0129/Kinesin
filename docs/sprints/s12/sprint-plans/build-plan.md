# Sprint 12 Build Plan

## Intents
- [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md) — planned;
  all four acceptance criteria, with INT-0026's larger scope explicitly deferred.

## Schema Tree
- Usable local assistant
  - T-110: bounded context, frozen provenance and replay
  - T-111: interactive memory, reset/status, local launch and editing workflow

## Execution Sequence

### T-110: Freeze bounded session context independently of prior answers
- **Intent:** [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md)
- **Touches:** src/session.rs; src/lib.rs; src/policy.rs; src/config.rs;
  src/core.rs; src/runner.rs; src/replay.rs; related tests after implementation
- **Depends on:** none
- **Acceptance criterion:** bounded origin-marked memory, fresh authority,
  capture privacy and deterministic replay.
- **Success criterion (EARS):**
  - **WHEN** recent session turns are supplied, **THEN** initialization **SHALL**
    retain ordered user/answer pairs with run origin within fixed byte/count bounds.
  - **WHEN** context is invalid or submitted with a checked task or legacy prior
    answer, **THEN** authorization **SHALL** reject it without effects.
  - **WHEN** metadata or replay capture is selected, **THEN** only replay **SHALL**
    persist context content, validate its source digest, and reproduce requests;
    absent context SHALL preserve legacy request construction.
- **Notes:** implement before writing/running tests per user instruction.

### T-111: Keep interactive follow-ups useful and demonstrate local file work
- **Intent:** [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md)
- **Touches:** src/cli.rs; src/cli/presentation.rs; src/session.rs;
  README.md; docs/cli.md; docs/getting-started.md; sprint evidence and tests
- **Depends on:** T-110 interfaces; CLI implementation may proceed in parallel
- **Acceptance criterion:** recovery from long/failed turns, reset/status and
  real local launch/context/file-edit workflow.
- **Success criterion (EARS):**
  - **WHEN** a reply is oversized or an entry fails, **THEN** the next request
    **SHALL** remain usable; clipping/eviction is reported and completed history
    survives failed entries. Initial history and compiled-tool encoded request
    bounds are honored. Dynamic MCP schemas remain admitted by the runner after
    discovery; fitting session history to those schemas is deferred to INT-0026.
  - **WHEN** `/new`, `/clear` or `/context` is entered, **THEN** the CLI **SHALL**
    reset or report in-memory context without a model call; JSON output remains
    machine-readable and context notices do not become result receipts.
  - **WHEN** the built assistant runs with the installed local model, **THEN** it
    **SHALL** recall an earlier detail and create/edit a file in the chosen folder.
- **Notes:** implementation first; then format, Clippy, focused regressions,
  native live walkthrough and independent review. No push or merge authorized.
