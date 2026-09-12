# Sprint 2 Research Report

## Intents Reviewed
- [INT-0002](../../../intents/INT-0002-context-compaction.md) — selected; relevance: this sprint's sole goal is to let a long run/session continue past `max_history_bytes` instead of stopping; current state: `proposed` (moves to `planned` at plan finalization). No revision to the intent was needed.

## 1. Sprint Goal
When a run's conversation reaches `max_history_bytes`, keep going under a bounded,
evidence-preserving policy instead of stopping with `history_bytes_limit`. This
matters now that interactive sessions exist (each turn a new run citing the prior
answer), so a long thread must degrade gracefully. The strategy must be **pure and
deterministic in the core** so replay reproduces the exact compacted conversation
and its request fingerprints; it must **preserve complete tool-call/result groups**
(never leave a partial group) and **never drop or rewrite evidence bytes in a way
that changes a checked-run verdict**; and it must hold the immutability and replay
contracts. Non-goals: unbounded history; a strategy that mutates evidence.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| [src/core.rs](../../../../src/core.rs) | high | `RunState { messages: Vec<Message>, … }` (l.144), the pure transition owner. A group is an assistant `tool_calls` message (l.352) followed by ordered `Role::Tool` results (l.380). Compaction belongs here so it is pure and deterministic. |
| [src/runner.rs](../../../../src/runner.rs) | high | Enforces the limit at three sites (l.691, l.861, l.1147), each `stop(Stopped, "history_bytes_limit")`; `history_exceeds` (l.1160) serializes and compares. These stops become compaction calls. |
| [src/replay.rs](../../../../src/replay.rs) | high | Mirrors the same check and stop (l.587, l.742, l.762) with its own `history_exceeds` (l.887). Replay recomputes state, so it must invoke the identical pure compaction or its fingerprints diverge. |
| [src/model.rs](../../../../src/model.rs) | high | `conversation_json` (l.98) and `prepare` (l.131) build the request whose `sha256` is the replay fingerprint; the compacted message list flows through here unchanged. |
| [src/config.rs](../../../../src/config.rs) | high | `max_history_bytes` (l.187, default 65536). A compaction policy + ceiling (floor of preserved turns / max compactions) is added here. |
| [src/policy.rs](../../../../src/policy.rs) | medium | Rejects an initial conversation already over budget (l.373); compaction never applies to the initial two messages. |
| [src/verification.rs](../../../../src/verification.rs) | medium | Resolves cited `evidence_id`s for checked acceptance; a dropped evidence-bearing result would change a verdict, so these results must be preserved. |
| [tests/runner_journal.rs](../../../../tests/runner_journal.rs) | medium | Where the boundary/preserved-group/checked-run integration tests will live. |
| [tests/replay.rs](../../../../tests/replay.rs) | medium | Must still pass unchanged, proving replay reproduces a compacted run's fingerprints. |

## 3. External Sources
- [Kinesin paper review](https://github.com/crussella0129/building-an-agent-harness/blob/main/paper-review.md) — records the file-pointer (programmatic tool-result) alternative and why it was rejected for weak local backbones; supports a drop/summary strategy over pointers.

## 4. Risks, Unknowns, Dependencies
- **Risk — replay divergence.** The runner and replay independently gate on
  `max_history_bytes`. If compaction is not pure and applied identically in both,
  the recomputed request `sha256` diverges and replay fails. Mitigation: put the
  drop logic in `core` as a deterministic `RunState` method and call it from both
  the runner and replay at the sites that currently stop.
- **Risk — checked-run evidence loss.** A checked candidate cites `evidence_id`s
  from `read_file` results; dropping such a result would change acceptance.
  Mitigation: compaction preserves evidence-bearing tool results (and their group),
  dropping only non-evidence groups/turns; if it cannot get under the limit without
  dropping evidence, it stops (bounded), which keeps today's behavior for that case.
- **Risk — partial groups.** Dropping half a tool-call/result group would leave a
  dangling assistant `tool_calls` or an orphan `Role::Tool`, which the model API
  rejects. Mitigation: the unit of compaction is a whole group (or a whole plain
  user/assistant turn), never a single message.
- **Unknown — strategy.** Drop-oldest (deterministic, no model call) vs. a summary
  of older turns (adds a model call whose output is non-deterministic and would
  itself need to be journalled and replayed). Recommend **drop-oldest** for this
  sprint; a summary strategy is a larger follow-up because of the replay contract.
- **Unknown — the ceiling.** What "up to a configured ceiling" means concretely: a
  floor of preserved recent turns and/or the system + initial task turn always kept;
  when compaction cannot fit, stop. Settled in the plan.
- **Dependency — none on other intents.** Reuses the existing journalling and
  replay untouched in contract; adds a compaction event for observability.

## 5. Recommended Approach
**Primary.** Add a pure, deterministic `compact` step to `core::RunState`: given the
byte limit, while the serialized conversation exceeds it, drop the oldest droppable
unit — a complete tool-call/result group or a plain user/assistant turn — always
preserving the system message, the initial task/prompt turn, a floor of the most
recent turns, and any evidence-bearing tool result. Replace the three runner stop
sites and the mirrored replay sites with a call to this method; when it cannot get
under the limit without violating a preservation rule, it still stops
(`history_bytes_limit`), so the checked-evidence and floor cases keep today's
behavior. Journal a `history_compacted` event (dropped-group count / bytes) for
observability; the event is additive, so replay of older captures is unaffected.
Add a config policy (enable + floor/ceiling) under `max_history_bytes`.

**Alternative considered.** A summary-of-older-turns strategy. Deferred: it adds a
model call and non-deterministic text that must be captured and replayed, a larger
change than the replay contract warrants for a first cut.

**Rationale.** Keeping the drop logic pure in `core` and invoked identically by the
runner and replay is the only way to preserve the fingerprint-replay contract for
free, and dropping whole groups while preserving evidence keeps the checked-run
verdict and the API's group-integrity rule intact.

## Artifacts
- No code snippets saved; the survey cites live source at the paths and lines above.
