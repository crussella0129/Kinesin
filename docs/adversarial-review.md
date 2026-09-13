# Adversarial review: execution is not task correctness

The machine-local temporary path in this historical report is normalized to an
explicit placeholder for privacy. Review timestamps, outcomes, and evidence
references are unchanged.

Reviewed **2026-09-08 UTC**. This is a review and revision of the design, not a
penetration test or a claim that an unbuilt Rust runtime has passed its tests.

**The previous pass had not solved the problem.** It said that a `succeeded` run
need not solve the task and added offline evaluations, but there was no per-run
acceptance contract, independent checker, persisted verdict, or client rule that
prevented completion from being consumed as success.

The revised design makes that distinction enforceable for a small supported
task class. It does not solve general natural-language verification. Freeform
results remain unchecked. A pass always identifies the contract and observations
it covers. [Task acceptance](verification.md) is the authoritative specification.

## Review checklist

- [x] Read the current contracts and identify claims without enforcement.
- [x] Independently challenge security, concurrency/finalization, and the build sequence.
- [x] Construct false-pass cases and define explicit outcomes.
- [x] Revise the runtime, configuration, storage, clients, metrics, and build guide.
- [x] Validate the final cross-document contracts, examples, and proposed schema.

## Findings and disposition

Severity describes the consequence if the earlier plan were implemented as
written. “Addressed” below means addressed in the specification; executable
evidence is still required at the corresponding build step.

| Finding | Severity | Why it matters | Revision |
|---------|----------|----------------|----------|
| Completion had no independent acceptance gate | High | A fluent wrong answer could become the only terminal success signal | Separate `completed` from acceptance; add a pure checker and receipt |
| No frozen, authorized task specification | High | A weak check could certify an unrelated request, or the model could supply its own test | Disjoint freeform/checked submissions; operator task profiles; resource authorization intersection |
| Provenance alone did not establish the claimed fact | High | The correct file can be cited while the answer contradicts it | Compare each structured value against the actual complete scoped source observation |
| No atomic result/verdict contract | High | Readers or restart recovery could see an answer accepted before checking settled | One transaction for candidate, receipt/projection, and terminal event |
| Cancellation precedence was promised until commit | Medium | A submitted storage command cannot safely be assumed recallable | Reserve inbox capacity, recheck cancellation/time, synchronously transfer terminal command, then settle once |
| Checking was absent from resource accounting | Medium | A future validator could hold model slots, hide work after terminal state, or retain unlimited evidence | Bounded inline checker; release model capacity; retain active ownership; no extra pool or repair loop |
| Clients and metrics could collapse all completion into success | High | HTTP 200, exit zero, or a terminal SSE event could be mistaken for correctness | Both outcome axes everywhere; derived `task_accepted`; strict CLI; separate throughput denominators |
| A checker may implement an inadequate human requirement correctly | Residual | A narrow pass cannot establish an unstated broader goal or the truth of a stale file | Explicit scope, profile review, independent eval fixtures; unrestricted work stays unchecked |

## The concrete counterexample

The profile asks for `language` from `project.txt`. The actual complete tool
observation is `language=Rust`, with runner-issued evidence ID `e0`.

```json
{
  "facts": [
    {"id": "language", "value": "Python", "evidence_id": "e0"}
  ]
}
```

The model used an allowed tool, read the right file, produced valid JSON, cited
real evidence, and ended normally. Execution is **completed**. The field check
is **failed**, `task_accepted` is false, and the CLI exits 2. Merely checking for
an evidence ID, tool success, or required output key would miss this error.

With the exact value `Rust`, a candidate can pass this extraction contract.
That does not establish that the repository is implemented in Rust, that the
file is accurate, or that it remained unchanged after observation.

## Attack and failure matrix

Feed these cases directly to the checker/runner using deterministic fixtures;
do not depend on persuading a live model to produce the attack. The actual
implementation must demonstrate the expected result before the release gate.

| Attack or failure | Expected behavior |
|-------------------|-------------------|
| “All tests passed” in an unrestricted final answer | Freeform stays unchecked |
| Correct value with an invented evidence ID | Failed binding; no accepted result |
| Correct value copied from another owner's or another run's observation | Evidence lookup is local to the owning run; failed binding |
| Right basename in the wrong workspace/path | Failed resource binding |
| Allowed task alias points into a forbidden workspace | Reject before admission; alias permission cannot grant workspace access |
| Caller adds “also evaluate this investment” to an extraction task | Reject extra prompt/unknown fields in checked submission |
| Caller/model supplies an easy expected answer, omitted criterion, or alternate checker | Reject untrusted contract mutation |
| Empty requirements or unknown checker | Reject setup; empty conjunction is never a pass |
| Correct fields plus unsupported or contradictory prose | Fail the entire typed output contract |
| Duplicate JSON keys or repeated fact IDs | Fail; never silently overwrite one value with another |
| Error body contains the expected word | No usable successful evidence; source check inconclusive, or failed if the candidate also forges a reference |
| Required file/key absent or beyond truncation | Source check inconclusive; unseen bytes cannot establish the field. A separate demonstrated binding/output failure takes precedence |
| Required source has malformed lines or duplicate keys | Inconclusive; parser never chooses a convenient occurrence |
| Two complete reads disagree | Inconclusive; no cherry-picking an observation |
| File changes after it was read | Evaluate exact observed bytes, with explicit as-observed scope |
| Required full-content digest does not match | Fail the declared revision requirement |
| Source text contains instructions to mark the task passed | Treat as source data; cannot change criteria, permissions, or verdict |
| Candidate exceeds 8 KiB | Failed output-contract check; retain bounded candidate under result policy |
| Retained evidence exceeds 64 KiB | Stop with verification_evidence_limit and inconclusive acceptance |
| Checker faults after one criterion passes | Inconclusive; no partial or averaged pass |
| Cancellation/deadline arrives during checking or terminal queue wait | Before submission, cancellation/stop takes precedence and acceptance is inconclusive |
| Cancellation arrives after terminal command entered storage inbox | Settle that command once; cannot recall or rewrite it |
| Process dies after checking but before terminal commit | Recovery yields interrupted + inconclusive; no automatic recheck |
| Receipt write fails after result write inside transaction | Roll back both result and verdict/event; never expose a partial accepted state |
| SSE observer sees a plausible answer before checking | Provisional display only; terminal receipt follows combined commit |
| Same submission key retried after profile edits | Original run and verdict returned; changed requested task/mode conflicts |
| Imported receipt says passed | Untrusted historical data, not authority to install a live accepted result |
| An LLM judge returns high confidence | No path to deterministic acceptance in the initial design |

Detailed assertions and fixture guidance are in [testing](testing.md#per-run-task-acceptance).
The guide adds [an acceptance-checking step](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md#21-check-file-fields-before-accepting-a-task)
before concurrent execution, then carries its ownership, persistence, replay,
metrics, and API consequences through later steps.

## Why this checker and not a universal judge

The immediate learning target is small enough to understand completely:
typed Rust values, a bounded line scanner, scoped evidence lookup, exact
comparison, exhaustive outcomes, and a transaction. No extra model calls,
generic workflow engine, validator plugins, shell tests, or automatic repair
loop are required. These are engineering choices for this project.

Outcome-based evaluation checks what happened, and different code, model, and
human graders have different strengths and limitations. That supports keeping
execution evidence and acceptance separate. [Anthropic's agent evaluation guide](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)

Model judges introduce an additional prompt-injection attack surface. A judge
can be useful in a later calibrated evaluation process, but its output would
need its own trust and error model. It is not the first acceptance authority.
[Adversarial attacks on LLM-as-a-judge systems](https://arxiv.org/abs/2504.18333)

For a new task class, define the requested postcondition, observable evidence,
checker authority, known false-positive/false-negative cases, and limits first.
Some goals need a human decision or remain uncheckable with the available tools.
Adding a verifier trait cannot manufacture missing evidence or a good rubric.

## What remains outside the claim

- **Specification adequacy:** humans can choose the wrong criterion. Review
  profiles against the intended task, with independent labeled examples.
- **World truth and freshness:** observed-file agreement is neither a factual
  truth guarantee nor a cross-file atomic snapshot.
- **Open-ended quality:** unrestricted prose, creative work, and broad judgments
  remain unchecked by the initial runtime.
- **Trusted implementation:** a checker bug, compromised controller, or modified
  database can invalidate a verdict. Digests are identity checks, not signed attestations.
- **Deployment security:** capability paths, bounded resources, owner checks,
  and prompt-injection tests still need implementation and selected-platform
  evidence. A contract pass is not a security certificate.
- **Latency:** bounded simple work is a design constraint; its measured overhead
  remains unknown. Report checker time, execution completion, contract-pass
  throughput, and independently evaluated goal success separately.

## Validation record

Final checks passed across 21 Markdown files: 248 local links/anchors, unique
headings, balanced fences, 41 consecutive guide steps and matching roadmap
entries, and each step's build/proof/failure/reading sections. Five JSON and three
TOML examples parsed; the combined configuration's task, workspace, model, and
owner references resolved. The proposed SQLite DDL executed: eight valid state
cases were accepted, thirteen invalid cases rejected, and a projection/event
transaction rollback preserved the original state. `git diff --check` passed.

These checks exercise document consistency, example syntax, and SQL constraints.
No runtime checker, Rust tests, live model benchmark, or deployed security test
has run. All implementation milestone checkboxes remain unchecked.

The preceding uncommitted draft was copied before edits to
`<TEMP_DIR>\Kinesin-before-adversarial-i9vlwjzp`.
