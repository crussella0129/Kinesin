# INT-0010 — Cross-agent write coordination (presence-aware leases)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0010
- **State:** proposed
- **Work evidence:** [T-107 backlog](../work/tasks.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

> **Roadmap:** theme A (security hardening) — see [the roadmap](../roadmap.md) (INT-0011).

## Intent
Prevent concurrent agents from racing or overwriting each other when they work
on the same machine and/or in the same repository at the same time. Introduce a
coordination layer over shared writable resources (a workspace path, a file, or
a declared repository region) modeled on the **AMP / Lakewood** mortgage-banking
system: a resource is held by **exactly one writer at a time**, the runtime is
**aware of who is observing and who is holding** each resource, and a holder must
explicitly **"back out"** (release) before another waiting agent may take it.
Crucially, design against AMP's known failure mode — an **idle holder blocking
everyone** (the underwriter who sat in a screen all night and left loans
un-worked): every hold is a **lease with a TTL and heartbeat**, and an idle or
dead holder is **preempted** (or its lease expires) so the resource frees
without a human. Non-goal: distributed consensus across untrusted machines; a
general lock service for non-Kinesin processes; replacing git's own merge — this
coordinates *before* writes conflict, it does not resolve merges after.

## Acceptance criteria
- Lease ownership is fenced at the actual mutation boundary: a paused former holder that resumes after reassignment cannot write. Canonical resource identity, multi-resource acquisition order, and cross-process ownership are explicit and tested.
- A second agent's write to a resource already held is **blocked or queued**, not
  silently applied or lost; the first writer's changes are never clobbered.
- Releasing a hold ("back out"), or a completed run, **unblocks the next** waiter
  in a defined (e.g. FIFO/fair) order.
- **Presence is observable:** the runtime can answer "who holds and who is
  observing resource X" for humans and agents alike.
- An **idle or crashed holder is reclaimed**: a lease past its TTL without a
  heartbeat is preempted, the preemption is recorded, and the preempted run ends
  with a defined non-corrupting outcome (never a half-applied write).
- The coordination is honored across **concurrent runs on one machine** and
  across **separate agents/sessions in the same repository**; a matrix test
  proves no two writers commit overlapping changes to one resource.
- Waiting on a lease is journaled as an **observed event**, so the immutable-run
  / pure-replay contract still holds (a lease wait is nondeterministic input, not
  a pure decision).

## Rationale
This is the central safety problem for multi-agent operation and the stated
motivation for the whole architecture pass. The current runtime bounds
*resource competition* (scheduler admission, capability-scoped `WorkspaceWriter`s
per run) but has **no arbitration of two runs writing the same path** — nothing
stops overlapping edits. AMP's presence-aware single-writer model is a proven
(decades in mortgage banking) way to make concurrent editing safe, and its
famous weakness (idle lockout) is exactly what a lease/heartbeat/preemption
design neutralizes.

## Alternatives
- **Optimistic concurrency** (let both write, detect the conflict via
  hash/version, retry the loser). Lower latency when contention is rare; but it
  permits wasted work and needs a rollback story — worse when agents genuinely
  contend for the same files.
- **Isolation instead of locking**: give each agent its own git worktree/branch
  and merge later (the harness already supports worktree isolation). Strong for
  independent tasks; does not help when agents *must* edit the same live files.
- **OS advisory locks** (`flock`) only: simple and machine-local, but no
  presence, no fair queue, no idle preemption, and nothing across repositories.
- Do nothing (current): correct only while runs never share a path.

## Consequences
Adds a lease/presence registry — shared, durable state, a natural fit for the
existing single SQLite owner thread on one machine, but needing a coordination
service for the cross-machine case; a new **deadlock/starvation** surface (lease
ordering, priority, preemption policy) to design and test deliberately; a
tuning axis (TTL vs. responsiveness) that trades a snappy hand-off against
premature preemption of a slow-but-live holder; and an interaction with replay
that requires lease acquisition/expiry to be journaled as input. Pairs naturally
with the multi-agent operation that INT-0008 (remote model) and future
multi-session work enable.

## Transition history
- 2026-09-11: created as `proposed`.
- 2026-09-12: proposed acceptance clarified by the intent-first sprint 10 audit; implementation remains proposed.
