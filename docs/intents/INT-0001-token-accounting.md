# INT-0001 — Token accounting

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** active
- **Work evidence:** [T-001 build plan](../sprints/s0/sprint-plans/build-plan.md#t-001-parse-and-carry-usage-out-of-koil)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Record per-run model token usage and surface it. `llama-server` returns a
`usage` object (prompt and completion tokens) that the streaming path already
validates and then discards; `Counters` tracks only `model_turns` and
`tool_calls`. Capture the reported counts into the run record and the
`inspect` output for both streaming and non-streaming replies. Non-goal:
per-span tracing or a metrics subsystem.

## Acceptance criteria
- A completed run's stored record and `inspect` output include prompt and
  completion token totals when the server reports them.
- Absence is honest: unreported usage stays unknown, never reported as zero.
- Both the streaming and non-streaming paths capture it.
- Tests assert capture, the honest-absence case, and any storage-schema
  version and replay-compatibility handling.

## Rationale
The stated low-latency and scaling goal needs per-run token visibility, and the
data is already in the response, so this is cheap and adds no new trust surface.

## Alternatives
A separate telemetry module (premature; owned timing records already exist).
Per-span token attribution (deferred; larger and not yet needed).

## Consequences
Adds optional token fields to the journalled events and the terminal `counters`.
Research established these ride in the existing schemaless `data_json` column, so
**no SQLite schema migration is required**. The added fields are additive and
optional, so replay of older captures is unaffected; the one replay interaction
is that requesting streamed usage changes the streaming request fingerprint, so
any streaming replay capture must be re-recorded or confirmed absent.

## Transition history
- 2026-09-08: created as `proposed`.
- 2026-09-08: `proposed → planned`; selected for sprint 0 and linked to T-001 in the build plan.
- 2026-09-08: amended Consequences after research — no SQLite migration is needed (usage rides in schemaless event data); the only replay interaction is the streaming request fingerprint change.
- 2026-09-08: `planned → active`; sprint 0 build began.
