# Bounded reference study: evidence-driven development loops

On 2026-09-20 the user asked for principles from
[Animus_Sprint_Loops](https://github.com/crussella0129/Animus_Sprint_Loops),
following two failed live attempts. This read-only study viewed the repository
README/tree and seven selected files at main commit
`0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f`. Nothing was cloned, vendored or
executed. These are design recommendations for INT-0032/T-115, not a claim that
new progress verification has been implemented or that either attempt passed.

## What the reference actually does

- [Phase routing](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/open-harnesses/scripts/current-phase.sh#L19-L33)
  derives a phase from durable artifacts, open/completed tasks, a failure report,
  or a test report with accepted critique. A conversational completion claim is
  not its transition input.
- [Test](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/open-harnesses/particles/07-test-phase.md)
  and [loop](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/open-harnesses/particles/08-loop-phase.md)
  contracts distinguish successful verification, explicit failure and unresolved
  criteria. The loop reconciles intent against evidence before realization.
- [Sensitivity checking](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/tools/check-suite-sensitivity.sh#L75-L142)
  binds a passing control to the current committed tree and suite hash, rejects
  malformed/duplicate/stale evidence, then checks that disabling the subject
  makes the observation fail. It explicitly treats this as a minimum check,
  not proof that subtle errors will be detected.
- [Verification intent](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/docs/intents/INT-0013-verification-integrity.md#L34-L60)
  requires a no-change assertion to be paired with proof that the action actually
  ran successfully. Failure to execute cannot masquerade as successful restraint.
- [Research budgeting](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/open-harnesses/scripts/research-budget.sh)
  is mechanically bounded. This particular helper counts declared references;
  it is not an action-level stagnation detector.

The boundary matters: [check-book](https://github.com/crussella0129/Animus_Sprint_Loops/blob/0bdbe66f3f2b82584e4f8b44cbd3f5101f0dc69f/open-harnesses/scripts/check-book.sh#L49-L97)
checks lifecycle fields and evidence-link shapes. It does not independently
execute or prove the linked product behavior. The selected reference files do
not implement a general semantic progress oracle for arbitrary applications.
The proposals below extend its principles rather than copy an existing feature.

## Concrete application to Kinesin

Keep the useful separation between intended outcome, attempted operation and
observed result. Current milestone receipts prove that an authorized operation
ran; they do not prove the requested feature was delivered. Attempt 1's inert
cart after successful file writes is the concrete counterexample. A successful
read, command exit or write must never alone become a generic completion proof.

For an observable criterion, retain a bounded snapshot containing the frozen
requirement ID, relevant artifact hashes, observer/version identity, observation
status (`pass`, `fail`, `inconclusive`) and the actual tool/effect receipt that
produced it. Compare equivalent observations before and after a change:

| Classification | Evidence rule |
| --- | --- |
| Forward | At least one required criterion newly passes and no prior pass is lost. |
| Backward | A previously passing required criterion now fails; preserve any simultaneous gains separately. |
| No change | Comparable criterion states are unchanged after an action known to have run; record any artifact changes separately. |
| Unknown | Observations are missing, incomparable, stale or inconclusive, or code changed without an independent behavioral observation. |

This is a vector of criterion states, not a count of writes, tools or model
assertions. A changed hash is activity, not necessarily improvement. Lost or
invalidated evidence requires observation again; it cannot be silently retained
as a pass. Completion requires all required criteria currently passing under a
suitable observer; otherwise acceptance remains unchecked/inconclusive.
Where no behavioral criterion is observable, identical artifact/error
fingerprints can still detect operational stasis, but cannot prove semantic
progress or completion.

Use a small child loop for one unmet criterion: inspect the failure, perform one
bounded repair, observe again, then return its evidence to the parent. Children
share the original request, grants, run/time/tool budgets and repair allowance;
they cannot mint a new budget or weaken the criterion. Repeated identical
error/action/artifact fingerprints trigger a different bounded strategy or an
honest stalled result, never unlimited retries or a completion claim.

For this storefront, independently observing search, cart totals, checkout and
Clear Cart is materially stronger than fetching an HTML response. The existing
external browser rubric is the current observer. If runtime browser feedback is
added later, it should expose real page errors and interactions through granted
tools, with criteria bound to the user request. A model-authored check can itself
omit requirements; its existence is not independent acceptance evidence.

Keep the user's live-first order. Use disposable operation and repair to gain
confidence before official unit/integration checks. After the live gate, add
only focused regressions for the new transition rules and a small sensitivity
case proving that broken requested behavior makes the observer fail. Preserve
both failed attempts and measure elapsed time/internal retries/operator effort;
the target remains less babysitting than writing the app directly.
