# INT-0022 — Repair completed harness contracts

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0022
- **State:** realized
- **Work evidence:** [sprint 10 build plan](../sprints/s10/sprint-plans/build-plan.md)
- **Completion evidence:** [T-011 integrated completion](../work/completed-tasks.md#t-011-sprint-10), [T-007 MCP completion](../work/completed-tasks.md#t-007-sprint-10), [T-005 sandbox completion](../work/completed-tasks.md#t-005-sprint-10)
- **Code evidence:** [core](../../src/core.rs), [model](../../src/model.rs), [policy](../../src/policy.rs), [runner](../../src/runner.rs), [replay](../../src/replay.rs), [tools](../../src/tools.rs), [process ownership](../../src/process.rs), [MCP](../../src/mcp.rs), [configuration](../../src/config.rs), [dependency policy](../../deny.toml)
- **Test evidence:** [sprint 10 test report](../sprints/s10/sprint-tests/test-report.md)
- **Documentation evidence:** [security boundaries](../security.md), [configuration](../configuration.md), [supply-chain policy](../supply-chain.md), [scoped remote observations](../sprints/s10/sprint-tests/remote-deployment.md)

## Intent
Repair the concrete completed-intent defects identified in sprint 10 research while preserving authority, checked evidence, bounded effects and pure replay. Scope is token-accounting honesty, freeform compaction, command lifecycle/output and Linux isolation, MCP lifecycle/resource/authority gates, confidential model-origin policy, and an enforced dependency policy. Historical INT-0001/0002/0003/0005/0008/0012/0013 remain provenance; this follow-on owns the repairs. Non-goals: implementing every proposed capability or claiming unavailable live/platform evidence.

## Acceptance criteria
- Continuation admission counts the actual frozen prior-answer framing and content; faithful cache validation uses harness-prepared session requests, without claiming unexecuted live timing or full session-history continuity.
- An admitted MCP startup either durably freezes the discovered definitions before any model dispatch, or settles a defined failure/cancellation outcome with no model/tool effect. Replay validates that ordering without reconnecting.
- A workspace move never replaces an existing or concurrently created destination; no-replace semantics are atomic. Broader stale-edit fencing and shared leases remain proposed.
- Unreported token dimensions and incomplete multi-call totals remain unknown; complete streaming/non-streaming counts remain exact.
- Read-heavy freeform runs can compact complete obsolete groups under policy; checked-run evidence remains immutable, and replay reproduces both.
- Commands enforce encoded result caps, avoid pre-cancelled spawn, and terminate their owned process group/job on normal exit, timeout, cancellation and owned cleanup. Cleanup awaits owned handles and reaps the direct child; Unix orphan-zombie reaping belongs to the OS. Linux refuses incomplete enforcement and denies outside truncation/network bypasses, process-group escape and private-state access.
- MCP startup, discovery and sessions obey bounded admission/cancellation/deadlines; frames and discovery metadata are bounded before accumulation; server environment/stderr and process teardown have explicit safe defaults.
- Both live and replay MCP dispatch require frozen definitions AND run allow-list membership; replay performs no external effects.
- Non-loopback model URLs require HTTPS; private address classification never implies encryption; redirects and ambient proxies remain disabled.
- Duplicate dependency violations fail CI except version-specific reviewed exceptions; adversarial regressions demonstrate the repaired cases; format/clippy/tests and supported-platform CI pass.

## Rationale
Existing passing tests miss defects that contradict already recorded acceptance criteria. Repairing those contracts is necessary before adding more capability.

## Alternatives
Only revise prose (rejected for actual defects); replace the harness architecture (unnecessary; preserve the pure-core and journal design).

## Consequences
Stricter transport and sandbox enforcement can reject previously accepted configurations. Record the migration path and evidence limits rather than weakening controls.
Operator-trusted MCP binaries must not deliberately escape their owned process group; they remain outside the command sandbox. The process owner terminates that group/job and awaits owned cleanup, without claiming a Unix subreaper or confinement of hostile trusted-server code.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.
- 2026-09-12: `proposed → planned`; selected for sprint 10 under the user's instruction to audit, repair, verify and submit the PR.
- 2026-09-12: `planned → active`; the reviewed plan pair is locked and sprint 10 Build has begun.
- 2026-09-12: `active → realized`; every repair criterion is linked to executed adversarial/regression evidence, successful native Windows/Ubuntu CI and a clean independent TEST critique. Full local Linux attempts that failed from mounted execution or storage limits are not counted as passing; broader proposals retain their own acceptance boundaries.
