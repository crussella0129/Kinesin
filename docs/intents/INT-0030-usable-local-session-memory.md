# INT-0030 — Usable local sessions with bounded conversation memory

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0030
- **State:** active
- **Work evidence:** [sprint 12 plan](../sprints/s12/sprint-plans/build-plan.md)
- **Completion evidence:** none
- **Code evidence:** pending
- **Test evidence:** pending; implementation first at the user's request
- **Documentation evidence:** [CLI](../cli.md)

## Intent
Make the existing local assistant useful across successive requests: launch it,
remember recent user requests and answers, and create/edit workspace files in
follow-ups. Deliver a bounded in-process conversation now. INT-0026 retains
responsibility for persistent/branched sessions, real model token admission,
cache measurements and mixed-owner slot experiments.

## Acceptance criteria
- Recent successful turns retain both user text and final model answers with
  run origin. History is bounded by bytes and turn count; truncation/eviction
  is explicit. A failed or oversized turn cannot poison subsequent requests.
- Every request receives fresh authority for the selected workspace and model.
  History carries no tool grants or checked evidence. Metadata capture does not
  persist conversation content, and opt-in replay reproduces the frozen input.
- `/new` and `/clear` erase the live history; `/context` reports its size and
  limitations. File creation and subsequent edits work through existing grants.
- Native Windows build and a real local model conversation demonstrate launch,
  remembered details and a file edit after implementation. Targeted regression
  tests, formatting and Clippy follow implementation, not precede it.

## Rationale
The user requested a usable local assistant quickly: launch, remember context,
edit files, with testing afterward. The launcher and file tools already exist,
but each follow-up currently receives only the immediately preceding answer.

## Alternatives
Overload the old prior-answer field with a transcript (mislabels provenance);
persist every turn by default (changes capture/privacy policy); implement all
of INT-0026 before improving interactive use (unnecessary scope for this pass).

## Consequences
Use a distinct frozen session context with bounded recent prompt/answer pairs.
Large entries may be explicitly clipped and oldest turns evicted. Tool results
are not retained between runs; the model must re-read files before editing.
Memory lasts only for the running session. Byte admission is not a tokenizer
or proof that all arbitrary inputs fit a backend token window. Keep the wider
INT-0026 criteria open.

## Transition history
- 2026-09-20: created as planned for the user's implementation-first local
  assistant request, with scope confirmed as launch, context and file editing.
- 2026-09-20: planned → active after independent plan review and helper finalization.
