# Sprint 16 frozen diagnostic decision

Approved canonical plan locked before execution. Maximum four submitted requests
shared across T-125/T-127; failed/transport/cancelled submissions consume slots.
No replacements. Full-workload allowance remains separately gated and unused.

## Frozen inputs and identity

[Preparation manifest](../sprint-tests/diagnostics/preparation/manifest.json)
binds all five branch prompt files, both complete seed fixtures, observed names
and source, profile, source, binary, runtime and model. All conditional inputs
were frozen at 2026-09-21T01:51:42.862919+00:00, before any inference. Runtime
model/seed/alias and limits match Sprint 15. Source remains `ad058c5`; no T-126
implementation is authorized unless A fails and B passes. Each submitted call
gets an additional immutable profile/prompt/seed/binary/first-wire freeze.

Inputs use one physical UTF-8 line plus one LF; full source is encoded inside
JSON strings with escaped line endings, preserving complete contents and hashes.
These are direct operator observations, not fabricated harness tool events.
Root observations cap at 2,048 bytes; complete source observations at 16,384;
actual inputs fit those caps. No snippets, diagnosis or imperative to use tools
is appended. Held-out inputs stay outside earlier workspace grants.

## Prediction, falsifier and decision

- A: normal original request. Prediction: answer-only failure may persist;
  falsifier: actual changed app, owned returned preview and all browser criteria.
  Pass sends slot 2 to normal held-out and skips B/C/T-126.
- B, only after A fails: same task/seed with authentic root names. Prediction:
  if missing path salience is sufficient on this sample, inspection and useful
  repair will occur. Answer-only, wrong effects, missing preview or failed
  behavior falsifies sufficiency. Pass authorizes T-126, then normal original
  and held-out repairs in slots 3/4; stop if slot 3 fails.
- C, only after B fails: same B plus all three unmodified source files.
  Prediction: supplied source may enable correct repair even when discovery
  fails. Missing real repair/preview/control pass falsifies that prediction.
  Pass permits only source-assisted held-out slot 4; either outcome stops with
  findings. C never authorizes T-126 or full-workload acceptance.

Framing, salience, observation bytes and inference/cache behavior can influence
contrasts. These few adaptive samples do not identify a unique cause or prove
reliability. Model prose, CLI success or file writes alone never pass.

## Independent browser baseline and scoring

Observer: Codex operator through in-app browser tab 9; owned preparation server
`http://127.0.0.1:61458/` serving frozen fixture copies. These are seed
observations, not model previews or acceptance runs. Source hashes are in the
manifest. No browser feedback is included in normal prompts.

Original, 2026-09-21 01:48:40–01:49 UTC: Copper Pencil $2 and Field Notebook $5
initially show count 0/total $0. Add each: observed count 2/total $5, expected
$7. Add another pencil: count 3/total $2, expected $9. Searches `Copper` and
`Notebook` show only the corresponding catalog product with correct price;
clearing restores both. Cart rows and count survive filtering. Repair must
produce $7 then $9 and preserve these controls.

Held-out, 01:49:57–01:50:14 UTC: Grid Pad $4 and Paper Tape $3 initially empty.
Add Pad twice and Tape once: count 3/$11, Pad ×2 and Tape ×1. Remove Grid Pad:
observed Pad ×1/Tape ×1, count 2/$7; expected only Tape ×1/count 1/$3. Search
`Tape` shows only Tape at $3; clear restores Pad $4 and Tape $3. Add Tape again:
observed count 3/$10. Remove the single remaining Pad: only Tape ×2/count 2/$6.
One stale browser element reference was refreshed; it did not act or alter
source/model input. For repaired fresh copies, follow the same first removal
then add Tape: only Tape ×2/count 2/$6; add one Pad then remove that row:
only Tape ×2/count 2/$6. Search/clear and catalog prices must stay correct.

Every candidate requires actual authorized operations, a returned owned preview
usable while its session lives, fresh browser criteria and session shutdown.
No returned preview means fail/unobserved browser outcome, not a substitute
operator server. Diff/hash observations are independent of model claims.

## Effort and live order

Preparation uses ignored disposable `target/s16-live`, reusable S15 no-inference
request preparation and loopback wire relay. Lab preparation, seed creation and
browser operation are operator effort, not harness autonomy. Active human time
is not measured. Calls receive no corrections, manual patches, tool forcing or
context resets; B/C assistance is disclosed upfront. Official unit/integration
tests and Clippy remain deferred until the unchanged full workload passes.

## Slot ledger

No submitted requests at freeze. Per-call `started.json` is create-exclusive.

| Slot | Frozen branch | Result | Actual operations | Request seconds |
| --- | --- | --- | --- | --- |
| 1 | A, no assistance | FAIL | 2 model turns, 0 tools, no changes or preview | 2.281 |
| 2 | B, root filenames | FAIL | 2 model turns, 0 tools, no changes or preview | 2.328 |
| 3 | C, filenames and complete source | FAIL | 2 model turns, 0 tools, no changes or preview | 2.844 |
| 4 | Conditional held-out | NOT RUN | C failed; decision tree stops | — |

All three actual first wire hashes matched their pre-inference expected bytes.
Every terminal file hash equals the original seed. All sessions closed and
captures exported. See [attempt ledger](../sprint-tests/diagnostics/attempt-ledger.md)
for per-call evidence. No T-126 implementation or normal held-out qualification;
zero full-workload submissions. No new budget is implied by the unused slot.
