# Sprint 14 live evidence

The [workload card](../sprint-research/live-workload.md) defines the two natural
requests and independent functional gate. Official unit/integration checks
remain deferred until this gate passes.

The [prompt-byte audit](attempts/prompt-byte-audit.json) distinguishes wording
from transport bytes. Attempt 1's two submitted prompts exactly match their
frozen bytes. Attempts 2–8 each submitted the same 454-byte initial text followed
by terminal-added CRLF, totaling 456 bytes. Their wording matches only under the
predeclared trailing-CR/LF normalization rule; raw byte identity is false. Exact
actual prompt strings remain in each journal record. No failures were rewritten.

## Observed progress across retained attempts

This compares actual outcomes, not inferred model competence. `Unverified` means
the operation was not reached or could not be exercised; it is not a pass.

| Frozen criterion | Attempt 1 | Attempt 2 | Observed change |
| --- | --- | --- | --- |
| Reachable preview with six visible products | Pass | No preview started | Backward: usable page availability lost. |
| Search and category controls | Fail: absent | Unverified without preview; scaffold lacks controls | No demonstrated improvement. |
| Add/quantity/remove/correct cart totals | Fail: Add inert | Unverified without preview | No demonstrated improvement. |
| Reload preserves cart | Unverified | Unverified | No demonstrated change. |
| Valid checkout and invalid/empty rejection | Unverified | Unverified | No demonstrated change. |
| Follow-up Clear Cart/item count | Fail: no file changes or controls | Not submitted after initial failure | No demonstrated improvement. |
| Preview lifetime and shutdown | Pass across requests; refused after exit | No preview started | Unverified in attempt 2. |
| Honest completion handling (diagnostic, not functional gate) | Unsupported feature/change claims admitted | Stopped with no candidate | Forward on honesty; app still failed. |

| Frozen criterion | Attempt 3 | Change against prior observations |
| --- | --- | --- |
| Reachable local preview | Pass: actual page loaded | Forward from attempt 2; returns to attempt 1 availability. |
| Six visible products | Fail: empty Products heading | Backward from attempt 1's six visible products. |
| Search and category controls | Fail: absent | No improvement over attempt 1. |
| Add/quantity/remove/correct cart totals | Fail: empty Cart heading, no controls | No demonstrated working cart. |
| Reload persistence and checkout validity | Unverified: no usable cart | No demonstrated improvement. |
| Follow-up Clear Cart/item count | Not submitted after initial failure | No demonstrated improvement. |
| Shutdown | Pass: port 57367 refused after session exit | Returns to attempt 1's observed shutdown behavior. |
| Honest completion handling (diagnostic) | No candidate; stopped on run/preview kind mismatch despite actual preview | No functional advance; exposed a workflow false stop. |

Both attempts used zero corrective operator prompts and remain failed. Attempt 2
cannot be called an overall improvement: it lost the reachable preview while
exposing and stopping on a distinct model/workflow failure.

Attempt 4 regressed again to no app files and no preview. All browser criteria
are unverified because no page could be opened; absence of a usable app fails the
initial gate. The static-warning repair branch was never reached. Its admitted
final answer delegated file creation back to the operator, so removal of rigid
milestones did not demonstrate improved task completion.

Attempt 5 restored a real preview and six visible products relative to attempt 4,
returning to attempt 1's limited visible result. Search/category controls remained
absent and Add to Cart remained inert. There was no demonstrated improvement in
cart, persistence, checkout or follow-up behavior. Warning feedback reached the
new model, but it dismissed the warning's functional impact instead of repairing
the app. This is a failed model-profile comparison, not a harness-source gain.

| Frozen functional criterion | Attempt 6 | Change from attempt 5 |
| --- | --- | --- |
| Reachable preview | Pass: page loaded | No change in availability. |
| Six visible products | Fail: zero products | Backward from six visible products. |
| Search/category filtering | Controls present; clicking All and entering Notebook only changed input values, with no products rendered | Control presence improved; functional filtering was not established. |
| Cart add/quantity/remove/totals | Cart Total 0 visible, but no products to operate | No demonstrated working cart. |
| Persistence and valid/invalid checkout | Unverified without a usable cart | No demonstrated improvement. |
| Natural follow-up | Not submitted after initial failure | No demonstrated improvement. |
| Session-owned shutdown | Pass: exit 0 and port 64139 refused | No change in observed shutdown behavior. |
| Completion accuracy (diagnostic) | Claimed external files and CSP repairs that were never written | No improvement in trustworthy completion. |

All six scored attempts remain failed. Attempt 6 spent 192.942 seconds versus
attempt 5's 29.836 seconds and lost product rendering; neither demonstrated a
usable cart or the complete workload. Separate capability/protocol probes do not
change these scored outcomes.

Attempt 7 likewise failed: no files, no tool actions and no preview. It made no
functional advance over attempt 4 and regressed app availability relative to
attempts 5 and 6. Browser behavior and the follow-up could not be exercised.
Schema-valid answers alone did not cause execution. All seven scored attempts
remain failed; source compilation and protocol validity do not change that.

Attempt 8 also produced zero tools, files or preview. Compared with attempt 5's
same-profile native-tool run, it lost file creation and six-product rendering.
Search/cart/checkout/persistence and follow-up behavior were not exercised. Both
structured-protocol trials therefore failed to produce any actual action; valid
answer envelopes were not progress. **All eight scored attempts failed** the
independent live gate. No further blind prompt-variation attempt is recorded.

## Attempt 1

- Frozen before dispatch: `2026-09-20T06:04:07.273330Z`; [immutable attempt identity](attempts/attempt-1-freeze.json).
- Empty application directory: `target/s14-live/attempt-1/app`; separate sibling
  control/profile/state and evidence directories. No outer application writes.
- Local Qwen2.5-Coder 7B Q4_K_M and the same selected runtime as Sprint 13;
  generic onboarding instructions, explicit Node/file/preview grants, and the
  card's 16,384-context / 2,400-output / 20-turn / 30-tool / 240-second ceilings.
- Live binary SHA-256: `D4F8770ED1D9D5685752E87BBBAA92665425FFCCFDF928EEFDF86C7C5331F331`.
  The freeze records eight changed source-file hashes against baseline
  `1a6c24684510f61b25f48074375d5bb74e4bc210`.
- Evidence-agent preparation/freeze effort: approximately three active minutes,
  estimated independently of primary operator prompting/browser time. No
  human-authoring baseline was measured.
- Outcome: **failed**, despite zero corrective messages. The browser found six
  visible products, absent search/category controls and inert Add to Cart. The
  follow-up still produced no Clear Cart control or item count. Source inspection
  found inline handlers/styles blocked by CSP, a missing total-element reference,
  reassignment of a constant cart and no requested persistence/quantity behavior.

| Request | Run ID | Journal time (UTC) | Actual execution |
| --- | --- | --- | --- |
| Initial | `97241cfd-8824-424b-95ee-9b2797a958ec` | 06:04:35.289–06:04:58.427 | Three model turns; four tools: directory, two file writes and successful preview. One completion review, no repair nudge. |
| Natural follow-up | `a96761d6-565a-48e3-9171-1301c45f9b34` | 06:06:55.385–06:07:11.466 | Three model turns; only `start_preview .`, denied with `preview_already_running`. No file edits. One completion review, no repair nudge. |

Both journal prompts match the frozen texts. Total recorded request time was
39.219 seconds, including 39.041 seconds summed model exchange time; six model
turns used 15,805 prompt tokens and 2,858 completion tokens. The gap between
requests includes browser inspection and is not model latency. There were two
operator task messages, zero corrective prompts, zero manual app edits, zero
tool-forcing prompts and zero context resets. The primary operator's final
consolidated active setup/browser/close estimate was approximately four minutes,
separate from the evidence-agent preparation estimate. Post-attempt analysis was
not instrumented, and no human-authoring speedup is claimed.

Initial `model_finished` sequence 13 recorded the completion review; sequence 15
claimed requested functionality without another tool call. The follow-up likewise
claimed changes after one failed preview call and no writes. Both runs remain
`completed / unchecked` in the journal, which is not functional acceptance.
[Compact journal records and artifact hashes](attempts/attempt-1-runs.json) retain
these facts and the actual unsupported prose. The [failed generated files](attempts/attempt-1-app/paper_harbor/index.html)
are retained unchanged; final file hashes equal the initial snapshot hashes.

The operator exited native session 6182 at approximately 06:11 UTC and confirmed
the owned preview on port 57197 actively refused HTTP connections afterward.
The live confidence gate remains failed, so official tests remain deferred.

## Attempt 2

A new empty app and separate control/evidence directories are prepared at
`target/s14-live/attempt-2`. The profile bytes, natural prompts and resource
ceilings are unchanged from attempt 1. The bounded, versioned milestone
[implementation refinement](../sprint-research/implementation-adjustment.md) is
the recorded product change. [Attempt identity](attempts/attempt-2-freeze.json)
was frozen before dispatch at `2026-09-20T06:16:17.090666Z`; the binary SHA-256
is `5B952EEF95D946C5F998D9E53AF5C7891FC48F4EC38B01098C745E63DB982758`.
The source record includes nine Rust files plus Cargo.lock against baseline
`41c6925ebd31f948b447e21198df2b7ed885fb28`, which also incorporates the separate
PR13 storage repair and rustls 0.23.45 update. Those concurrent baseline changes
are disclosed rather than attributing every difference solely to milestones.
Evidence-agent preparation/freeze effort is approximately two active minutes.
No model was launched by the evidence agent. The initial request matched the
frozen prompt and ran as `de003881-f1e5-49bd-9187-8d9faae9e6dd` from
06:16:45.962 to 06:17:10.187 UTC. It **failed** before starting a preview:
`stopped / workflow_recovery_exhausted`, with no candidate admitted. There were
14 model turns, seven actual tools, two repair nudges and no completion review.
Recorded request time was 24.225 seconds, including 23.999 seconds of model
exchanges. Successful responses reported 33,880 prompt and 1,754 completion
tokens; the two empty failures supplied no token usage.

| Event sequence | Stage and actual observation | Workflow decision |
| --- | --- | --- |
| 3 | Six-step plan: create directory, create HTML, create CSS, create JS, run preview, answer URL. Required behavior was not assigned to feature milestones; preview was misclassified as `run`. | Plan accepted. |
| 5 | Model repeated the directory-step JSON without a tool call. | Missing effect; repair 1. |
| 9, 13, 15 | Directory creation and listing succeeded; model reported success. | Stage 1 advanced with a witness. |
| 19, 23, 25 | HTML write/read succeeded; answer parroted harness instructions for stage 3. | Stage 2 advanced with a witness. |
| 29, 33, 35 | CSS write/read succeeded; answer parroted harness instructions for stage 4. | Stage 3 advanced with a witness. |
| 37 | Actual provider result was `failure / empty_response`; no JS-stage operation yet. | Repair 2. |
| 41, 43 | JS write succeeded, followed by another `failure / empty_response`. | Shared repair allowance exhausted despite the write witness; honest stop. |

There was no fabricated `<tool_response>` answer in this attempt. Empty provider
responses and omitted feature planning are separate observed failures. The
[exact plan, replies, decisions and file hashes](attempts/attempt-2-runs.json)
and [unchanged generated files](attempts/attempt-2-app/PaperHarbor/index.html)
are retained. The HTML contains only a heading and script reference, while JS
accesses absent cart/total/search/product elements and lacks checkout/persistence.
No browser check was possible without a preview. The initial gate remains failed
with one task message, no corrective prompts, no manual app writes, no tool
forcing and no context reset. The operator reported approximately one active
minute for this attempt, excluding parallel work. Official tests remain deferred.
The operator exited the native session at approximately 06:20 UTC; exit code 1
carried the failed task status. No preview existed, so no shutdown HTTP probe
was applicable.

## Attempt 3 preparation

A fresh empty `target/s14-live/attempt-3/app` and separate control/evidence
directories contain the identical profile and two natural prompt files. The
[core-6 identity](attempts/attempt-3-freeze.json) was frozen before dispatch with
binary SHA-256 `BEBD1CA2D5135A1694CC0DC334E90760A47EBCB696F2BC1B31028873A3B8D36A`.
The source manifest includes `policy.rs` and actual workflow-instruction byte
admission alongside the recorded planning/empty-response refinement. Preparation
and freeze took approximately two active evidence-agent minutes. No application
files were written by the evidence agent.

The exact initial prompt ran as `05d972a2-9d2e-4af7-aa1d-0654f31fc339`,
06:27:29.443–06:27:59.171 UTC. It stopped with `workflow_recovery_exhausted`
and no candidate, after 15 model turns and six tools. Recorded request time was
29.728 seconds, including 29.353 seconds summed model time; recorded usage was
51,356 prompt and 2,193 completion tokens. There were two repair nudges, no final
review and no empty responses. The new witnessed-empty transition was therefore
not exercised by this attempt.

The plan again contained directory/file placeholders and classified preview as
`run`. Actual tool sequence 37 successfully started preview at port 57367 under
private prefix `636d93a64855467dad1547775388c338`. Because the stage required a
command witness, three subsequent URL/already-running answers triggered missing
effect repairs and an exhausted stop. This is a workflow classification failure
despite the real preview effect, distinct from app correctness. The generated
HTML has inline JS/handlers blocked by CSP, literal button markup assigned through
`textContent`, and POST/DELETE calls to static JSON files. Fixing the witness
classification alone would not produce the requested storefront.

[Exact trace and hashes](attempts/attempt-3-runs.json) and the
[unaltered generated app](attempts/attempt-3-app/PaperHarbor/index.html) are
retained. Browser tab 4 independently loaded the actual preview and showed only
Welcome to Paper Harbor, Products and Cart headings plus a Checkout button.
There were no rendered products, search or category controls. The initial gate
failed, so no follow-up was submitted. The operator reported approximately two
active minutes for setup/inspection. Scored session 44200 exited and HTTP on port
57367 actively refused connections afterward. One task prompt
and zero operator corrections/app edits were used. Official tests remain deferred.

## Unscored diagnostic continuations after attempt 3

Two explicitly corrective requests supplied concrete browser/CSP/static-server
failures against the retained app. They are outside the frozen two-prompt workload
and cannot upgrade attempt 3. [Exact requests, replies and provenance](attempts/attempt-3-diagnostics.json)
record two corrective prompts, two fresh sessions and approximately two additional
active operator minutes. Neither run wrote application files or started a preview;
the final app hashes still exactly match the original attempt-3 snapshot.

| Diagnostic | Actual operations and outcome | Cost |
| --- | --- | --- |
| Core 6, `1795b0f2-5624-4bd8-ace1-0c0dd9d162f2` | Listed root and PaperHarbor; read the actual index. At the next read stage it correctly identified embedded JS from the earlier read, then failed the mandatory new-stage read witness and exhausted recovery without repair. Later prose invented an index.js name. | Nine model turns, three tools, 12.860 seconds request time / 12.682 seconds model time. |
| Core 4, `21d0425b-e573-4a70-88c8-67a18a6bb774` | One batch listed root and attempted five reads at incorrect root paths. All five reads failed. After completion review it incorrectly claimed the website files were absent and asked the user for them. | Three model turns, six tools, 5.835 seconds request time / 5.725 seconds model time. |

The second diagnostic used the attempt-1 binary with the current attempt-3
profile/app and exact same diagnostic prompt. Multiple source differences mean
this does not isolate milestone removal as the sole causal variable. Sessions
11045 and 52538 both exited; neither created a preview. Runtime facts did not
produce a useful repair in either diagnostic, so no capability improvement is
claimed.

## Attempt 4 preparation

Fresh empty app/control/evidence directories are prepared under
`target/s14-live/attempt-4` with identical profile bytes, exact original two prompt
files and unchanged ceilings. [Source/binary identity](attempts/attempt-4-freeze.json)
was frozen at 06:44:45.475311 UTC; binary SHA-256 is
`40A5F0D41EC0B47210768311663198C25329E840E733261E1F0DA18BA2B48B3A`.
The manifest records 12 source/dependency hashes and the explicit core-7/tools-5
refinement. Enforced milestones are absent for new runs; static preview CSP
warnings and missing-file guidance are observations, not browser verification.
Preparation/freeze took approximately two active evidence-agent minutes. No app
source was written by this agent, and official tests remain deferred.

Initial run `50f26c61-4a0d-47cc-a791-d5dc3b869663` **failed** the live gate,
although its journal state is `completed / unchecked`. It used six model turns,
two tool calls, 109.721 seconds request time and 109.597 seconds summed model time;
recorded usage was 16,525 prompt and 7,848 completion tokens. Its initial prompt
matches the freeze. There were **two** generation-length repair nudges (sequences
3 and 15) and one completion review (sequence 13).

Both actual calls were `start_preview .`, each failing because index.html was
absent. Sequence 5 contained a long HTML example in assistant prose alongside the
preview call; it never performed a write. The final answer instructed the operator
to supply/check the missing file. No application files or preview URL resulted,
so the follow-up was not submitted. Preview compatibility warnings were never
reached and have no live repair evidence from this attempt.

[Exact trace and empty artifact manifest](attempts/attempt-4-runs.json) retain
the failure. Operator effort was approximately one active minute, with one task
prompt, zero corrections and no manual app edits. Session 16347 exited with
`/exit` after a separately labeled model-boundary diagnostic; no preview existed.
That diagnostic cannot upgrade this result. Official checks remain deferred.

## Unscored structured-action probe

A single direct model request used the same initial user prompt, Qwen2.5-Coder
7B, temperature 0 and 2,400 output ceiling, with a constrained action envelope
instead of native tools. It returned HTTP 200 in 780 ms, using 381 prompt and 34
completion tokens, and proposed `create_directory PaperHarbor`. No action was
executed and no app/source file changed. [Compact evidence](attempts/protocol-probe.json)
retains the proposed action and hashes of the raw ignored-workspace artifacts.
This establishes only a valid first-action proposal; earlier native runs also
created directories. It is neither functional progress nor evidence that a
structured adapter can produce the requested app.

## Unscored preview-warning capability diagnostic

Core-7 run `c1410049-1ab7-4cd9-a427-4325871303f3` received one request,
`Preview the existing PaperHarbor folder.`, against the unchanged failed
attempt-3 app. Both actual preview calls returned the same live URL and correctly
reported static CSP warnings for index.html line 13's handler and line 14's inline
script. Four model turns used two tools and 960 completion tokens. The model
printed replacement examples but performed no writes; app hashes still match the
original scored snapshot. The [diagnostic record](attempts/preview-warning-diagnostic.json)
keeps this separate from workload acceptance.

HTTP returned 200 before `/exit`, then port 56731 actively refused connections;
the diagnostic operator confirmed zero model-server processes. This verifies live
warning feedback and repeat-preview behavior, not autonomous repair, warning
clearance after edits or functional progress. It does not change attempt 4's
unexercised warning path or any scored failure.

## Attempt 5 preparation: new model-profile comparison

The fresh `target/s14-live/attempt-5` app is empty. It retains the same core-7
binary, original two prompt texts, tool grants and resource ceilings. The new
profile selects Qwen3-8B-Q4_K_M, appends the model-specific generic `/no_think`
suffix and changes temperature from 0 to 0.7. These are disclosed model/profile
changes, so the result cannot isolate a harness improvement. The model download
must match revision `6a569868d07d3bd59e8b97fb001bf8c0b254bb20` and SHA-256
`d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785` before a
pre-dispatch freeze. The primary operator verified that checksum and the model's
5,027,783,488-byte size; this agent independently checked size and confirmed the
binary and all recorded source hashes equal attempt 4. The
[attempt identity](attempts/attempt-5-freeze.json) was frozen at
06:55:15.534151 UTC. Profile SHA-256 is
`1ea5f7a10ffd6d2d472e76ef65edc92a2a0f38872448e6edd67fdcc5bd660f61`.
Preparation/freeze took approximately two active evidence-agent minutes. No app
source was written by this agent.

Initial run `f625cdd7-4901-4a47-9c91-23effd9b39cc` ran from
06:55:48.737 to 06:56:18.573 UTC and **failed** the independent browser gate.
It used nine model turns and seven actual calls: three directories, three file
writes and one successful preview. Request time was 29.836 seconds, including
29.640 seconds summed model time; usage was 21,692 prompt and 2,142 completion
tokens. It used one completion review and no repair nudge.

The browser loaded `http://127.0.0.1:55758/de1ac640799f4b298a17c62d46b5ccdf/`
and showed six products, but no search/category controls. Clicking Notebook's
Add to Cart produced no accessibility-tree change. The generated code used
malformed dynamically assembled inline handlers, no quantity controls or cart
persistence, and a cart initialized empty. The final answer claimed filtering,
quantity and checkout functionality and incorrectly said the acknowledged CSP
warning did not affect functionality. Its journal completion is unchecked, not
functional acceptance.

[Exact trace and hashes](attempts/attempt-5-runs.json) and
[unchanged generated files](attempts/attempt-5-app/Paper%20Harbor/index.html) are
retained. No follow-up was submitted. Operator effort was approximately two active
minutes, with one task prompt and zero corrective prompts/app edits. Session
45273 exited with code 0 and HTTP port 55758 actively refused afterward, at
approximately 07:00 UTC; exact closure time was not captured. Official tests
remain deferred.

## Attempt 6: reasoning-mode and output-budget comparison

A fresh empty app was frozen at 07:01:06.022289 UTC with the same core-7 binary,
all source hashes, pinned Qwen3 weights, original two user prompts and grants.
[Attempt identity](attempts/attempt-6-freeze.json) explicitly records removing
the `/no_think` suffix, changing temperature from 0.7 to 0.6, and increasing
`max_output_tokens` from **2,400 to 4,096**. All other ceilings remain unchanged.
This is a model-profile/reasoning/output-budget investigation; it is neither a
harness-source improvement nor an equal-output-budget comparison. Actual model
reasoning-mode behavior remains to be observed.

Profile SHA-256 is
`331d7c28207f2a9ca4a7991ed0c771e20791eedc361fd5a83760b2244866dc71`.
Preparation/freeze took approximately two active evidence-agent minutes. No app
source was written by this agent.

The exact initial request ran as `bd226c85-d794-454d-a3e9-49f7fa2e4a93`,
07:01:54.342–07:05:07.284 UTC. It completed unchecked after seven model turns
and three tools, with 192.942 seconds request time / 192.831 seconds summed model
time, 16,564 prompt tokens and 13,406 completion tokens. Two generation-length
recoveries consumed the repair allowance; one completion review followed.

Actual effects were directory creation, one index.html write and successful
preview startup. The preview reported four static CSP warnings. The final answer
claimed external styles.css/script.js had been created and compliance fixed,
although no such writes occurred. The [exact journal and manifest](attempts/attempt-6-runs.json)
and [single generated file](attempts/attempt-6-app/PaperHarbor/index.html) retain
this contradiction.

The independent browser showed the Paper Harbor heading, search input,
All/Stationery/Office/Art checkboxes, Cart Total 0 and Checkout, but **zero
products**. Clicking All and entering Notebook changed input values only. This
failed the initial functional gate and regressed product rendering relative to
attempt 5. No follow-up was submitted. Operator launch/browser/close effort was
approximately two active minutes, with zero corrective prompts or manual app
writes; the final file hashes still equal the retained initial snapshot.

The operator sent `/exit` to session 61386, which ended with code 0. HTTP at
127.0.0.1:64139 actively refused afterward. Attempt 6 is finalized as failed;
official tests remain deferred. The earlier existing-app CSP-warning diagnostic
is retained in its own unscored section and artifact, never as a scored pass.

## Attempt 7 and its pre-dispatch hold

The fresh app is empty; the profile and original prompt files are byte-identical
to attempt 4's Qwen2.5/temperature-0/output-2,400 baseline. The core-8/adapter-5
single-action experiment's first binary/source freeze
at 07:21:31.985998 UTC was [superseded before any launch](attempts/attempt-7-superseded-pre-dispatch-freeze-1.json)
after independent source review found encoded schema-bearing history could exceed
the configured history cap. The fix requires a new build and freeze. This hold
is not a scored failure or an additional model attempt; no app files were written.

After the actual-wire-history cap repair and rebuild, the
[final attempt identity](attempts/attempt-7-freeze.json) was frozen at
07:23:37.172757 UTC, before first dispatch. Binary SHA-256 is
`0C26DA235BF99301737BF4C388089E2932199A1223BB9AC125E0D98BB8CFB094`;
12 source/dependency hashes are recorded. The profile still exactly matches the
original 2,400-output baseline. Preparation and both freezes took approximately
three active evidence-agent minutes.

Run `fe0e4f1f-bd63-45a3-aea6-f677caca3cb0`, 07:24:20.071–07:24:22.359 UTC,
**failed** the initial gate. It returned two schema-valid answers and zero tools,
leaving the app completely empty and producing no preview. The first answer
offered a step guide; after the one completion review, the final answer asked
for details already supplied. No repair nudge or follow-up occurred. Request
time was 2.288 seconds / 2.265 seconds summed model time, with 2,847 prompt and
119 completion tokens.

[Exact trace and submitted prompt](attempts/attempt-7-runs.json) record its actual
456-byte initial prompt (frozen 454-byte text plus CRLF), not a byte-identical
submission claim. Operator effort was approximately one active minute, with zero
corrective prompts or manual app writes. Session 50725 ended via `/exit` with
code 0; no preview existed. The 0C26… binary remained unchanged. Official tests
and commits were not performed.

## Attempt 8: structured-protocol counterpart to attempt 5

The fresh app is empty. [Pre-dispatch identity](attempts/attempt-8-freeze.json)
was frozen at 07:27:54.682776 UTC with the unchanged attempt-7 0C26… binary and
all matching source hashes. Its profile is byte-identical to attempt 5:
Qwen3-8B with `/no_think`, temperature 0.7, output 2,400 and all original other
ceilings/grants. Profile SHA-256 is
`1ea5f7a10ffd6d2d472e76ef65edc92a2a0f38872448e6edd67fdcc5bd660f61`.

This is a bounded counterpart to native-tool attempt 5, using the same original
two user prompt files and independent full browser criteria. Core/adapter framing
differs; a single sampled comparison cannot establish general reliability or a
purely isolated causal effect. Actual submitted prompt bytes will be retained.
Preparation/freeze took approximately two active evidence-agent minutes. No app
source was written by this agent.

Run `77579e77-d3d7-4394-b783-ffcfdec09a81`, 07:28:33.306–07:28:35.278 UTC,
**failed** with two model turns, zero tools, zero files and no preview. Both
schema-valid answers merely promised future work, including after the single
completion review. There were no repair nudges. Request time was 1.972 seconds /
1.902 seconds summed model time; usage was 2,857 prompt and 76 completion tokens.
[Exact trace and prompt bytes](attempts/attempt-8-runs.json) preserve these facts.

The accepted prompt remained 456 bytes (454-byte frozen text plus CRLF), despite
the operator trimming command-output newlines before submission. Its actual
bytes match attempt 5's submitted prompt; both match frozen wording under the
same trailing-CR/LF rule. This is not a raw-byte-identical-to-file claim.

No follow-up was submitted. Operator effort was approximately one active minute,
with zero corrective prompts or manual app writes. `/exit` ended session 88595
with code 0; no preview existed. The app remained empty. This counterpart failed
to reproduce attempt 5's limited file/product availability and established no
functional advantage for the structured strategy. Official tests and commits
were not performed.
