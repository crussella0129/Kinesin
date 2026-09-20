# Sprint 15 Research Report

## Intents Reviewed
- [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md) — created,
  proposed; qualify actions and ground repair/session reference state in actual
  effects. The goal explicitly remains less babysitting than direct coding.
- [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) — selected,
  active and unrealized; preserve its zero-correction storefront/follow-up gate.
- [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md) — selected,
  realized and unchanged; INT-0033 is the follow-on for failed/partial-run facts.
- [INT-0024](../../../intents/INT-0024-harness-evaluation.md) — selected, proposed;
  broad benchmark/model comparisons remain backlog, not a Sprint 15 deliverable.

## 1. Sprint Goal
Replace guess-and-rerun development with a bounded causal investigation and the
smallest evidence-supported mitigations to action selection, truthful work state
and repair. Preserve all eight Sprint 14 failures. Qualify real effects before
adding workflow machinery, retain what actually happened between requests, and
measure progress against independent behavior rather than self-review or tool
counts. A truthful failure is not a working assistant. Operate the product in
disposable workspaces first; official unit/integration checks remain after live
confidence as the user requested.

## 2. Existing Code Survey
| File | Relevance | Notes |
| --- | --- | --- |
| src/model.rs | high | Ordered-wire mismatch in action_schema; native and historical structured paths. |
| Cargo.toml | high | Serializer configuration and existing Rust dependency boundary. |
| Cargo.lock | high | serde_json 1.0.151 has no preserve_order/indexmap path. |
| src/core.rs | high | Original objective protected; answer/recovery transitions and historical versions. |
| src/recovery.rs | high | One completion review, two nudges; 12-entry/4096-byte operation receipts. |
| src/runner.rs | high | Actual dispatch/results, budgets; response lifecycle Completed is not outcome proof. |
| src/replay.rs | high | Frozen core/adapter/tools versions and request hash reproduction. |
| src/tools.rs | high | Real writes/edits and errors; a failed operation may still have partial effects. |
| src/preview.rs | high | Owned preview and index compatibility scan; no browser behavior observation. |
| src/verification.rs | high | Checked file-fact semantics must not be repurposed as app behavior proof. |
| src/session.rs | high | Frozen bounded prompt/answer reference, not authority. |
| src/cli/session.rs | high | In-process byte/turn limits; successful answer history only. |
| src/cli.rs | high | Failed run effects omitted from remembered turns. |
| src/cli/presentation.rs | medium | User-facing lifecycle/candidate rendering needs honest interpretation. |
| src/storage.rs | high | Durable Event metadata and bounded event pagination can supply reference facts. |
| src/policy.rs | high | Fresh authority and context admission; reference facts cannot grant effects. |
| src/config.rs | medium | Explicit candidate opt-in and backward-compatible defaults. |
| src/onboarding.rs | medium | Preserve fresh/existing profile behavior and native default until qualified. |
| docs/sprints/s14/failure-report.md | high | Authoritative failed closeout; no feature completion. |
| docs/sprints/s14/sprint-tests/e2e-tests.md | high | Eight attempts, diagnostics, actual behavior and intervention costs. |
| docs/sprints/s14/sprint-research/reference-loop-principles.md | high | Existing pinned Animus study; activity versus outcome and freshness. |

The survey also used the retained attempt journals/profiles/artifacts and intent
chapters. Detailed provenance and quantified results are in the independently
prepared [attempt analysis](attempt-analysis.md).

## 3. External Sources
- [llama.cpp b6500 schema-to-grammar converter](https://raw.githubusercontent.com/ggml-org/llama.cpp/b6500/common/json-schema-to-grammar.cpp)
  — ordered JSON and required-property concatenation establish the wire-prefix
  mismatch. Downloaded source SHA-256:
  `af3b67e9f80bdc87ae0905f1619471be6b794fc4a7750b512f03ad9e0d194ec5`.
- [llama.cpp b6500 server utilities](https://raw.githubusercontent.com/ggml-org/llama.cpp/b6500/tools/server/utils.hpp)
  — the server's ordered JSON path supports the same integration finding.
- [Playwright browser contexts](https://playwright.dev/docs/browser-contexts)
  — isolated browser state is useful for observation, but is not OS containment.
- [Playwright BrowserContext API](https://playwright.dev/docs/api/class-browsercontext)
  — browser-state isolation and lifecycle options; no integration selected.
- [Playwright Browser API](https://playwright.dev/docs/api/class-browser)
  — owned browser lifecycle options considered in the bounded architecture audit.
- [Playwright MCP README](https://github.com/microsoft/playwright-mcp)
  — existing browser-server surface examined, not adopted or executed.
- [Playwright MCP options](https://playwright.dev/mcp/configuration/options)
  — origin lists exclude redirects and are not a security boundary; installing
  the stock server would not automatically satisfy owned-preview restrictions.
- [Microsoft Edge DevTools Protocol](https://learn.microsoft.com/nl-nl/microsoft-edge/devtools/protocol/)
  — an alternative browser-control boundary, not a selected implementation.

The prior pinned Animus study is reused rather than cloning or re-surveying the
repository. Its applicable principle is evidence-driven transitions; it does
not supply a general arbitrary-app correctness oracle. No browser library is
selected by this research and no new browser capability is claimed.

## 4. Risks, Unknowns, Dependencies

| Mechanism | Established evidence | Remaining uncertainty / decision |
| --- | --- | --- |
| Grammar and instructions disagree | Production tool keys serialize arguments/kind/name, answer keys kind/text; instructions teach kind first. The successful probe used a different order. | Strong hypothesis for answer-only attempts 7/8; not demonstrated as their cause or a cure for native app failures. Compare only ordering first. |
| Same model grades its own completion | Another answer can terminate after review despite no effects or dismissed warnings. | Receipt-backed truthful status can help honesty; it cannot implement features or prove behavior. |
| Activity substitutes for outcome | Writes, reads and preview startup occurred while required controls were absent or inert. | Need criterion/observer/artifact-bound comparisons outside model claims. A new runtime browser tool is not yet justified. |
| Failed effects vanish from session reference | CLI omits unsuccessful runs; successful answer prose is remembered. | Code-level epistemic gap; not proven to explain all failures because seven attempts never reached the follow-up. |
| Workflow stages create false failures | A real preview was rejected as the wrong stage; prior useful reads were not credited. | Do not rebuild a mandatory read/write/run ceremony. Qualify actions before generalized work-state transitions. |
| Runtime resources / model capacity | Four truncations occurred in attempts 4/6; no scored run exhausted time/turn/tools/history. About 99.7% of summed request time was model exchange. | More tokens are not a supported general fix; intrinsic model limits, prompt interference and repair ability remain unresolved. |
| Feedback availability versus use | Real warnings were dismissed; browser-fact diagnostics also failed or hit confounding stage gates. | Accurate feedback may help a qualified repair loop, but sufficiency needs a matched single-defect comparison. |
| Experimental confounding | Profiles, reasoning and caps changed together; no repeated matched baseline or human effort baseline. | Freeze each contrast and retain inconclusive outcomes; tiny diagnostic success cannot establish general improvement. |

Original objective retention is implemented; mechanical loss during compaction
is not supported by these runs. No blanket conclusion that native dispatch is
broken is supported either: the scored requests made 13 writes and seven mkdirs.
Zero command calls means Node command sandbox behavior did not cause these
scored application failures. Disposable workspaces remain necessary for safe
operation, without falsely claiming a Windows OS sandbox.

Dependencies: existing pinned b6500 runtime and two already-downloaded models;
adequate local disk; reproducible source/binary identity; bounded journal/effect
extraction; preserved capture and fresh authority contracts. The local native
core-7/adapter-4/tools-5 default remains unverified Sprint 14 source; the installed
assistant is still Sprint 13. Historical core-8/adapter-5 replay remains evidence,
not an endorsed production strategy. PR 14 is a draft failed-sprint archive.

## 5. Recommended Approach
1. Freeze a small causal decision card before any new model call: at most six
   diagnostic requests total, explicit predictions/falsifiers, fixed profiles,
   paths, wire requests and limits. First materialize the grammar discrepancy
   without inference, then compare the old and corrected schema ordering on
   identical tiny create/read/edit work. Both answer and action remain legal.
2. Implement a separately versioned ordered wire path in Rust, without globally
   enabling preserve_order or altering old captures. Keep the native default
   until live evidence supports a candidate. Actual correct effects decide
   qualification; JSON validity alone cannot pass it.
3. Derive a bounded machine-owned account of recorded operations and outcomes
   from existing durable journal events, separate from model claims. Missing
   results mean unknown outcome, and recorded errors have unverified repair
   status. Do not infer unresolved conditions or introduce a new semantic
   stalled state. Carry partial-run facts into session reference
   under existing admission/privacy limits. Do not promote operation success,
   a static warning scan or lifecycle completion into verified work completion.
4. Spend remaining diagnostic calls only on the next discriminating contrast:
   basic action/framing if effects fail, or a matched known-defect repair with
   and without authentic browser observations if effects work. No blind model,
   temperature, token or prompt sweep. Generalized browser automation and a
   semantic child-loop state machine are deferred until this evidence warrants
   their design; they are not hidden conditional implementation scope.
5. Preserve INT-0032's full workload and independent browser gate. Allow at most
   two fresh full attempts once basic create/edit and observed repair qualify.
   Record forward/backward/unchanged/unknown per comparable criterion, including
   regressions and operator effort. No app patches or corrective prompts count
   toward the accepted zero-correction attempt. If qualification or the full
   gate fails, retain the result and stop/replan explicitly. Run official focused
   unit/integration checks only after the live gate passes.

This plan addresses representational and control-loop failures first. It neither
promises that schema ordering explains all eight failures nor equates honest
failure reporting with a useful harness. The smallest supported change may still
fail the storefront; that outcome must remain visible rather than trigger more
unbounded experiments.

## Artifacts
- [Sprint 14 failed closeout](../../s14/failure-report.md)
- [Independent attempt analysis](attempt-analysis.md) — code/source observations,
  retained run IDs, timings, and candidate discriminating probes; no new live run.
- [Prior pinned reference study](../../s14/sprint-research/reference-loop-principles.md)
- [Sprint 15 metadata](../sprint-meta.md)
- [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md)
- [Proposed build plan](../sprint-plans/build-plan.proposed.md) and
  [proposed test plan](../sprint-plans/test-plan.proposed.md) — reviewable scratch
  proposals added in Plan Phase, not approved or finalized plans.
- [Pre-approval proposal review](../sprint-plans/proposal-review.md) — independent
  findings and dispositions; canonical final critique remains pending approval.

## Budget Override
The 21-row survey captures the main code paths, not the total number of retained
evidence files opened. Cross-cutting causal review also inspected all eight
attempt records, their profiles and generated artifacts, diagnostics, the local
serde_json map implementation, Book closeout/work records and four intent
chapters. Permit those already-retained inputs beyond the 20-file default because
the question is why eight different attempts failed across protocol, recovery,
browser output and session state. Permit eight external primary documents:
two pinned runtime sources and six browser lifecycle/API documents opened by the
parallel architecture audit. The latter are moving documentation as read on
2026-09-20; no browser implementation is selected. The prior Animus study is
reused. No broad repository clone,
new model survey or additional inference is part of research. The bounded
parallel audit completed within the phase's 30-minute research allowance;
subsequent plan drafting/review is a separate phase.
