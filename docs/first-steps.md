# Where to begin

The complete instructions are in the [build guide](build-guide.md).
Start there at step 1. This page is an orientation, not a second implementation
sequence to reconcile.

You will write the Rust yourself. This repository supplies behavioral contracts,
a learning sequence, reading, and checks; it does not contain a completed harness.
Each guide step tells you what to build, how to prove it, a failure to exercise,
and what to read when the concept becomes useful.

## Your first sitting

1. Write a short description of the personal and shared operating profiles.
   The destination is concurrent agents for you first, a shared service later.
2. Verify Rust/Cargo and use a separate scratch project to practice moves,
   borrowing, enums, and `Result`. Follow the exact commands in guide step 2.
3. Initialize one package only when you reach step 3. Keep its first build tiny:
   a binary calls your library, and the checkpoint commands pass.

You can learn the pure core and use a scripted model while a live model is
unavailable. Manual provider preflight is a separate gate before live networking
and then before real tool calls. A particular GPU is not a prerequisite for
learning or testing the controller.

## What you will build

| Checkpoint | Guide steps | Observable result |
|------------|-------------|-------------------|
| First live turn | 1–14 | A bounded, durably recorded exchange with explicitly unchecked acceptance |
| Useful local agent | 15–22 | Two read-only tools and an independently checked file-field task with a durable receipt |
| Concurrent local harness | 23–31 | Isolated runs share bounded resources; receipts, replay, and streaming work |
| Deployment options | 32–33 | Private remote inference; optional owned server process |
| Shared service | 34–41 | Owner-authorized results/receipts, quotas, fair scheduling, recovery, and operational gates |

A completed checkpoint is useful even if the next one takes weeks. Treat the
step numbers as dependency order, not as a promise of how many evenings this takes.
Step 21 is a separate lesson in [task acceptance](verification.md): the model
supplies a candidate, and a small pure Rust checker decides whether its configured
file-field contract passed. General freeform answers remain unchecked.

## How to use the documents

Read the [architecture](architecture.md) once for the overall model. Keep the
[roadmap](roadmap.md) as your implementation checklist. Open the specific contract
linked from each guide step when you implement that boundary.

The [process](process.md) explains the edit/check/commit rhythm.
[Resources](resources.md) indexes the readings. [Understanding](understanding.md)
gives feedback on your current model of harnesses and questions you can use to
check what you have learned.

When a Rust concept is unfamiliar, reproduce it in scratch code before mixing it
with the whole agent loop. When a design feels too large, finish the current
proof before introducing another module or dependency.
