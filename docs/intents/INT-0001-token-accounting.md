# INT-0001 — Token accounting

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** proposed
- **Work evidence:** none
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
Adds fields to the run record; a storage-schema version bump and a replay
compatibility decision for older captures.

## Transition history
- 2026-09-08: created as `proposed`.
