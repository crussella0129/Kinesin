# Sprint 15 full live workload card

Prepared for T-123 under the [locked build plan](../sprint-plans/build-plan.md).
This preserves the [original Sprint 14 workload](../../s14/sprint-research/live-workload.md),
not the smaller diagnostic fixtures.

**Status: NOT RUN — repair qualification failed.** Unaided diagnostic request 5
and assisted request 6 both returned answers with zero tool calls, unchanged app
bytes and no preview. The required unaided observed repair was not demonstrated.
**Zero T-123 full-workload attempts were consumed.** This card is retained as the
blocked planned workload, not success evidence or authorization for another run.
Official unit/integration verification remains deferred; no task completion is
claimed. A new qualifying path requires an explicit research/plan decision.

The preserved protocol below requires both action qualification and independently
observed unaided repair with its unaffected control preserved before dispatch.
Assisted-only repair would not unlock it either. These prerequisites are unmet.

## Exact original task messages

Each block below is one physical line. Use the original frozen prompt bytes,
not Markdown markers or the wrapped display text from another report.

Initial request:

```text
Build a small local stationery storefront called Paper Harbor. Include six products with names, prices and categories; search and category filtering; a cart with quantities, remove controls and a correct total; and a demo checkout collecting a name and email and showing an order confirmation. Keep the cart after a page reload. Use synthetic data, no accounts or real payments, and no external dependencies. Start it locally and give me the preview URL.
```

Natural follow-up, sent only after the initial browser gate passes:

```text
Add a Clear Cart button and show the total number of items in the cart. Keep the existing catalog, quantity controls and checkout working.
```

The original files remain in
`target/s14-live/attempt-1/evidence/initial-prompt.txt` and
`target/s14-live/attempt-1/evidence/follow_up-prompt.txt`. Their verified frozen
contents contain no CR or LF and have these identities:

| Message | UTF-8 bytes before terminal delimiter | SHA-256 |
| --- | ---: | --- |
| Initial | 454 | 04d6f1a9c2af917728286de972d6e6119e48fb3fd37fc51fc6b1e9c083dbd1b1 |
| Follow-up | 138 | 2b643ab793fcb9f9ca416d06161fe0e8a75c29f04ed8f810a2f918cfc260d85d |

Archive copies with the new attempt before dispatch. If the ignored lab files
are unavailable, recover the exact retained prompt from the archived Sprint 14
records and verify these identities; do not rephrase the task.

## CLI input and newline provenance

Read-only inspection of `src/cli.rs::read_session_line` confirms that interactive
input reads through the first LF, retains the delimiter, and limits a session
entry to 16 KiB. It does not join multiline paragraphs into one request.

Send exactly one complete prompt line and one terminal line delimiter, then
wait for that request and the independent gate. Do not paste wrapped lines or
send the follow-up in the same write as the initial request. Keep stdin/session
alive between requests; closing and restarting would lose the required session.

An LF-delimited submission has 455/139 bytes; CRLF has 456/140. Record actual
accepted prompt bytes and hashes from the journal, including the exact suffix.
Report raw-byte equality separately from equality after removing only the
submission's trailing CR/LF. Do not trim internal whitespace or silently rewrite
the retained bytes. Sprint 14's
[prompt audit](../../s14/sprint-tests/attempts/prompt-byte-audit.json) preserves
this distinction: its later initial submissions had CRLF while frozen files did
not. A delimiter is not an extra task message; an embedded newline can be.

## Conditional profile and pre-dispatch freeze

The prospective path is the ordered candidate selected by the small-task
diagnostics, **conditional on the unaided repair gate**. The actual qualifying
decision and observer evidence must be linked before dispatch; this card is not
that evidence. Do not substitute another profile if qualification fails.

- Model: Qwen2.5-Coder-7B Q4_K_M, pinned b6500 runtime, temperature 0 and the
  selected diagnostic sampling/seed settings. Freeze the actual model/runtime
  hashes, served alias, all effective settings and runtime ownership/lifecycle.
- Mode: explicit `structured_ordered_v1`, core 8 / adapter 6 / tools 5. The
  initial empty-reference request uses capture tuple **4/8/6/5**; a follow-up
  carrying the new recorded-operation reference uses **5/8/6/5**. Record the
  actual tuple and frozen reference input for each request. Native remains the
  product default; this workload does not authorize promotion.
- Original ceilings for **each request**: context 16,384 tokens; output 2,400;
  20 model turns; 30 tool calls; 240 seconds; history 32,768 bytes; tool results
  8,192 bytes. Two automatic repair nudges and one separate completion review
  must fit these bounds. Preserve all other effective selected-profile limits.
- Preserve the generic baseline workspace-assistant instruction and original
  grants, including only the existing node command grant. No storefront-specific
  system coaching, new capability, larger limit or dependency is added.
- Freeze binary/source/dependency identities, full profile, exact prompt files,
  actual initial workspace contents and the qualifying diagnostic record before
  each attempt. The app starts empty; private control/evidence files stay outside
  its grant. Copy no diagnostic seed, generated storefront or solution into it.
- Use independent fresh browser state for each attempt. These are disposable
  scoped workspaces; they are not claimed to provide Windows OS containment.

## Unchanged independent acceptance criteria

1. The preview URL returned by an actual tool serves the new workspace files;
   a browser shows six products and usable search/category filtering.
2. Add at least two products, increase/decrease quantities and remove an item;
   totals match the displayed prices and the remaining cart survives reload.
3. Valid synthetic name/email checkout shows the correct amount and empties
   the cart; empty cart and invalid email cannot produce a valid order.
4. After the natural follow-up, the item count equals summed quantities; Clear
   Cart removes all items, resets the count/total and remains empty after reload.
   Catalog, quantity controls and checkout still work.
5. Preview remains available between turns and closes with the owning session.

Use actual browser interactions and HTTP observations, not model completion
claims, a successful write, static CSP warnings, a live URL alone or visible
control labels. Compute totals independently from the displayed product prices.
Use only synthetic checkout details. Preserve the initial product/catalog/cart
observations before checking that the follow-up kept them working.

Before the follow-up, independently pass criteria 1–3 and observe continued
preview availability. Send the exact follow-up in the **same Kinesin session**,
without clearing its reference history, restarting the process or injecting
browser feedback. Complete criterion 4 and repeat affected original behaviors.
Finally close the owning session and observe that its preview no longer serves.
Do not send the follow-up after an initial failure.

## Attempt and intervention rules

- At most **two** fresh full-workload attempts. The initial and conditional
  follow-up together form one attempt. Retain failures, cancellations and missing
  observations; do not replace them with an unrecorded retry.
- An accepted attempt has zero corrective operator messages, code patches,
  exact-edit instructions, named-tool imperatives, context resets or manual app
  repairs. The initial task and planned natural follow-up are its only task
  messages. Normal harness recovery within its frozen budgets is recorded.
- Independent observation does not authorize app editing or extra model
  requests. Do not feed observed failures back during a scored attempt. CLI
  lifecycle controls used to close the attempt are recorded separately.
- The optional second attempt requires a stated generic harness correction to
  a recorded defect, within approved scope, with new source/binary identity and
  a new empty workspace. Do not change the criterion, profile, prompt or ceiling.
  A necessary out-of-scope capability requires research/plan, not an implicit
  extension. No second attempt is required after a pass.
- If either qualification prerequisite is unmet, no full attempt starts. If the
  full-attempt budget is exhausted without a pass, record failed/inconclusive
  usefulness. Do not begin official unit/integration checks or treat diagnostic
  infrastructure as delivered app competence.

## Outcome evidence

For each request retain exact submitted bytes, run ID, capture tuple, source and
profile identities, actual effects, terminal reason, automatic retries, UTC
start/end, model/request time, available token counts, operator effort and every
manual intervention. Preserve file hashes and browser/HTTP observations before
and after the follow-up, with observer identity and criterion IDs.

Bind each behavior observation to the relevant artifact revision and actual
observation record. A relevant file change invalidates its earlier observation
until observed again. Report lost passes as regressions and gains separately;
missing, stale or incomparable observations remain unknown. An unchanged result
requires comparable observations and evidence the attempted action actually ran.
Changed bytes, truthful stopping or a response lifecycle of completed do not
establish forward progress.

Keep all eight Sprint 14 failures and the six-slot diagnostic ledger available.
Report this fixed workload without claiming general reliability or a measured
advantage over human authoring. Only a complete independent pass of the initial
app, same-session follow-up and preview lifecycle opens T-124's official focused
verification gate. No task-ledger completion is asserted by this card.
