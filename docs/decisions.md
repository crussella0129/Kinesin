# Decisions and tradeoffs

Reviewed 2026-09-08 UTC. These are design choices for the implementation you will
write, not claims of measured performance or a deployed security boundary.

The priority change is explicit: **minimality, security, low latency, and
scalability all matter**. The first complete release includes concurrent agents
for one owner; a shared service follows. “Maybe add async later” no longer
describes that destination.

## Chosen mechanisms

| Choice | Why it fits | Cost/limit | Revisit when |
|--------|-------------|------------|--------------|
| One package, small modules | Keeps learning/build boundaries understandable | In-process modules are not hostile-code isolation | Reuse or deployment needs a real boundary |
| Pure synchronous core | Deterministic transitions and simple Rust learning | Effects still need a carefully owned shell | The state contract changes, not just because async exists |
| Tokio + reqwest from first HTTP | Supports independent concurrent waits and streaming | Async ownership/cancellation must be learned | A workload requires different runtime constraints |
| Concrete Scripted/Http client enum | Fake and real boundary without async trait-object machinery | Adding adapters changes an enum | Independent plugin/library consumers need an open interface |
| Separate execution and acceptance outcomes | A complete answer cannot masquerade as a verified solution | Freeform work remains unchecked; strict CLI returns nonzero by default | Another well-specified task class needs a checker |
| Frozen FileFieldsV1 contract and pure checker | Independently compares structured claims with actual scoped observations | Proves only configured fields as observed; bounded evidence/receipt overhead | A task needs different evidence, semantics, or expensive checking |
| Capability-backed WorkspaceReader | Narrows file authority without hand-writing path-race logic | Trusted code can bypass it; exposed files can contain secrets | Hostile executables or filesystem objects become supported |
| Bounded central scheduler | Makes overload, cancellation, and ownership explicit | Queues/permits need independent byte and waiter accounting | One controller's measured capacity is insufficient |
| SQLite WAL + FULL | Transactions, indexed ownership queries, durable acknowledged status | One writer; local filesystem; disk latency/checkpointing remain real | Measured storage bottleneck or multi-controller requirement |
| One storage owner thread | Keeps blocking DB work off runtime threads; clear ownership | Commands and acknowledgements need bounds/error policy | Bounded read workers demonstrably improve query latency |
| Metadata capture plus retained final results | Limits unnecessary intermediate-content retention | Exact replay unavailable; final results can still be sensitive | Explicit private replay/debug task |
| Normalized replay deltas | Avoids storing growing full prompts repeatedly | Requires compatible versions and every state-affecting input | A forensic raw-wire capture is explicitly needed |
| Serial tools within each run | Straightforward call/result ordering and authority | Independent tool reads could overlap later | Measurements show a useful gain and dependencies are known |
| Explicit no generation/tool retry | Avoids inventing safe replay of ambiguous effects | Transient failures can end a run | Effect-specific retry/reconciliation is designed |
| Single-controller authenticated service | Useful shared deployment without distributed ownership protocol | One controller is a trust/availability boundary | Load/availability evidence justifies another architecture |

See [research](research.md) for the primary evidence and
[architecture](architecture.md) for the resulting contracts.

## Why this is still minimal

Minimal code length, minimum dependency count, and minimum maintenance burden are
different goals. A library can increase compiled code while reducing the number
of tricky mechanisms you must implement and prove.

Tokio adds runtime machinery but avoids a bespoke worker/concurrency framework.
SQLite adds SQL and a native library but avoids a private crash-recovery format
and mutable event index. `cap-std` adds a dependency but avoids treating a string
prefix/canonicalization recipe as secure path resolution.

You still hand-write the interesting runtime: the core, policy, request mapping,
resource accounting, tools, event schema, cancellation, service authorization,
and tests. Parser/socket experiments remain worthwhile in a scratch project;
they are not prerequisites for building the harness.

## Deliberate alternatives not selected

**One thread per run with blocking HTTP.** Viable for a small fixed workload.
It is a useful systems exercise, but would make streaming, cancellation,
subscriber handling, and a shared async API a second integration path here.

**A large agent framework.** It may accelerate application development, but this
project's purpose includes learning and owning the harness itself. Study its
interfaces when useful; do not adopt its entire execution model just to obtain
two tools and a loop.

**JSONL or one JSON file per event.** Fine for an export or a small local recorder.
The required shared-service queries, idempotent admission, projection updates,
and crash behavior would force extra persistence code. SQLite has a clearer
contract for this destination.

**PostgreSQL and a distributed queue immediately.** Reasonable once controllers
or durable workers span hosts. Initially they add deployment/lifecycle work
without removing a measured bottleneck. Do not claim SQLite on a shared mount
is an equivalent shortcut.

**An embedding index and vector retrieval.** Reasonable when evidence is rarely
literal. Kinesin's checked contract compares a claimed field against the bytes of
a named source, which is the distribution where measured lexical search wins:
[Sen et al.](https://arxiv.org/abs/2605.15184) report inline grep above inline
vector for every harness and model pair they evaluate. A vector path would add an
embedding model, an index, and indexing latency to serve answers that are already
literal. Revisit for a task class whose evidence is paraphrased rather than
quoted, and see the [paper review](paper-review.md) for what that result does not
cover.

**File-pointer (programmatic) tool results.** Writing a large result to disk and
returning a path decouples result size from context pressure, and it is the right
answer for a strong backbone that reliably closes a read-then-integrate loop. The
same study measures the cost when it does not: one pair falls from 93.1% inline
to 55.2% programmatic, and weaker backbones show the largest gaps. Kinesin runs
8B local models, so it keeps inline results with explicit truncation and an
honest `truncated` flag. Revisit when a measured task needs a result larger than
the context can hold, and treat closing that loop as the thing to prove first.

**MCP immediately.** Add it when existing tool servers solve a real task.
Discovery and schemas do not establish authority. It must adapt into the same
policy and execution limits.

**Arbitrary shell/code tools immediately.** They would change the trust model
before the first project has a tested process/worker boundary. Shell execution is
a different trust class from a bounded file write: arbitrary process spawning, an
argument vector rather than a shell string, output bounding, its own timeout, and
reconciliation of an effect that cannot be un-run. It stays deferred until a
demonstrated task needs it, and it arrives with that machinery, not before.

**A bounded `write_file`, built.** The read-only posture was a security stance,
not a permanent limit. `write_file` now crosses it deliberately, and keeps the
stance's guarantees: a `WorkspaceWriter` separate from the reader so reads cannot
write; a writer built only for a workspace the operator granted, so an ungranted
run has none; an atomic temp-then-rename that never leaves a partial file; refusal
to leave the root, follow a symlink, or overwrite a directory; and a bar on write
tools in checked runs, since a run that could plant the value it later reads would
certify its own change. `edit_file` follows: it replaces one
unique passage through the same capability and guards, refusing an absent or
ambiguous match rather than editing the wrong place. `delete_file` and `move_file` complete
the file-mutation set: delete removes only a regular file, and move refuses to
overwrite an existing destination. Shell execution is the remaining, and largest,
deferred surface. See [security](security.md).

**Custom VPN/gateway.** An existing OS route reaches private inference already.
A gateway must earn its place through authentication, policy, queueing, or
operational needs. Do not mix VPN maintenance into model-message code.

## Paper-informed refinement

The [paper improvement pass](paper-review.md) preserves the selected runtime.
It adds explicit input provenance, feedback-dependent tasks, and independent
goal evaluation. ReAct informs the feedback loop; ICM informs context authoring;
PDDL-INSTRUCT informs validation exercises; OoO-Spec belongs to optional backend
research. None establishes that a folder layout, model-generated plan, or draft
token can replace enforced authority.

Keep structured tool calls and serial tools within a run initially. A future
parallel scheduler or speculative backend must earn its complexity through the
named experiments. The guide's step count is a consequence of its teaching
sequence, not a project requirement.

## Revision history

| Version of the plan | Main direction | Reason for the next change |
|---------------------|----------------|----------------------------|
| Original checked-in draft | Several named processes/crates, handwritten formats, per-event file/index, private remote link | Too much protocol/infrastructure work before a useful learning milestone |
| First refinement | One synchronous local program, ureq, per-run event files, concurrency deferred | Easier first Rust project, but underweighted the clarified concurrent/shared-service goal |
| This redesign | Pure core then async I/O, capability tools, transactional store, bounded concurrency, explicit shared-service gate | Treats the four goals as actual requirements and provides a complete build path |
| Paper pass | Context provenance, feedback tests, independent offline goal checks | Completion versus correctness still lacked a per-run contract |
| Adversarial pass | Frozen task profiles, bounded pure acceptance checks, atomic receipts, separate outcomes | Supports narrow checked tasks and explicitly leaves unrestricted work unchecked |

The preceding working draft was uncommitted. A separate local copy was made before
this redesign; the original checked-in plan also remains in Git history.

## What measurements can change

Do not defend a setting because this document picked it. Measure model slots and
latency, storage commit delay, memory under overload, and actual task success.
Change one relevant variable at a time. Record the policy/durability mode alongside
performance results.

A genuine tradeoff may require choosing a different target or dependency. Never
hide it by dropping validation, reducing durability without disclosure, allowing
unbounded queues, or treating incomplete output as success.
