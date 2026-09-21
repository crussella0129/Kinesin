# Sprint 15 diagnostic decision card

Prepared for T-122 under the [locked build plan](../sprint-plans/build-plan.md).
These are diagnostic fixtures and operator-assisted comparisons, not the full
storefront or evidence that INT-0032 is realized. No model request, server or
browser observation was performed while preparing this card. Dispatch requires
the separate source/binary/profile/first-wire freeze owned by the live operator.

## Fixed budget and preparation boundary

- At most **six top-level user-request invocations**, allocated below. An
  invocation consumes its slot once submitted, even if it fails, aborts or never
  reaches model HTTP. Internal model turns and recovery share its existing caps.
  Preparation without a submitted request is recorded separately, not scored.
- Qwen2.5-Coder-7B Q4_K_M; pinned b6500 runtime; temperature 0; context 16,384;
  output 2,400; 20 model turns; 30 tool calls; 240 seconds; history 32,768 bytes;
  tool result 8,192 bytes; two repair nudges and one completion review. Freeze
  every remaining effective profile setting and the model/runtime file hashes.
- Keep the existing baseline generic instructions and grants: list_files,
  read_file, search_files, create_directory, write_file, edit_file, run_command
  with only the existing node grant, and start_preview. No new capability or
  file access is authorized by a fixture or observation.
- Use one owned loopback runtime with a fixed served-model alias and the same
  effective seed for paired arms if supported. Record unsupported seed control.
  Fresh Kinesin sessions share that backend, not conversation or reference state.
  Root owns actual launch, configuration, source freeze and cleanup.
- Every paired arm starts in an independent empty session with identical
  initial reference state and its own copied app seed. Only the native fallback
  deliberately keeps the session from request 3 into request 4.
- Copy **only** the indicated seed directory into the granted app workspace.
  Profiles, prompts, oracles, manifests and evidence stay in disjoint private
  control/evidence directories. Never grant the whole fixtures directory.
- Archive the exact seed/prompt files and their manifest in diagnostic evidence
  before dispatch. Record actual accepted prompt bytes separately from prompt
  file bytes, including any CLI CR/LF suffix; paired submitted text must match.
  Prompt files below are UTF-8 and end with one LF. Do not silently normalize
  recorded bytes or let a prompt reader add additional instructions.
- Freeze each selected pair's exact prompts, fixtures, source/binary/profile,
  prediction and falsifier before its first dispatch. A generic implementation
  repair does not reset the budget. No model, token, temperature or prompt sweep.

## Seed identity

Lab root: `target/s15-live/fixtures/`. Initial seed manifest SHA-256:
`2307939177bf491fcffd153da5dbea79feff37c966e26aadc59b1229fc104d1e`.
The manifest includes all files below and the initially empty fallback directory;
it excludes itself. The observation template is not dispatchable evidence.

| Seed file, relative to fixtures | SHA-256 |
| --- | --- |
| order-base/data/inventory.json | f401975d50fc5a8b04505b640878b85544f681417353365b2c82d4d4e8f5cd9b |
| heldout-base/settings/shipping.json | 3cb026e20641fb7abef58339ce102d0c9336ce9ede5af1f492df6f30825449f7 |
| repair-base/index.html | b7876f7e7e6ec72d7445b6220b1adc66b7e4cc4acd60b5759b6a3cec43af1ee5 |
| repair-base/style.css | ca72c560519bb7b4b158c5b907566a7bb4bb4332d8ed3105667ad493a21f0df7 |
| repair-base/app.js | 8bfccd3e533a710807b711031661a228a443e73b9f7bdc22ffffb22c8fc062ff |

`native-fallback-base/` is empty. Initial inventory has pencil quantity 11 and
notebook quantity 4; its currency, unit prices and location are controls. Initial
shipping settings have dispatch_window `morning`; carrier, package_limit and
both label fields are controls. Preserve raw before/after bytes and hashes.
For JSON acceptance, compare decoded values to the expected object: whitespace
and property ordering alone do not fail correct contents. Extra/missing fields,
changed controls or an incorrect note fail the task. A note has exactly its
specified text and one terminal newline (LF or CRLF, recorded as actually written).

## Requests 1/2: old versus ordered action schema

Copy `order-base/` independently for both requests. Use the exact same
`prompts/pair-1.txt` text (SHA-256
`21a062de089de131b43ea3f2dc8b4e93beb291c225fec14b11cad5df4821dcc8`):

> In data/inventory.json, change the quantity of the item with id pencil to 7, leaving every other JSON value unchanged. Create change-note.txt containing one line in the form pencil: OLD -> 7, where OLD is that item's quantity before your change. End the line with a newline. Make these changes in the workspace and report what actually changed.

- Request 1: historical structured control, tuple **4/8/5/5**.
- Request 2: ordered structured candidate, tuple **4/8/6/5**.
- Before inference, materialize and inspect the actual first wire bodies and
  b6500 grammar ordering. Only action-schema property order may differ; reject
  unrelated first-body differences before dispatch. Both answer and action
  branches must remain legal. Record exact full bodies and hashes. Later
  output-dependent histories, receipts and request-derived call IDs may differ;
  do not describe those later requests as identical controlled inputs.
- Correct effects: `data/inventory.json` equals its seed with only pencil
  quantity changed from 11 to 7; `change-note.txt` is `pencil: 11 -> 7` plus
  one newline. Record reads and dispatched actions independently, but no model
  claim or parsed proposal substitutes for correct files.
- Prediction: correcting the opening discriminator order permits a useful
  action trajectory where the original ordering does not. Record first
  action/answer selection separately from eventual file effects. An ordered-only
  pass supports this narrow prediction; both/neither passing does not establish
  that order caused sprint 14's failures. No reliability claim follows one pair.

## Requests 3/4: exact qualification branches

If request 2 passes, copy `heldout-base/` independently into two fresh apps and
empty sessions. Request 3 is native **4/7/4/5**; request 4 is ordered **4/8/6/5**.
Both use `prompts/heldout.txt` (SHA-256
`87445b8a3f177f729fa5d47334a8835daf4db028cf8e421508777a6c2cf0591b`):

> In settings/shipping.json, change dispatch_window to evening, leaving every other JSON value unchanged. Create shipping-note.txt containing one line in the form dispatch_window: OLD -> evening, where OLD is the field's value before your change. End the line with a newline. Make these changes in the workspace and report what actually changed.

Correct effects: only dispatch_window changes from `morning` to `evening`;
`shipping-note.txt` is `dispatch_window: morning -> evening` plus one newline.
Prediction: an action-capable protocol handles a held-out small file task;
failure falsifies qualification for that path on this task. Select solely from
the held-out results, not an earlier easier success:

| Request 3 native | Request 4 ordered | Decision |
| --- | --- | --- |
| pass | pass | Select native; no demonstrated ordered advantage. |
| pass | fail | Select native. |
| fail | pass | Select ordered for remaining diagnostics. |
| fail | fail | Stop, even if request 2 passed. |

If request 2 fails, requests 3/4 instead use one initially empty
`native-fallback-base/` app and the **same native session**. This branch qualifies
basic effects and carry-forward; it is not a paired protocol comparison.

Request 3 uses `prompts/fallback-create.txt` (SHA-256
`cd47efd661a4ae012158d6072d1da79049ae383b8ca3eae16b865777f0170f18`):

> Create packing-list.json in this workspace as a JSON object with owner set to "demo" and an items array containing two objects in this order: id "blue-pen" with quantity 2, then id "small-pad" with quantity 3. Do not add other fields. Report what actually changed.

Request 4 uses `prompts/fallback-amend.txt` (SHA-256
`b14b8767365b6fbed4f51647ba89593ad5d2676703bd4c4c624fa4849b074dcc`):

> In that packing list, change the blue-pen quantity to 5 and keep every other value unchanged. Also create packing-note.txt containing exactly blue-pen: 2 -> 5 followed by one newline. Report what actually changed.

Both requests must pass the independently checked contents to select native.
If creation fails, stop; the remaining slots need not be spent. If amendment
fails, stop. The first request uses **4/7/4/5**; the second uses **5/7/4/5**
when the new recorded-operation reference is present. Freeze and report actual
session/reference bytes and tuples, never infer them from the expected sequence.

## Requests 5/6: one-defect repair and real feedback

Proceed only with the qualified path above. Use independent empty sessions and
identical fresh copies of `repair-base/`. Both requests use the selected native
**4/7/4/5** or ordered **4/8/6/5** path with no prior references.

The seed is a two-product static app with external `style.css` and `app.js`,
relative asset links and addEventListener handlers. It contains no checkout,
persistence or full-storefront solution. Its single seeded JavaScript defect
assigns the latest product's price to the total instead of accumulating it.
Do not expose this source diagnosis or a patch as feedback to the model.

Before either dispatch, root/evidence owner must independently operate an
unmodified seed copy, bind observations to its actual file hashes and record the
observer/version and UTC. Expected observations below are an oracle, **not
claimed browser results**:

1. Initially Copper Pencil costs $2.00, Field Notebook costs $5.00, item count is
   0 and total is $0.00.
2. Add Copper Pencil, then Field Notebook. The seeded defect is expected to show
   2 items and both names, but total $5.00 rather than the correct $7.00.
3. Search `Notebook`: only Field Notebook remains visible. Clear search: both
   products return. This is the unaffected control.

If the defect or control is not actually observed, record the preparation
failure and stop/review the fixture; do not fabricate the feedback condition.
Preparation does not consume a request slot until a request is submitted.

Both arms use `prompts/repair-base.txt` (SHA-256
`f8618671b422710147b38008511c17446e999c40e91a22976100e454842e08c6`):

> Repair the cart total calculation in this existing small app. The total should include the price of every added item. Keep the existing product catalog and search filtering working, and avoid unrelated features or a rewrite. Start it locally and report the actual preview URL and what you changed.

Request 5 receives only that base prompt. Request 6 receives the identical base
text followed by the completed authentic observation prepared from
`prompts/repair-observation-template.md`; freeze the final combined text and
its exact separator before either arm runs. The observation is the sole added
input. It may report observed steps/results and arithmetic, but no replacement
code, implementation diagnosis or tool-forcing instruction. It is explicitly
diagnostic operator assistance, not the zero-correction acceptance workload.

After each terminal request, independently inspect actual edits and the actual
returned owned preview. With a fresh page state, add Pencil then Notebook: total
must be $7.00 with 2 items and both names; add Pencil again: $9.00 with 3 items.
Search `Notebook`, clear, then search `Pencil`; filtering and both catalog prices
must remain correct. An unaffected-control regression fails repair. Missing
preview/observation is unqualified, not a pass. Record current file hashes,
browser observations and preview cleanup for both arms.

| Unaided request 5 | Assisted request 6 | Decision |
| --- | --- | --- |
| pass | pass or fail | Unaided repair qualifies; T-123 may run. Preserve any assisted regression. |
| fail | pass | Feedback dependency supported; stop and replan a scoped observation capability. |
| fail | fail | Stop; neither feedback availability nor the current repair path establishes usefulness. |

No automatic browser subsystem is implemented under this card. These sampled
contrasts support/falsify only their stated predictions; unavailable or
incomparable observations remain inconclusive and cannot unlock T-123.

## Evidence and exit

For every used slot retain: slot/branch, exact submitted prompt, fixture hashes,
full effective profile and capture tuple, source/binary/model/runtime identity,
first prepared request and later request hashes, run ID, actual tool receipts,
terminal reason, file bytes/hashes, observer-linked results, prediction/falsifier
and disposition, request/model elapsed time, token counts when available,
automatic retries, operator effort and interventions. Missing measures stay
unknown. Record unused slots with the stop decision instead of silently omitting
them. Root owns the append-only attempt ledger and actual freeze timestamps.

No assisted repair, truthful status or tiny-task pass satisfies INT-0032. Only
qualified real actions **and unaided observed repair** permit the separately
frozen unchanged storefront/follow-up workload, with at most two attempts under
T-123's original zero-correction rules. Official unit/integration checks remain
deferred until that full live gate passes. Exhaustion or an unmet prerequisite
ends in an explicit failed/inconclusive decision, without replacement slots.
