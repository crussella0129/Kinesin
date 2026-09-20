# Independent analysis of Sprint 14 failures

Research only: retained journals, generated files, current retained implementation,
and the pinned runtime's source were read. No model, live application, unit test or
integration test was run. No Book or product source file was edited.

## Findings in priority order

### 1. A concrete schema/prompt ordering mismatch confounds the structured experiment

`src/model.rs::action_schema` builds schemas as `serde_json::Value`. Cargo.lock's
serde_json 1.0.151 has no indexmap dependency; its local `map.rs` confirms the
default BTreeMap. Production tool properties therefore serialize in the order
`arguments, kind, name`; answer properties serialize `kind, text`.

The b6500 converter uses ordered JSON, retains properties order, and concatenates
required properties in that order. Thus the production grammar permits a tool
object beginning with `arguments`, while beginning with `kind` selects the answer
branch. The action instruction teaches a kind-first object. This ordering fact
follows from [pinned converter source](https://raw.githubusercontent.com/ggml-org/llama.cpp/b6500/common/json-schema-to-grammar.cpp)
at lines 15, 614–659 and 827–844, together with the local serializer configuration.
The [server also uses ordered JSON](https://raw.githubusercontent.com/ggml-org/llama.cpp/b6500/tools/server/utils.hpp).

The successful direct probe's actual request instead orders tool properties
`kind, arguments, name` and answer properties `text, kind`. Its kind-first result
therefore selected a tool under a different grammar prefix. This is a real
integration mismatch and a strong explanation to falsify for attempts 7/8's
answer-only behavior. It is **not yet proof that property order caused those
samples**, nor proof that correcting it would produce a working app. Tool names
and argument fields being emitted late also defer action selection until after
arguments have been generated.

Do not fix this by globally enabling preserve_order: unrelated request bytes and
legacy replay fingerprints could change. A versioned ordered wire schema should
put the discriminator first in every branch and leave both action/answer choices
available. The runtime source was downloaded read-only to
`target/s15-research/references/json-schema-to-grammar.cpp` (ignored scratch; the immutable source URL above is the durable reference), SHA-256
`af3b67e9f80bdc87ae0905f1619471be6b794fc4a7750b512f03ad9e0d194ec5`.

### 2. Completion review is not an outcome gate

Proven behavior: one review is followed by acceptance of another ordinary answer,
even when it contains no execution, contradicts receipts or dismisses warnings.
`recovery.rs::reason/respond` explicitly implement that behavior. Freeform results
remain unchecked; this is a usefulness/termination-contract gap, not evidence
that checked acceptance was bypassed.

- Attempt 1 follow-up `a96761d6-565a-48e3-9171-1301c45f9b34` made one failed
  preview call, no writes, then claimed Clear Cart/item-count changes.
- Attempt 5 `f625cdd7-4901-4a47-9c91-23effd9b39cc` acknowledged a real CSP warning
  but said it did not affect functionality; the browser's Add button was inert.
- Attempt 6 `bd226c85-d794-454d-a3e9-49f7fa2e4a93` claimed external CSS/JS repairs
  after only one index.html write and no post-warning file operations.
- Attempts 7/8 accepted future-work prose after review with zero tools.

Receipt validation could prevent some false completion claims but would not
implement missing features. A useful repair loop needs an explicit observable
outcome and a bounded response to its failure, not another self-review alone.

### 3. Real effects work, but correctness observations are largely absent

Across nine scored requests there were 13 file writes, seven directory creations,
seven preview calls, one listing and two reads; **zero command calls**. No model
received browser behavior during the scored requests. Preview startup establishes
serving, and the new scan establishes index-only compatibility warnings—not a
working cart, valid DOM wiring or persistence. The two reads in attempt 2 checked
scaffold files, not behavior. Native dispatch is therefore not globally broken.

Actual generated defects include CSP-blocked inline handlers/scripts, malformed
dynamic button markup, missing DOM targets, unsupported writes to static JSON,
and absent feature implementations. These are direct artifact/browser findings.
Their recurrence shows failure under this workflow; it does not isolate an
intrinsic model-capacity ceiling.

Feedback alone has not been shown sufficient: diagnostic
`1795b0f2-5624-4bd8-ace1-0c0dd9d162f2` was blocked by stage witnessing after it
had already read the relevant inline code; `21d0425b-e573-4a70-88c8-67a18a6bb774`
read wrong root paths and falsely declared the app absent. Neither wrote files.
The preview-warning diagnostic `c1410049-1ab7-4cd9-a427-4325871303f3` returned
correct repeated warnings but the model printed repair examples instead of edits.
These confounded/failed diagnostics do not justify claiming a browser tool alone
will solve the problem.

### 4. Rigid milestones introduced additional failure modes

Attempt 3 `05d972a2-9d2e-4af7-aa1d-0654f31fc339` genuinely started preview, then
exhausted repairs because the model had classified that stage as `run` rather than
`preview`. The read-stage diagnostic likewise required a new read even though the
prior read had already answered the stage's question. These are observed false
workflow stops. Plans also reduced requested behavior to file placeholders.
Removing stages avoids those particular stops; it does not establish task success.

### 5. Output truncation matters locally; other resource exhaustion is unsupported

Attempt 4 had two 2,400-token truncations; attempt 6 had two 4,096-token truncations.
Discarding incomplete actions prevented partial writes, but the model did not
reliably reduce scope on retry. Attempt 6 simultaneously changed reasoning mode,
temperature and output budget, so its worse result cannot be attributed to any
one setting. The retained incomplete events do not show whether the truncated
budget was spent on code, reasoning or other text.

No scored request exhausted time, turn count, tool count or history limits, and no
compaction count was recorded. Short answer-only attempts 7/8 were nowhere near
their output/context ceilings. More context or tokens is therefore not a supported
general explanation or fix. Prompt/framing interference remains possible even
without physical context exhaustion.

## Observed comparison and costs

Run prefixes below identify the full run IDs in the linked retained records.

| Attempt | Configuration/change | Actual outcome | Request seconds |
| --- | --- | --- | ---: |
| [1](../../s14/sprint-tests/attempts/attempt-1-runs.json) `97241cfd` / `a96761d6` | 7B native; completion review | Six products, absent filters, inert cart; follow-up no writes | 39.219 combined |
| [2](../../s14/sprint-tests/attempts/attempt-2-runs.json) `de003881` | Typed milestones | Scaffold only; empty-response exhaustion; no preview | 24.225 |
| [3](../../s14/sprint-tests/attempts/attempt-3-runs.json) `05d972a2` | Refined milestones | Preview, zero products; run/preview witness false stop | 29.728 |
| [4](../../s14/sprint-tests/attempts/attempt-4-runs.json) `50f26c61` | Native core 7, static warnings | Code as prose, failed previews, no files | 109.721 |
| [5](../../s14/sprint-tests/attempts/attempt-5-runs.json) `f625cdd7` | Qwen3 non-thinking, temp .7 | Six products; no filters; inert cart; warning dismissed | 29.836 |
| [6](../../s14/sprint-tests/attempts/attempt-6-runs.json) `bd226c85` | Qwen3 reasoning, temp .6, output 4096 | Zero products; controls present; fictitious fixes | 192.942 |
| [7](../../s14/sprint-tests/attempts/attempt-7-runs.json) `fe0e4f1f` | Structured protocol, 7B | Two answers; no effects | 2.288 |
| [8](../../s14/sprint-tests/attempts/attempt-8-runs.json) `77579e77` | Structured protocol, profile identical to 5 | Two promises; no effects; lost 5's file/product availability | 1.972 |

All eight failed. No scored attempt established a working cart, persistence or
checkout; seven did not reach the natural follow-up. This is insufficient evidence
to diagnose session-memory failure. Preview availability and shutdown worked when
a preview was actually created. Criterion presence is not functionality: attempt
6 added controls while losing attempt 5's product rendering.

Nine scored requests used 61 model turns, 30 tools and 429.931 seconds. Summed model
exchanges account for 428.628 seconds (about 99.7%); computational harness overhead
was not the measured latency bottleneck. Operator setup/browser/close estimates
sum to roughly 14 active minutes; separately recorded evidence preparation sums
to roughly 18 minutes, potentially overlapping. The two corrective diagnostics
add roughly two operator minutes. Engineering, downloads, research and other
diagnostic work were not comprehensively timed: these are **not total project
costs**, and there is no human-authoring baseline or demonstrated time saving.

## Controlled probes to plan, not run now

Run sequentially with early stop decisions; do not launch a full factorial sweep
or another blind storefront rerun. Freeze source, actual wire request, model hash,
template/runtime, sampling/seed, fixture hashes and budgets. Report real effects
and independent behavior. Keep these diagnostics separate from the final full
two-prompt/zero-correction acceptance attempt.

1. **Protocol ordering, first.** Materialize adapter-5 wire schemas and grammar
   without inference. Check legal next keys after the common prefix. Then compare
   the original property order against discriminator-first order in every branch,
   holding all other request bytes semantically equal, the same model/sampling,
   and the same tiny file-creation task. Both branches remain available. A correctly
   selected/executed tool only after reorder supports the prefix hypothesis;
   unchanged answer-only behavior falsifies it as a sufficient cause. Do not
   award success for parseable JSON alone or force the tool branch.
2. **Action selection versus prompt overhead.** On a tiny create/read/edit request
   whose answer fits well below the cap, use a fixed model and verified protocol.
   Compare a minimal generic instruction against the full production instruction,
   with identical tool semantics, task and budgets. Require actual correct file
   bytes and a successful edit; record prompt tokens and first action. If minimal
   succeeds and full fails, investigate framing/overhead; if both act correctly,
   stop attributing the storefront failure to basic dispatch. No global prompt
   rewrite should precede this discrimination.
3. **Output ceiling alone.** On the same bounded coding task and fixed reasoning
   mode/model/temperature, compare only 2400 versus 4096 output tokens. Record
   finish reason, usage, actual valid writes and correctness; capture separate
   reasoning/content token or byte counts if the runtime exposes them. Increased
   correct output would support a budget bottleneck; more prose or truncation
   without useful effects would not. Never infer this from the confounded 5→6 pair.
4. **Missing feedback versus repair ability.** Use identical disposable copies of
   a small known-broken fixture with one observable defect and known correct paths.
   Compare the same repair request with and without an accurate bounded browser
   observation, keeping the qualified native loop and all other inputs constant.
   Verify the interaction after actual edits. Only a successful feedback arm
   supports building automatic observation delivery; failures in both arms point
   toward action selection/repair capacity instead. This is an unscored diagnostic
   with explicit operator feedback, not the final zero-correction benchmark.
5. **Model/profile capacity last.** Once protocol, basic effects and feedback paths
   are qualified, compare the two already available model profiles on the same
   small correctness task, then the unchanged full workload only if warranted.
   Hold non-model settings fixed where supported; disclose required template/mode
   differences and recorded sampling. Repeated paired disagreement can support a
   model/profile limitation; one failure cannot establish universal incapacity.

Guardrails for the plan: changing schema order is not the same as forcing an
action; stricter truthful completion is not implementation success; warnings are
not browser verification; an observed repair is not complete feature coverage.
Keep the independent full acceptance gate and legacy replay identities intact.

