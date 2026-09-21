# Sprint 16 experiment option: discover, inspect, repair

This is an independent research proposal, not an authorization to dispatch or
modify the product. No new model requests, tests or browser operations were run
to prepare it. The S14/S15 failures remain failures; the S15 six-request budget
is exhausted and is not reused here.

## Why this boundary

S15 ordered requests 2 (`030c89b2-43d0-430b-bb83-d0a741f7af04`) and 4
(`5318c555-9eb6-4c15-b491-1d170adedd41`) successfully read named paths, made exact
edits and wrote notes. Native held-out request 3
(`a40346e2-18c3-411e-a503-3e8bb0712bae`) emitted dependent actions together,
used a pattern where an exact match was required, and failed the file oracle.

Ordered repair request 5 (`88a1318a-c848-44ca-afb6-4f34229410d6`) did not inspect
the available files and asked for code. Request 6
(`2f6fb621-84f5-4323-9ff8-d63f7971e045`) received authentic browser symptoms but
still emitted only answers, including an unsupported edit claim. Neither
exercised code repair after reading source. The key contrast with requests 2/4
is not merely more complex code: the repair request did not name its source
paths. Task complexity, wording and required discovery remain confounded.

S14 similarly mixed omissions, invalid app code, protocol artifacts and absent
effects. No result establishes blanket model incapacity, nor that browser
feedback alone will supply the missing action selection.

## One adaptive ladder; two to four requests

Freeze the decision tree and every possible prompt/fixture before request A.
Use the existing ordered protocol and the same Qwen2.5 profile, output/context
ceilings, runtime, seed, grants and repair/review caps. Freeze the actual current
source and wire identities; do not silently substitute historical binaries.
Every arm has an independent empty session and fresh fixture copy. No operator
patches or mid-run corrective messages. Failed, aborted and transport-failed
submissions consume their slots; no replacements.

**A — current unaided baseline.** Reuse the exact S15 repair seed and base
request, with the already observed one-defect oracle and unchanged search
control. Require actual corrected files, a returned running preview and the
same independent cart/search interactions. Historical S15 failure is context;
this fresh baseline controls the current build and runtime. If A passes,
skip B/C and proceed to D using unaided input: two total requests.

**B — filenames only, if A fails.** On a fresh identical seed, append a frozen
reference-only list of all existing relative source filenames: `index.html`,
`style.css`, `app.js`. Do not name the suspected faulty file, functions or lines;
do not give source text, a diagnosis, an implementation or a tool command.
Everything else matches A. If B passes, select filename assistance for D and
skip C: three total requests. This is an assisted diagnostic, not a natural
production acceptance request.

**C — authentic listing and source, only if A/B fail.** Use the same B prompt
and file inventory, adding the complete bounded, unmodified text of those three
files as hash-bound historical observations. The inventory must come from the
actual fixture, not an invented directory listing. Present all files without
highlighting or explaining the seeded bug. Keep the existing output and history
caps: if the frozen observation cannot fit, stop preparation instead of raising
limits or truncating away relevant code. This bypasses discovery and source
retrieval while leaving diagnosis, authorized editing and verification to the
assistant. If C fails, stop after three requests. If it passes, proceed to D
using the same source-assistance rule: four total requests.

**D — one held-out repair using the least-assisted successful condition.**
Prepare this fixture before A and keep it unavailable to earlier workspaces.
Use different file locations/names and a different defect: for example removing
a cart row with quantity two must remove both units and their full price,
while catalog/search remain correct. Give the requested behavior without source
paths in the base prompt. Use A's unaided condition, B's all-filename condition
or C's complete-source condition exactly as selected above; no ad hoc extra
help. Independently exercise the seeded failure before dispatch and the repaired
interaction plus unaffected control afterward. One pass measures limited
transfer to this distinct fixture, not general coding reliability.

## Decisions and observations

| Result | Supported interpretation | Stop/promotion rule |
| --- | --- | --- |
| A and unaided D pass | Current path repaired two distinct fixtures without assistance. | May justify the separately frozen original full storefront gate. No usefulness claim before that gate passes. |
| A fails; B and filename-assisted D pass | Explicit file-location context is sufficient on these samples; discovery/path salience is a plausible bottleneck. | Stop diagnostic work and plan a minimal automatic discovery/context mechanism. Assisted wins do not unlock production acceptance. |
| A/B fail; C and source-assisted D pass | Repair can occur after authentic source is supplied; context acquisition/presentation is a plausible dependency. | Stop and plan a bounded automatic retrieval mechanism; do not introduce a general state graph. |
| Selected condition fails D | The local success did not transfer to the held-out fixture. | Retain the regression and stop; no second held-out fixture. |
| A/B/C all fail | Filename and source provision were insufficient under this frozen profile. | Stop. Do not launch a token/model/prompt sweep or infer universal incapacity. |

Record first selected action/answer; actual listing/reading and supplied-source
visibility; successful/failed effect receipts; exact before/after file hashes;
preview availability; browser defect/control results; model/request time and
usage; operator preparation/assistance time and interventions. Keep unsupported
claims separate from actual effects. In particular, distinguish:

- No inspection/action at all: action selection or framing remains the immediate
  observed barrier; repair reasoning was not exercised.
- Reads occur, then no edits: discovery alone is insufficient on that arm.
- Edits occur but the independent behavior fails: repair correctness, not mere
  ability to act, is now the observed boundary.
- Correct repair with an unaffected-control regression: overall failure.

## Limits on inference and scope

This is a small deterministic-profile sample, not a statistically established
causal estimate. A→B changes path salience as well as the need for discovery;
B→C changes source availability, token count and framing. A pass therefore
supports sufficiency of the intervention on that task, not a unique root cause.
Sequential shared-runtime cache state prevents clean latency claims. Report
fresh observed outcomes rather than upgrading the earlier S15 failed repairs.

Do not use assisted D as evidence of autonomous source discovery. Do not copy
its fixture implementation into the model's workspace for the full workload.
Keep the original storefront initial request and same-session Clear Cart/item
count follow-up, all browser criteria, and zero human corrections/patches/tool
forcing/context resets unchanged. Official unit/integration checks remain after
the live usefulness gate. Automatic listing/retrieval, if indicated, requires
an explicit scoped implementation decision and unaided requalification under a
new frozen budget; this proposal does not silently authorize that extra work.
