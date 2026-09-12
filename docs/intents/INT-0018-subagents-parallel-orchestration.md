# INT-0018 — Subagents & bounded parallel orchestration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0018
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Let a run fan work out to bounded child runs ("subagents"), each with its own
context window, capability subset, and budget, and gather their results — the SotA
pattern for scaling beyond a single context window and for adversarial
verification (a separate checker run). Parallelism must preserve the project's
invariants: each subagent is its own owner-scoped, admitted, journalled,
replayable run under an authority derived from (and no broader than) the parent's;
scheduling stays inside the existing admission/resource bounds. Non-goals: an
unbounded fan-out; shared mutable state between subagents (they coordinate through
results and, where they touch a shared workspace, through INT-0010); model-authored
orchestration that escapes enforced authority.

## Acceptance criteria
- Bounds cover total descendants, nesting depth and cumulative budget; parent cancellation reaches descendants, and a parent waiting at full admission capacity cannot deadlock its own children.
- A parent run can spawn a bounded set of subagent runs with narrowed
  authority/tools/budget; total concurrency stays within configured admission and
  resource limits (no bypass).
- Each subagent produces its own immutable, replayable journal; a subagent's
  authority is provably a subset of the parent's (a subagent cannot reach a tool,
  workspace, or owner the parent lacks).
- Results compose deterministically enough to replay the parent's decision to
  fan out and integrate; a failed/cancelled subagent yields a defined outcome.
- Concurrent subagents touching the same workspace are coordinated (INT-0010), not
  racing.

## Rationale
Parallel subagent orchestration (plan-in-code fan-out with adversarial
verification) is a defining 2026 SotA capability. decisions.md deferred the
"future parallel scheduler," requiring it to "earn its complexity through named
experiments" — this intent captures that workstream and its guardrails. It is
the lowest-priority roadmap item: genuinely downstream of the scheduler, INT-0010
(coordination), and a demonstrated task that needs it.

## Alternatives
Serial single-run only (current; simplest, but caps work at one context window and
one worker). External multi-process orchestration above the CLI (loses shared
admission bounds, unified journalling, and subset-authority proof). Model-authored
orchestration scripts without an enforced authority subset (rejected — orchestration
must not enlarge authority).

## Consequences
Significant scheduler and authority-derivation work; a replay story for a
fan-out/gather shape; resource-accounting across a run tree; a new starvation/
deadlock surface shared with INT-0010; only worth starting once the coordination
and scheduling foundations and a real motivating task exist.

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme D — SotA capability); lowest priority — downstream of the scheduler, [INT-0010], and a demonstrated need.
- 2026-09-12: proposed acceptance clarified by the intent-first sprint 10 audit; implementation remains proposed.
