# Plan Critique — Sprint 15

Scope: preliminary review of build-plan.proposed.md and test-plan.proposed.md
before user approval, using the installed plan-critic failure-mode screen.
This is not the canonical critique.md or evidence that plans were approved or
finalized. INT-0033 remains proposed; canonical plan files remain empty.

## Concerns

(none — revised scratch plans are clean per the failure-mode screen.)

## Confidence

clean

## Review and Disposition Record
Three independent read-only reviewers examined the proposed causal decision
sequence, action/replay boundary, operation facts/session design and matching
verification. No model calls, product edits or official tests were performed.

| Finding | Disposition in revised proposal |
| --- | --- |
| C-001: a prior tiny success could qualify a path after both held-out arms failed | T-122 now selects only from held-out passes, native wins ties, neither passes means stop; native fallback needs both create and amend. Named verification covers each branch. |
| C-002: streaming suppression lacked named coverage | T-119 E4 and structured_stream_does_not_leak_arguments cover valid answers, complete actions, malformed/truncated streaming and provisional argument suppression. |
| Short event pages are not EOF | T-120 requires contiguous scan through run_finished and separates known omitted records from an unknown tail at the cap. |
| Conflicting/duplicate event inputs could overwrite errors | T-120 marks affected results unknown/incomplete rather than last-write-wins success. |
| MCP/unknown operations could disappear from a compiled-tool summary | Unsupported activity is counted and conservatively labeled instead of presented as zero operations. |
| Session clipping could corrupt fact identities | Whole records/references are omitted; only prose is clipped. An answerless turn with no remaining reference is evicted. |
| Session memory could confound paired diagnostics | Each paired arm starts with independent empty context; only the explicit native create/amend fallback shares history. |
| Capture-version interaction was underspecified | Full capture/core/adapter/tools matrix is explicit, including old lab control, new reference versions and rejected combinations. New lab restrictions do not reject valid historical captures. |
| Later output-dependent histories cannot remain byte-identical | First request bodies are compared before dispatch; first branch/action is measured separately and later natural divergence is retained. |

The main critic re-read the revisions and returned clean. The operation/session
reviewer and wire/diagnostic reviewer independently confirmed their concerns were
addressed, with no remaining blocker in their respective scopes. Two final
clarifications preserve answerless-turn validity and historical capture admission.

Selected scope remains narrow: ordered protocol opt-in, deterministic operation
facts, partial-run reference, six diagnostic invocations and at most two qualified
full attempts. Broad browser automation and semantic completion machinery remain
deferred. A schema correction or honest failure cannot realize the usability
intent; official checks remain after the unchanged live confidence gate.

After user approval, a fresh canonical-plan critic and finalize-plan.sh remain
required by the skill. This preliminary clean verdict does not substitute for
that gate.
