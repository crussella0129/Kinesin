# Task acceptance

This is the authoritative per-run acceptance contract. It closes a gap in the
earlier plan: offline evaluations distinguished correct answers from completed
runs, but the runtime had no check or machine-readable verdict.

It does not solve verification of arbitrary natural-language goals. It makes
unchecked outcomes explicit and supports a small, independently checked class
of tasks. General-purpose execution and general-purpose proof of correctness
are different capabilities.

## Two separate outcomes

Rename the execution phase `succeeded` to **`completed`**. Execution phases are
`queued`, `running`, `cancelling`, `completed`, `stopped`, `failed`,
`cancelled`, and `interrupted`. There is no terminal `checking` state:
the initial pure check runs while the owning runner remains `running`.

Every run also has an acceptance status:

| Status | Meaning |
|--------|---------|
| `unchecked` | Freeform run; no acceptance contract was selected |
| `pending` | Checked task admitted, assessment not yet settled; never terminal |
| `passed` | Every required check in the frozen contract passed |
| `failed` | A required property demonstrably failed |
| `inconclusive` | Required checking could not establish an answer: missing evidence, checker fault, cancellation, or interruption |

`task_accepted` is derived, never client/model-supplied:
`phase == completed AND acceptance.status == passed`.
No HTTP status, final text, successful tool call, confidence score, or omitted
field substitutes for that conjunction. A missing assessment in a legacy/imported
record is not a pass.

A pass means **passed this identified contract over these observed inputs**.
It is not an unlimited correctness or safety certificate. Keep the contract's
scope in CLI/API output. Policy enforcement, execution completion, and task
acceptance remain separate measurements.

## Bind the requested task before running

Start with a concrete `TaskContract` enum: `Freeform` or `FileFieldsV1`.
Use `verification.rs` for owned definitions and a pure checker; no new crate,
plugin registry, model judge, or execution service is needed.

Trusted operator configuration defines task aliases, versions, workspace,
required fields, and checker identity. Owners may select only permitted aliases.
Task permission must also intersect the owner's workspace, tool, and model
permissions; a task alias cannot confer access to its configured resources.
Reject empty/duplicate criteria, unknown checkers, inconsistent versions,
unavailable tools, or conflicting workspace selections before admission.
The model, tool text, and final JSON cannot edit the contract.
Bound configured owner/alias and criterion identifiers to 64 ASCII bytes using
letters, digits, `_`, and `-`; checker IDs come from the compiled set. Validate
that the worst-case required receipt fits its cap before admitting a profile.

There are two disjoint submission shapes:

- **Freeform:** workspace/model aliases and a bounded user prompt. Always unchecked.
- **Checked task:** task alias and model alias; the task profile fixes workspace
  and generates the task instruction/output contract. No arbitrary additional
  `prompt`, `instructions`, criteria, expected answer, or workspace override.
  The first checker has no user-defined parameter language.

This prevents a caller from attaching an easy check to a different freeform
request and labeling the whole request accepted. Future parameterized profiles
need a typed, bounded parameter schema and must bind every goal-changing field.
An owner allowed freeform may still request unchecked work; it cannot acquire
a passed label or satisfy a service policy requiring checked tasks.

At acceptance freeze the profile ID/version, checker ID/version, effective
specification digest, generated task instruction, and required criteria into
`RunAuthority`. Versions/digests include the output and source-parser semantics.
Criteria may be disclosed as instructions; their authority comes from their
trusted origin, not from being secret. Never let the model edit its checker.

## First useful checker: fields from observed files

Implement `FileFieldsV1` as a deliberately narrow extraction task with one to
four required fields. Each has a unique criterion ID, approved relative file
path, and field key. All files belong to the profile's fixed workspace.
An optional required full-content digest constrains the expected revision.

A synthetic source file can contain:

```text
project=Kinesin
language=Rust
```

The task asks for the configured field values **as observed in complete successful
reads during this run**. It does not claim the files are true descriptions of
the world or that they remain unchanged at completion.

Specify a small source grammar: UTF-8, LF or CRLF lines, ASCII keys matching
`[a-z][a-z0-9_]{0,31}`, one `=` separator followed by a nonempty single-line
value of at most 1,024 UTF-8 bytes. Do not trim value whitespace or change case.
Blank lines and lines starting with `#` are ignored; all other malformed lines
or duplicate keys make the source inconclusive. Values may contain further
`=` characters. No interpolation or execution occurs. This is a simple scanner,
not a regular-expression interpreter exposed to users.

Require the entire final candidate to be a typed JSON object, for example:

```json
{
  "facts": [
    {"id": "language", "value": "Rust", "evidence_id": "e0"}
  ]
}
```

Use direct typed deserialization with unknown/duplicate JSON fields rejected.
Require exactly the configured set of unique fact IDs. Reject extra narrative,
code fences, duplicate facts, additional claims, and missing fields. Do not
first parse through a map that silently overwrites duplicate keys.
The renderer displays checked fields itself; a correct field cannot certify
contradictory unchecked prose alongside it. No provider `response_format` or
grammar extension is required: invalid output fails the check without retry.

For each field, independently resolve the actual observation, parse the source,
and compare the submitted value exactly with the selected key's value. A genuine
citation alone is insufficient. Unknown/forged/wrong-source evidence or a wrong
value fails; absent, incomplete, or ambiguous source data is inconclusive.
Check output shape, evidence binding, and source availability as distinct
properties. A missing source never passes; a candidate that also invents a
reference has a demonstrated binding failure. Use the aggregate precedence
below when failures and unavailable evidence coexist. A required key absent
from an otherwise complete valid source is inconclusive.
Conflicting complete successful observations of the same required path are inconclusive;
do not select whichever makes the answer pass. Identical repeated observations
may be reused. A required revision digest mismatch fails the declared revision check.
An incomplete prefix is never treated as a competing full-file revision.

A valid candidate can still be wrong under a weak or mistaken task specification.
Review the profile against the human's actual goal. Adding support for other
documents, reasoning, or current-world facts requires an appropriate new checker,
not widening this one's label.

## Runner-owned evidence

Only the runner issues a bounded `evidence_id` for an actual successful file
observation. Add that optional field to the serialized tool-result envelope,
within its existing byte cap, so the model can cite it. It is a reference,
not an authority token. IDs are unique inside one run.

Keep a private evidence record binding owner, run, harness effect ID, tool name,
normalized workspace-relative resource identity, observed bytes/content digest,
typed status, completeness, and journal observation sequence. Checkers receive
read-only access to this bounded inventory; no ambient filesystem or database
lookup capability. Provider call IDs and answer-supplied hashes do not create
records. Lookups never search another run or owner.

Failed/denied tools do not create usable evidence. An error body containing the
expected word cannot pass. Truncated reads cannot prove whole-file properties,
missing-field absence, or a complete-content revision digest. A path string or
basename cannot substitute for the required workspace/resource identity.

The evidence bytes are the same observations made available to the model; share
immutable storage where useful, but count retained memory even if history later
releases its copy. Do not reread mutable files during checking. Current-at-finish
or cross-file atomic-snapshot claims require a separately designed observation
boundary and are outside this checker.

## Bounds and failure rules

Proposed fixed bounds for the first implementation:

| Resource | Bound |
|----------|-------|
| Required field criteria | 1–4 |
| Serialized task specification | 8 KiB, also inside the configuration cap |
| Final candidate accepted by the checker | 8 KiB |
| Retained verification evidence per active run | 64 KiB, plus existing per-tool/history limits |
| Evidence records | At most the run's admitted tool-call limit |
| Serialized acceptance receipt | 8 KiB |
| Criterion diagnostic | Stable code and at most 256 UTF-8 bytes of sanitized explanation |

Exceeding retained-evidence capacity stops execution with
`verification_evidence_limit`; do not discard necessary observations and later
invent acceptance. An oversized candidate is a failed output-contract check.
A faulty checker or unusable source is inconclusive. Do not average successful
checks, skip unknown criteria, or let `.all()` over an empty set pass.

The initial checker scans bounded data once per distinct source and uses
constant-size field comparisons. Run it inline with no I/O, spawning, arbitrary
regex, recursion, or model call. Active-run admission bounds simultaneous retained
work. Measure its runtime; a wall-clock wrapper cannot preempt arbitrary
synchronous code. New expensive checkers require a separate resource design.

Assessment consumes the remaining execution budget. Check time/cancellation
before and after it. The settlement grace records existing outcomes; it never
authorizes launching or repeating a checker after expiry. If the deadline or
cancellation is observed before finalization, use the relevant execution outcome
and `inconclusive` for a checked task. Do not publish partial passes.

## One finalization path

1. Normalize a complete provider answer into an **answer candidate**, not a
   terminal success. Finish its model-observation journal acknowledgement.
2. Release model capacity, retain active-run ownership, and check cancellation
   and remaining execution time.
3. For freeform, choose `unchecked`. For a checked task, run the frozen pure
   checker against candidate bytes and actual bounded evidence.
4. Produce a bounded receipt. On normal completion: any checker fault yields
   inconclusive; otherwise a demonstrated required failure yields failed;
   otherwise unresolved evidence yields inconclusive; only every required pass
   yields passed.
5. Reserve bounded storage-inbox capacity, then recheck time/cancellation after
   that wait. Apply an already observed cancellation/expiry instead of publishing
   a pass. Transfer the terminal command synchronously using the reservation;
   do not insert another unchecked await between arbitration and submission.
6. Submit one transaction containing result/candidate, result fingerprint,
   acceptance receipt/projection, and terminal event. Settle its acknowledgement.
7. Only committed state is final. GET, list, SSE, export, and CLI expose both
   outcomes and the derived `task_accepted`. Provisional text stays provisional
   throughout checking.

**Terminal-command submission is the cancellation arbitration boundary.** Once
that command is accepted into the storage inbox, do not assume it can be recalled
or overwritten. The controller settles it; a later cancel/deadline cannot rewrite
its committed result. Before submission, observed cancellation wins. This replaces
the earlier rule that loosely promised cancellation precedence until commit.

If storage admission/commit fails, no durable accepted result is advertised.
If the process crashes before terminal commit, recovery marks the run interrupted
and its checked acceptance inconclusive. If commit completed, both result and
receipt survive together. No post-terminal checker, automatic model repair,
reassessment on GET, or automatic re-verification after restart.

## Receipt, privacy, and retries

A receipt records acceptance status/reason, contract and checker identity/version,
specification digest, exact candidate digest, required criterion results, observed
evidence IDs/sequences/digests, scope, and duration. The runner binds owner and run.
For interruption or cancellation before a candidate exists, record its digest
as absent; do not manufacture an empty candidate or hash. Retain the frozen
required criterion IDs at admission even in metadata mode so recovery can
construct an inconclusive receipt without loading today's profile.
Bound all fields. Public diagnostics do not disclose hidden expected values,
unrelated file paths, raw exception text, or tool bodies.
Hash exact candidate UTF-8 bytes after provider decoding, before trimming,
parsing, or rendering. Evidence digests cover the exact observed tool-body bytes;
only a complete observation supports a full-content digest.

Retain bounded final candidates even when rejected or unchecked so the owner can
inspect them; the same sensitive-result exception applies in metadata mode.
A passed structured result includes only verified fields rendered by the harness.
Do not add a durable copy of every evidence body to metadata capture.
A stored receipt is a historical verdict, not enough data to recompute it.

Replay capture must retain the frozen specification and exact evidence inputs
necessary to rerun that checker. Without them, report verification replay
unavailable; never rebuild evidence from today's workspace. SQLite and digests
do not make this a cryptographically signed attestation against a compromised
controller or database administrator.
Compare semantic verdicts, check results, and input bindings on replay; a
remeasured duration need not equal the original recorded duration.
Imported receipts remain untrusted data; never accept their claimed status into
the live result store as a verified verdict. Effect-free replay checks internal
consistency against captured inputs, not authentic origin by itself.

Idempotency hashes the versioned requested submission, including mode and task
alias (plus typed parameters if a future profile adds them). The accepted run
separately freezes the effective specification digest/version. An identical retry
returns the original run and verdict, even after profile edits; no silent rejudge.
A changed task/mode under the same key conflicts. Recheck current owner access,
but do not substitute today's profile for the original snapshot. A new evaluation
requires a new run/key.

## CLI and service meaning

The strict CLI default returns:

| Outcome | Exit |
|---------|------|
| Completed and passed | 0 |
| Execution stopped/failed/interrupted or invalid setup | 1 |
| Completed and acceptance failed | 2 |
| Completed but unchecked/inconclusive | 3 |
| Cancelled | 130 |

For an intentional freeform demonstration, `--allow-unchecked` permits exit zero
for `completed + unchecked` only. It never applies to a checked task, changes
the receipt, or labels it accepted. Reject that flag with a checked-task selection.
Display “Completed; unchecked” even with the opt-in. Machine consumers should
read the receipt; exit policy is not an acceptance claim.

A batch returns zero only if every item satisfies its applicable exit rule.
Otherwise use priority cancellation, execution failure, failed acceptance, then
unchecked/inconclusive; retain each individual outcome.

HTTP 202 means admission, and HTTP 200 means retrieval. Neither means task
acceptance. Terminal SSE carries acceptance and closes only after the combined
commit is caught up. Clients requiring correctness must inspect
`task_accepted` and the contract scope/version, not just an HTTP code or phase.

## What this solves and leaves open

This design can reject a confidently wrong extracted field even when the correct
file was read and cited. It cannot certify an unrestricted essay, diagnose every
incomplete rubric, or prove that observed files describe reality. Freeform work
remains unchecked; human review or a future calibrated evaluator can assess it
without being mislabeled as deterministic proof.

Offline evals remain essential to test both the agent and the checker, including
false positives, false negatives, renamed inputs, hostile evidence, and omitted
criteria. Runtime acceptance reuses a tested checker on each applicable run;
it does not replace those evaluations. See [adversarial review](adversarial-review.md)
and [testing](testing.md).
