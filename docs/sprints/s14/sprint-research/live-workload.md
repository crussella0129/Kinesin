# Sprint 14 live workload card

Status: prompts and resource ceilings agreed for plan review; evidence owner
will record the frozen profile identity before the first scored dispatch.
This is a normal operator request, not an app-specific harness instruction.

## Initial request
> Build a small local stationery storefront called Paper Harbor. Include six
> products with names, prices and categories; search and category filtering; a
> cart with quantities, remove controls and a correct total; and a demo checkout
> collecting a name and email and showing an order confirmation. Keep the cart
> after a page reload. Use synthetic data, no accounts or real payments, and no
> external dependencies. Start it locally and give me the preview URL.

## Planned natural follow-up
> Add a Clear Cart button and show the total number of items in the cart. Keep
> the existing catalog, quantity controls and checkout working.

## Independent checks
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

## Attempt protocol
- Use a new empty isolated app workspace and disjoint private control/evidence
  directories; record initial contents and final file hashes.
- Freeze model/profile and resource ceilings before each scored attempt. Record
  changes between attempts explicitly; never silently rerun with larger limits.
- Send exactly the initial request and the planned follow-up in the same
  session. Do not provide code,
  exact edits, named-tool imperatives, corrective prompts or /new context resets.
- Browser inspection establishes results but does not authorize editing the app.
  A failed behavior fails that attempt; retain it before a generic product repair
  and a clearly labeled new empty-workspace attempt.
- Record start/end UTC, model wall time, active operator minutes, run IDs,
  automatic recovery events and each manual intervention (target: zero).
- Report all attempts and the accepted attempt, without claiming an unmeasured
  comparison to human implementation time or broad model competence.

## Frozen resource ceilings
Use the actual local model with context 16,384 tokens, output 2,400 tokens,
20 model turns, 30 tool calls and 240 seconds per request; history 32,768 bytes
and tool results 8,192 bytes. Record remaining effective profile fields, model
identity and source hashes before dispatch. Two automatic repair nudges and one
separate completion review must fit these bounds. System instructions stay a
generic workspace-assistant instruction, with no storefront-specific coaching.
