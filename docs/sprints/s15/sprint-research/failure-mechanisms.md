# Failure mechanisms after all six diagnostics

Source and retained-evidence review on 2026-09-20. No additional model request,
test, application edit or product change was performed for this note. The six
diagnostic slots are exhausted; proposed studies below require a new approved
budget. Neither repair arm qualifies the full storefront attempt.

## What the final pair establishes

Calls 5 and 6 used the same ordered `4/8/6/5` path that completed the explicit
file tasks in calls 2 and 4. Their actual first schemas contain eight tool
branches ordered `kind,name,arguments` and the answer branch `kind,text`.
The system message offers `list_files` at `.` and `read_file`, along with the
other six granted tools. No native `tools` field is expected in this declared
structured protocol. Both branches are valid choices; neither was forced.

| Recorded fact | Call 5, unaided | Call 6, browser observation supplied |
| --- | --- | --- |
| Run ID | `88a1318a-c848-44ca-afb6-4f34229410d6` | `2f6fb621-84f5-4323-9ff8-d63f7971e045` |
| First action | Answer requesting the app's code | Answer claiming an edit to `cart.js` / `updateCart` |
| After completion feedback | Again asks for files | Again claims JavaScript was edited |
| Actual model turns / tools | 2 / 0 | 2 / 0 |
| Summed completion tokens | 77 | 123 |
| Final journal state | completed / unchecked | completed / unchecked |
| Files and preview | All three seed files unchanged; no preview | All three seed files unchanged; no preview |

The fixture is `index.html`, `app.js`, and `style.css`; it contains no `cart.js`
or `updateCart`. The actual defect remains `totalCents = product.priceCents` in
`app.js`. Independent hashing found every file in both arms equal to the
[retained seed](../sprint-tests/diagnostics/repair-preparation/seed/app.js).
Call 6's observation correctly reported two items priced $2 and $5 producing a
$5 total, with search still working. It supplied behavior, not source code or
a repair instruction. No read, list, write, denial, preview or repair operation
was attempted in either arm.

The exact first-wire SHA-256 values are
`dc2e4e804d9988fb766bc732b8210f04778d5533ae2ca016ff16424efa5ffdea`
and `067c343e33faf81cead461e4140ff1f28616e6bbef1b297eeedfcf6566df4061`.
Raw local records are under `target/s15-live/call-05/evidence/` and
`target/s15-live/call-06/evidence/`: `wire-00/01-request.json`, corresponding
responses, freeze, capture and terminal records. These are distinct from the
[prepared prompts and authentic observation](../sprint-tests/diagnostics/repair-preparation/observation.json).

## Discovery, execution and termination are different boundaries

**Observed discovery/action failure:** tools for obtaining the code were offered,
but the model neither located nor read it. Files not already included in a
message were treated as unavailable in call 5; call 6 substituted invented
implementation details. This is not a filesystem failure or an authorization
denial. It also does not measure whether the model can repair this particular
code after observing it: that stage was never reached.

Calls 2 and 4 named an existing JSON path, target field and exact output file.
They successfully read, edited and wrote through the same ordered path. This
establishes some execution competence, not autonomous project discovery or
JavaScript repair competence. Filename specificity is a plausible contributor,
but those tasks also differ in language, complexity and requested preview;
their comparison with calls 5/6 does not isolate it. See
[pair 1](../sprint-tests/diagnostics/pair-1-analysis.md) and
[pair 2](../sprint-tests/diagnostics/pair-2-analysis.md).

**Observed termination mechanism:** `Recovery::reason/respond` grants one
completion review and sets `reviewed = true`. `RunState::observe_model` returns
early on that retry, without retaining the proposed answer in conversation.
The actual second wires therefore contain `system,user,user`, not the first
assistant claim. Their feedback contains `outcomes: []`. This is another
opportunity to work, not verification of the previous claim. A subsequent valid
nonempty answer becomes `Candidate`, then `Completed/Unchecked`; no independent
work contract rejects the unsupported blocker or fabricated repair. The CLI's
new unverified label is accurate, but cannot make the work happen. See
[recovery](../../../../src/recovery.rs), [core](../../../../src/core.rs), and
[runner](../../../../src/runner.rs).

Keeping the prior answer might improve continuity or instead reinforce the
invented repair. These runs establish its absence, not the effect of retaining
it. No new feedback prompt or candidate-history change is justified as a proven
fix by this observation alone.

## What this adds to the eight Sprint 14 failures

The [retained Sprint 14 assessment](../../s14/failure-report.md) already separates
missing features, false stage stops, ignored CSP warnings and answer-only runs.
The new pair shows that corrected schema order is insufficient for this repair
request. The earlier ordering pair supports a narrow benefit on its fixture;
it does not become invalid because another task failed.

Authentic browser feedback did not produce even an inspection action here.
That falsifies sufficiency for this sampled condition, not the general value
of browser observations. Context-window/output exhaustion or loss of carried
session facts do not explain these particular first-turn failures: the first requests
used only 1,339 / 1,583 prompt tokens, both generations stopped normally, and
both sessions began without prior context. No compaction or resource-exhaustion
mechanism occurred. Intrinsic model capacity, task framing and action-selection
bias remain hypotheses rather than uniquely established causes.

## Smallest discriminating next study, not authorized execution

First compare the same repair task and fixture with versus without one accurate
relative filename (`app.js`), holding protocol, feedback condition, model,
limits and all other wording fixed. Score actual discovery/read/edit operations,
correct cart behavior, preserved search and real preview—not the answer. A
named-file-only success would support a discovery/grounding dependency; neither
success would leave repair/action selection unresolved. One pair is not a
reliability estimate.

Only if another scoped study is warranted, contrast automatic discovery with a
genuine pre-recorded bounded directory or source observation under the existing
read grant. Mark that context as assistance and charge its real operation and
bytes; never invent a tool result. This would separate locating code from using
observed code. Do not simultaneously change prompts, model, schema and budgets.

A mandatory first tool call is not a general recommendation. It removes a legal
answer/blocker choice, can force irrelevant activity, and would confound action
selection with execution ability. Even a deliberately assisted first read could
only qualify the downstream path; it would not prove autonomous discovery.
Forcing mutation is particularly inappropriate as a substitute for understanding
the task. Any such contrast needs explicit scope and retained tradeoffs.

A larger state graph would presently encode assumptions about unmet goals and
tool witnesses without establishing the missing capability. A browser subsystem
would improve potential observations but would not explain why available reads
were unused. Both remain premature. Keep the factual session/status mitigation,
the failed repair evidence and the existing authority/budget boundaries; do not
promote them into semantic acceptance or a claimed usefulness pass.
