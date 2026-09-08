# Feedback on your understanding of harnesses

This assessment is based on the original checked-in plan, your stated goals, and
our conversation. It is not a test of your Rust ability or a claim that every
sentence in the documents came from you. Earlier assistants contributed wording
and design choices, including my previous revision.

## What your plan already shows

Your strongest instinct is separating orchestration from inference. The original
architecture treats the control program as lightweight and the model as a resource
that may live on a different machine. That is a useful systems boundary.

You also gave tools, observability, configuration, and design reasons explicit
places. Those are central concerns in a harness. Keeping a stable model-facing
boundary while changing deployment is a sound goal.

The useful next step is to turn named components into precise contracts:
what each owns, what it may do, what resources it can consume, and what happens
when another component fails. Names and arrows describe structure; contracts
determine behavior.

## A more complete definition

A harness controls a model's interaction with a changing environment. It decides:

- What context and tool descriptions the model receives.
- How a proposed action becomes a validated and authorized effect.
- What observations enter the next conversation state.
- How time, tokens, memory, concurrency, and storage are allocated.
- How cancellation, errors, incomplete output, and completion are represented.
- Which task contract is being checked, what evidence supports it, and whether
  the result is passed, failed, unchecked, or inconclusive.
- Which records are retained and who may inspect them.

The model supplies proposals and language. The harness owns execution policy.
An inference server produces model outputs and manages inference capacity.
A tool implements a concrete operation. An agent run combines model decisions,
context, and tools under the harness's rules.

A simple loop is enough to demonstrate an agent. A useful general-purpose runtime
needs the surrounding ownership, policy, and failure rules to remain consistent
across different tools and workloads.

## Where the original plan was incomplete

| Evidence in the plan | What to add to your mental model |
|----------------------|--------------------------------|
| Supervisor drawn as a message hop | Lifecycle control and data forwarding are separate jobs; avoid a proxy unless it serves a purpose |
| Private remote link as the central distributed feature | Connectivity, identity, authorization, scheduling, and state ownership are separate boundaries |
| “More sessions” as the scaling description | Specify where work waits, how much can wait, who gets capacity, and when overload is rejected |
| Recording model traffic as the main trace | A run also has admission, policy decisions, denied tools, cancellations, incomplete effects, and terminal results |
| Standard-library-only as minimality | Minimality concerns how much complex behavior you must maintain, including behavior hidden in handwritten protocols |
| Replay/rewind as a route to resume | Reconstructing a decision is different from knowing whether an external effect already happened |

These are gaps in the written plan, not evidence that you cannot understand the
concepts. They are also common places for an assistant-generated plan to sound
more settled than it really is.

## What I would correct in my previous advice

I made the first project easier to start by choosing blocking networking and
pushing concurrency into an optional later phase. That was a reasonable choice
for a single local agent, but it under-serves your clarified destination.

The revised plan keeps synchronous learning exercises and adopts Tokio when
networking begins. You learn ownership first without completing a client you
would immediately replace.

I also retained a custom per-event file journal. With concurrent runs and shared
queries now explicit requirements, SQLite is a better default: the transaction
and query behavior is already implemented and documented. You still write the
event model, ownership checks, queueing, and failure policy.

Finally, canonicalize-then-open was documented as a limited local precaution.
The stronger goal justifies a capability-based filesystem wrapper from the first
tool. That still does not turn arbitrary native plugins into sandboxed code.

## General purpose does not mean unlimited authority

General purpose comes from reusable execution contracts and suitable adapters.
It does not require arbitrary shell execution, a plugin loader, or dozens of
tools in the first version.

A new tool should answer the same questions as the first two: what inputs are
valid, which capability it needs, what it can disclose/change, how its output is
bounded, what cancellation means, and whether retry is safe. A new model adapter
must preserve the same decision/outcome semantics despite a different wire format.

Do not design every future integration in advance. Design enough structure that
adding one requires a clear local contract instead of changing the meaning of
every existing run.

## Secure, low-latency scalability

Security establishes whose resources a run may use. Scalability establishes how
many runs can use shared resources without uncontrolled growth. Low latency
requires measuring where each run waits. Minimality favors a few shared mechanisms
that satisfy these requirements together.

For example, a bounded model queue gives memory control and predictable rejection.
Per-owner scheduling prevents one user from monopolizing it. A reused async client
avoids unnecessary connections. None of those mechanisms creates more GPU capacity.

Read-only tools can still disclose secrets to an inference host. Private networking
does not establish ownership of an API resource. Rust memory safety does not
authorize filesystem access. These are separate properties, and keeping them
separate makes the design easier to reason about.

Measure runtime overhead apart from model inference. Also distinguish time to
first useful text from time to a correct final result. Faster incorrect tasks are
not improved useful throughput.

## Self-check questions

Try answering before reading the explanation.

1. **The model returns schema-valid arguments for a secret file. May the handler
   run?** Only if the trusted authority and path policy permit that resource.
   Syntax validity is not permission.
2. **There is a semaphore around model calls. Is memory bounded?** Not necessarily.
   Spawned waiters, input bodies, histories, queued bytes, and subscribers can
   still grow without separate limits.
3. **A request timed out. Can you safely retry it?** A timeout may hide completed
   work. Retry needs effect-specific semantics; it is not automatically safe.
4. **A tool task was cancelled. Has the operation stopped?** Not necessarily.
   Started blocking work or remote computation may continue. Keep ownership and
   resource accounting until its actual outcome is known or explicitly unknown.
5. **The model server has four slots. Should you run forty requests at once?**
   Only with a deliberate bounded queue and evidence that the latency/throughput
   tradeoff is useful. Extra demand is not extra capacity.
6. **A user knows another run's UUID. May they read or cancel it?** No. Every
   operation still checks the verified owner and resource policy.
7. **The journal is durable. Can you resume exactly once?** No. A crash can occur
   between an external effect and recording its result. Durability protects
   records, not an atomic transaction across arbitrary outside systems.
8. **Is generality the same as dynamic tool discovery?** No. Dynamic discovery is
   one adapter feature and brings new trust problems. Two tools can already
   exercise a general action/observation contract.
9. **Does running ten agents mean they collaborate?** No. Independent concurrency
   needs scheduling; collaboration additionally needs bounded delegation,
   authority inheritance, information-sharing rules, and combined budgets.

## Separating research layers

The paper pass adds one distinction to practice: context authoring, planning,
runtime execution, and decoding acceleration are separate layers. You can borrow
a folder-based input contract without adopting a filesystem scheduler; use
feedback without requiring a written thought trace; test formal preconditions
without training a planner; and benefit from faster decoding without executing
unverified tool proposals. Ask which layer a paper changes before importing its
architecture or benchmark numbers. See the [paper review](paper-review.md).

Also distinguish an inspectable record from an explanation of why the model
produced a sentence, and a completed run from a correctly solved task. Those
distinctions make your evaluations and security claims more precise.

## My assessment

Your latest question identifies a real gap in my preceding revision. I had
described independent evaluation without carrying a verdict through the runtime.
That distinction has to appear in state, persistence, CLI/API output, and tests,
not just the prose explaining `succeeded`. The plan now uses `completed` for
execution and a separate [acceptance contract](verification.md).

The next distinction is between a correct checker and an adequate specification.
A checker can faithfully prove that a field equals a file's value while the file
is stale or the human wanted a broader judgment. General-purpose execution can
support many task-specific checks; it does not create a universal judge. For
each new task, ask what an independently observable success condition would be
and whether it covers the actual goal. If you cannot establish it, keep the
outcome explicitly unchecked or inconclusive.

The plan shows a good grasp of the broad components and why remote inference and
observability matter. Its least-developed part was the operational contract:
authority, queueing, cancellation, durable state, and overload behavior.

That is the most valuable material for you to learn next. You do not need to
start as an expert in distributed systems. Build the pure core, then add one
observable boundary at a time, and make each failure exercise prove a specific
claim. The [build guide](build-guide.md) is organized around that progression.
