# Sprint 15 implementation and live handoff

The user approved the reviewed plan on 2026-09-20. The canonical critic returned
clean and finalize-plan.sh locked both plans. No renewed approval is required for
the planned build or live diagnostics.

## Implementation checkpoint
- T-119 source adds the native/structured_ordered_v1 selector, adapter 6's final
  ordered serializer, consistent admission/runner selection and historical replay.
- T-120 derives bounded operation facts from owner-scoped durable event metadata.
  Missing/conflicting results stay unknown; failed command results may have effects.
- T-121 retains partial-run reference separately from answer prose, bounds/evicts
  typed facts atomically and reports response completion as unverified work.
- CLI session preflight uses the same recovery frame and selected adapter as the
  admitted run. Capture 5 is reserved for new run references; old captures remain
  on their recorded semantics.
- A no-inference Rust example prepares the exact first model request for diagnostic
  wire provenance. It opens no model connection or tool capability. This is bounded
  live preparation, not a unit/integration suite.

The combined native executable and request-preparation example compiled. Relevant
source was formatted. Implementation remains queued for live qualification and
post-live official verification; these notes do not complete T-119–T-124 or
realize either intent. No official unit/integration tests or Clippy ran before
the live gate. The installed assistant and production native default are unchanged.

## Bounded preparation decisions
Diagnostics use a single owned loopback runtime with a fixed served alias so
paired wire bodies do not differ because managed startup chooses random aliases.
The retained Sprint 14 attempt-7 executable supplies the historical adapter-5 lab
control under normal grants; the current candidate supplies adapter 6. Exact first
wire bytes must match the prepared bodies and differ only in schema property order.
Only designated seed copies are granted; prompts, profiles, controls and evidence
are disjoint. Actual source/binary/runtime/model/profile hashes are frozen before
dispatch, and every submitted diagnostic consumes a slot, including failures.

See the [diagnostic decision card](sprint-research/diagnostic-card.md). Source-only
reviews found and corrected command empty-resource handling and preserved missing
history, atomic fact eviction and historical capture boundaries. These reviews
are not official regression verification or functional success evidence.
