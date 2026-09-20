# Sprint 15 failure assessment

Sprint 15 failed its usefulness qualification. The six-request diagnostic
sequence is complete; the full storefront and same-session follow-up were **not
run**, because both small-app repair arms failed. Official unit/integration
verification and Clippy remain unperformed under the approved live-first gate.

## What the evidence establishes

| Slot | Comparison arm | Independently scored outcome |
| --- | --- | --- |
| 1 | Historical structured schema | Fail: two answers, zero tools, inventory unchanged, note absent. |
| 2 | Ordered schema, identical inventory task | Pass: correct quantity edit, controls preserved, exact note written. |
| 3 | Native protocol, held-out shipping task | Fail: two exact edits failed, setting unchanged and note incorrect. |
| 4 | Ordered schema, identical shipping task | Pass: correct setting edit, controls preserved, exact note written. |
| 5 | Ordered, unaided app repair | Fail: two answers, zero tools/edits/previews; asked for files that were available. |
| 6 | Ordered, repair plus authentic browser observation | Fail: two answers, zero tools/edits/previews; falsely claimed a JavaScript edit. |

The [pair analyses and raw archive](sprint-tests/diagnostics/attempt-ledger.md)
bind results to frozen prompts, model/runtime/profile/binary identities, actual
wire requests, journals and file bytes. A browser operated the unchanged repair
seed before either arm: adding the $2 pencil and $5 notebook produced two listed
items but a $5 total; search still worked. Both terminal apps remain byte-identical
to that seed. Neither returned a preview, so there is no candidate browser pass.

The ordering change supported a narrow causal prediction on the matched tiny
task and qualified on a second explicit-path task. It did **not** establish
repair competence or general model reliability. The held-out native run's
dependent calls were proposed before it saw their results; regex-looking strings
were sent to an exact-match editor. This differs from the ordered repair arms'
failure to initiate any action. One universal failure explanation would erase
these distinctions.

## Deeper causes and remaining uncertainty

The harness still depends on the model to bridge a user's request to workspace
inspection. In the successful diagnostics the prompt names an exact file; in
the repair task it names app behavior. The available listing/reading tools were
never dispatched in either repair arm. This suggests a discovery/context
grounding boundary, but these different tasks are not a controlled contrast
isolating file names, task complexity or prompt semantics. The next study must
separate those variables before prescribing a fix.

The existing completion review accepts another answer candidate without an
independent task contract. It can expose that work is unchecked, but it cannot
turn a false edit claim into an actual edit. The new machine-owned facts improve
the account of prior activity; they are not an initiation mechanism and were not
exercised in a second session request here. More state machinery alone has no
demonstrated remedy for this failure.

Real browser feedback was insufficient in the sampled assisted run. That does
not prove feedback is useless: no action trajectory started, so repair ability
after discovery remains unmeasured. It does rule out treating observer
availability alone as a demonstrated solution. Neither a browser subsystem,
forced tool-only branch, new model nor larger token budget is authorized by this
negative result. See the [mechanism audit](sprint-research/failure-mechanisms.md)
for the evidence/hypothesis boundary and proposed discriminating research.

## Disposition

- **T-119–T-121:** compiled implementation retained at `ad058c5`, with source
  review and the limited live observations above. Official protocol, reducer,
  replay, session-memory and admission verification remains outstanding. These
  tasks return to backlog without product-completion claims.
- **T-122:** diagnostic execution/evidence task completed after independent
  scoring. Its success means the bounded experiment reached an honest decision,
  not that the product succeeded.
- **T-123:** blocked by unaided repair failure; zero of its two possible full
  attempts consumed. The [unchanged workload](sprint-research/live-workload.md)
  is retained as not run, not permission to bypass qualification.
- **T-124:** not started; official checks remain deferred. T-123/T-124 return to
  backlog for explicit replanning. Sprint 14's failed T-115–T-117 remain history.
- **INT-0032 and INT-0033:** active and unrealized. No reduced-babysitting or
  productivity claim; no installation, default promotion, release or merge.

All six top-level requests count, including failures; no replacements, model
sweep, corrective model messages, manual app edits or context resets occurred.
Slot 6's upfront observation was declared diagnostic assistance. Lab setup,
browser observation and evidence work were operator effort, not autonomy. Exact
human-equivalent effort was not measured. Slot 1 had a post-run evidence-script
parsing error; its real capture was exported without resubmitting the request.

All owned diagnostic sessions exited. The owned runtime, relay and seed server
were stopped; [cleanup](sprint-tests/diagnostics/cleanup.json) records zero
remaining listeners. Initial PowerShell cleanup raised two errors; identity-
checked termination completed afterward. No unrelated server was stopped.

Formatting and compilation passed before dispatch. Those checks do not verify
all new code paths. Evidence and source remain local under the approved Sprint
15 checkpoint boundary while Sprint 14's separate draft PR #14 awaits human
disposition. Pushing the shared `dev` branch would change that archival PR;
this assessment does not silently combine the two sprints.
