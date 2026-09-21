# Sprint 16 Research Report

## Intents Reviewed
- [INT-0033](../../../intents/INT-0033-evidence-driven-work-recovery.md) — selected
  and revised with AC6; active. Distinguish discovery, observed operations and
  actual repair; bound any automatic directory context without new authority.
- [INT-0032](../../../intents/INT-0032-low-intervention-local-workflows.md) —
  selected; active and unrealized. Keep the original goal: less time babysitting
  than writing the code, demonstrated by the unchanged zero-correction workload.

## 1. Sprint Goal

Find whether real workspace context is the missing prerequisite for useful
action, then conditionally supply the smallest demonstrated prerequisite in
the harness. Sprint 15's two explicit-file passes did not transfer to app repair:
both repair requests ended without a list/read/edit, even with real browser
symptoms. Preserve all failures and distinguish gains from unmeasured behavior.
Operate disposable apps first; official checks remain after a successful full
storefront and same-session follow-up. No further request is charged to Sprint
15, and no new request is authorized by this research report alone.

## 2. Existing Code Survey

| File | Relevance | Notes |
| --- | --- | --- |
| src/core.rs | high | Initial model context has no directory observation; model replies own pending tool batches. |
| src/runner.rs | high | Executes authorized tools with settlement ownership; turn-zero work needs explicit provenance. |
| src/policy.rs | high | Resolves effective grants and request bounds before execution. |
| src/cli.rs | high | Session admission and per-request memory; one physical input line submits one request. |
| src/replay.rs | high | Recorded versions and model-first sequence must preserve historical semantics. |
| src/model.rs | high | Ordered action wire offers list/read but supplies no actual workspace names. |
| src/onboarding.rs | medium | Existing interactive defaults must remain unchanged during experiments. |
| src/tools.rs | high | Bounded capability-based list/read already exist; listing is nonrecursive and may be incomplete. |
| src/config.rs | high | Trusted opt-in and eligibility belong in validated configuration. |
| src/storage.rs | high | Durable event and private-versus-metadata capture boundaries. |
| src/effects.rs | high | Facts need to distinguish harness-origin observation from model initiative. |
| src/session.rs | high | Typed reference versions and exact old encodings constrain new provenance. |
| docs/intents/INT-0032-low-intervention-local-workflows.md | high | Original usefulness/effort and live-first requirements. |
| docs/intents/INT-0033-evidence-driven-work-recovery.md | high | Evidence semantics and new bounded automatic-observation constraints. |
| docs/sprints/s14/failure-report.md | high | Eight distinct failures; neither stages nor syntax established usefulness. |
| docs/sprints/s15/failure-report.md | high | Exhausted six-slot decision and unverified implementation disposition. |
| docs/sprints/s15/sprint-research/failure-mechanisms.md | high | Available discovery tools were never dispatched; completion review accepted unsupported answers. |
| docs/sprints/s15/sprint-tests/diagnostics/pair-1-analysis.md | high | Narrow schema-order benefit, with binary/cache limitations. |
| docs/sprints/s15/sprint-tests/diagnostics/pair-2-analysis.md | high | Named-path ordered edits succeeded; native dependent batch failed. |
| docs/sprints/s15/sprint-tests/diagnostics/pair-3-analysis.md | high | Authentic behavior feedback did not initiate inspection or repair. |
| docs/sprints/s15/sprint-research/live-workload.md | high | Exact full prompts, original criteria/caps and same-session requirement. |
| docs/work/tasks.md | medium | T-125 research handoff and unverified prior implementation remain backlog. |

## 3. External Sources

No new external technical sources. This question is about this implementation
and its recorded failures; local source and retained live evidence are the primary
sources. The earlier Animus reference study remains prior-sprint provenance, not
a new framework to copy or authority to infer semantic progress from activity.

## 4. Risks, Unknowns, Dependencies

- **Unknown:** explicit filenames may change path salience or task framing,
  rather than uniquely remove a discovery limitation. Even source-assisted
  success is only sufficiency on that sample; four requests cannot establish
  reliability or human productivity.
- **Risk:** an initial directory observation could still be ignored. Do not
  implement it before the filename contrast supports trying it; require fresh
  normal-request qualification afterward.
- **Risk:** the current runner assumes model-origin tools and a prior model
  turn. A fabricated assistant call or pre-admission CLI read would hide effects
  from accounting and corrupt replay. Use an explicit harness-origin operation.
- **Risk:** directory names are untrusted content; partial or failed listing is
  not proof of absence. Do not guess source paths, recurse or attach file bodies
  automatically. A normal model-selected read uses the existing tool path.
- **Risk:** new origin fields may enter later session reference. Version that
  input as well as the initial operation; reject new shapes in legacy captures.
  Keep facts bounded and metadata capture free of directory bodies.
- **Risk:** the protected initial prefix can exhaust history/request bounds.
  Cap listing output at min(2,048, configured result cap), charge all framing,
  and fail/stop honestly if actual context does not fit; never raise limits.
- **Dependency:** keep the same Qwen2.5 Q4_K_M/b6500/seed-0/temperature-0 profile,
  adapter 6 and original caps. Freeze current source/binary and actual request
  bodies; no model/download/output/prompt sweep.
- **Dependency:** Sprint 15 implementation remains unverified. A full live pass
  must unlock focused verification of retained S14/S15 code as well as any new
  observation path before any successful closeout, install or promotion.
- **Dependency:** PR #15 is a draft failure archive. This research/plan stays
  local; no merge is authorized and its contents are not silently added to PR #15.

## 5. Recommended Approach

Use an adaptive **four-request maximum**, frozen before the first submission:

1. A: same unaided small-app repair on a fresh copy and current frozen build.
   If it passes, use request 2 for an unaided distinct held-out defect. Both
   passes permit the unchanged full workload; no new mechanism is needed.
2. If A fails, B: same request and seed with only an authentic bounded root
   filename observation added as declared diagnostic assistance. If B passes,
   implement the minimal opt-in automatic root listing, then use requests 3/4
   to qualify normal requests on the original and held-out fixtures. Both must
   pass, without user-supplied paths/context, before the full workload.
3. If B fails, C: same task with the complete bounded unmodified fixture source
   added to B's observation. If C fails, stop. If C passes, use request 4 for a
   source-assisted held-out defect and stop with retrieval findings either way.
   This branch does not implement automatic source reading or unlock storefront
   acceptance. No diagnostic replacement or fifth request is permitted.

This refines the independent [experiment option](experiment-options.md): after
a filename-only pass, spend the remaining two requests on the actual harness
mitigation and its transfer rather than another manual assistance trial. A
successful control avoids unnecessary source changes. The alternative of adding
listing unconditionally risks implementing another unproven layer; source-only
assistance could hide discovery failure. Both are rejected as automatic rollout.

The [source audit](grounding-options.md) identifies a small explicit grounded
initialization effect, existing filesystem capability execution, shared budgets
and a new replay/capture branch. Keep normal answer choice, checked acceptance,
native defaults and earlier wire bytes intact. This is one optional initial
observation, not a mandatory planning graph or completion oracle.

After genuine normal-request repair qualification, preserve the original full
Paper Harbor initial request and natural Clear Cart/item-count follow-up, maximum
two fresh attempts, zero corrective prompts/patches/tool forcing/context resets.
The second full attempt requires a specific in-scope generic harness fix to the
first failure; no blind rerun. Only a full pass unlocks official focused tests,
formatting, Clippy and final independent review. Failure preserves unverified
source and findings, with no product-completion claim.

## Artifacts
- [Grounding source audit](grounding-options.md) — existing mechanics and proposed
  bounds/provenance, not a new execution authorization.
- [Independent experiment option](experiment-options.md) — original alternative
  and inference limits; the selected refinement is above.
- [Prior failure assessment](../../s15/failure-report.md) and
  [exact workload](../../s15/sprint-research/live-workload.md).

## Budget Override

The cross-cutting source/replay/privacy question required 22 project files:
12 implementation modules, two stable intents and eight prior-evidence/work
records. This modest extension beyond 20 avoids omitting the retained failures
or typed-reference compatibility while keeping the candidate to one root listing.
No external technical source or model call was added. Research began around
00:44 UTC on 2026-09-21 and remained within the 30-minute wall-clock bound.
