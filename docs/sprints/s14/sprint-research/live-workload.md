# Sprint 14 live workload card

Status: prompts and resource ceilings frozen before attempt 1 dispatch at
2026-09-20T06:04:07.273330Z. The [attempt freeze](../sprint-tests/attempts/attempt-1-freeze.json)
records the empty initial app, exact prompts, effective profile, binary and
source hashes. Further attempts must retain a separate pre-dispatch freeze.
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

## Attempt 5: separately frozen model-profile comparison

The original baseline card and attempts 1–4 remain unchanged; all four attempts
with that 7B model profile failed. Attempt 5 uses the same core-7 source/binary
as attempt 4, the two exact user prompts above, resource ceilings and browser
criteria, in a fresh empty workspace. No adapter rewrite or app-specific system
instruction is part of this comparison.

The selected comparison profile is official Qwen3-8B Q4_K_M at temperature 0.7
with generic `/no_think` profile instructions. Its pinned model revision and
expected artifact hash are recorded in the
[profile-comparison provenance](implementation-adjustment.md#model-profile-comparison-after-failed-attempt-4).
Before dispatch, verify that hash and separately freeze the complete effective
profile, model/runtime, source and binary identities. Sampling/mode differences
are explicit profile changes; do not describe a better outcome as an isolated
harness improvement or erase the four baseline failures. The first-action JSON
probe executed no action and remains outside scored attempts and official tests.

## Attempt 6: separately frozen reasoning/output comparison

Attempt 5 failed independent browser checks: inert Add to Cart and absent search.
For attempt 6, retain the same core-7 source/binary and Qwen3 weights, remove
`/no_think`, set temperature 0.6 and explicitly raise output to 4,096 tokens per
response (previously 2,400). This allows reasoning plus action within the
unchanged 16,384-token context, 20-turn, 30-tool, 240-second, 32,768-byte history
and 8,192-byte tool-result ceilings. The repair/review ceilings also remain fixed.

Use a fresh empty app and freeze all effective settings before dispatch. The
two user prompts, independent feature checks and zero corrective operator
interventions remain exactly as above. Preserve the original baseline and all
five failed attempts. Any changed outcome reflects a profile/output-budget
comparison; it cannot isolate harness improvement. No production default or
adapter change is part of this attempt, and official tests remain deferred.

## Attempt 7: original profile with structured single-action execution

Return to the original Qwen2.5 7B baseline profile, temperature 0 and 2,400 output
tokens, restoring all original ceilings and the exact two user prompts. Use a
new empty app workspace and freeze core-8/adapter-5/tools-5 source/binary identity
and the effective profile before dispatch. The new harness strategy is the
[declared structured single-action protocol](implementation-adjustment.md#attempt-7-bounded-structured-single-action-protocol),
with normal authorization and no mandatory planning stages. No other model-profile
change or extra operator message is part of this attempt.

All six earlier failures and unscored probes remain visible. The one-action JSON
probe is not success evidence. A schema-valid reply or additional real tool calls
cannot satisfy the catalog, search, cart, checkout or follow-up criteria above;
the same independent browser observations and zero corrective interventions are
required. This comparison with attempt 4 does not establish broad model ability
or a measured productivity gain. Official tests still follow live confidence.

## Attempt 8: counterpart profile with unchanged structured execution

Retain attempt 7's failed zero-action result. Keep its core-8/adapter-5/tools-5
source and binary unchanged, but restore attempt 5's official Qwen3-8B profile:
temperature 0.7, generic `/no_think` and 2,400 output tokens. All other original
resource ceilings, exact two user prompts, required behavior and zero corrective
operator interventions remain unchanged. Freeze the effective profile and
unchanged source/binary identities before dispatch in a new empty app workspace.

This single counterpart comparison with attempt 5 tests the protocol strategy
under matching model settings; valid JSON and tool counts do not satisfy any
functional criterion. Preserve every failed attempt, send the natural follow-up
only after the initial browser gate passes, and claim no general reliability or
production-default change. If it fails, further design work must focus on actual
observation and repair rather than blind prompt variations. Official tests remain
after live confidence.
