# Sprint 15 diagnostic attempt ledger

All six authorized diagnostic requests were consumed. All used Qwen2.5-Coder-7B
Q4_K_M with the same pinned b6500 runtime, seed 0, temperature 0 and original
resource caps. Each app had a fresh session; source was not repaired between
arms. Every candidate run used source checkpoint `ad058c5`; slot 1 used the
retained historical executable. Binary hashes, not labels alone, identify them.

| Slot | Run ID | Capture/core/adapter/tools | Result | Consequence |
| --- | --- | --- | --- | --- |
| 1 | 72427c08-d777-4c84-af2e-44e8a6d97200 | 4/8/5/5 | Fail | Retain old-schema control. |
| 2 | 030c89b2-43d0-430b-bb83-d0a741f7af04 | 4/8/6/5 | Pass | Permit held-out comparison. |
| 3 | a40346e2-18c3-411e-a503-3e8bb0712bae | 4/7/4/5 | Fail | Native does not qualify on held-out task. |
| 4 | 5318c555-9eb6-4c15-b491-1d170adedd41 | 4/8/6/5 | Pass | Select ordered for repair pair. |
| 5 | 88a1318a-c848-44ca-afb6-4f34229410d6 | 4/8/6/5 | Fail | Unaided repair prerequisite unmet. |
| 6 | 2f6fb621-84f5-4323-9ff8-d63f7971e045 | 4/8/6/5 | Fail | Observation alone insufficient here; stop. |

- [Pair 1: schema ordering](pair-1-analysis.md)
- [Pair 2: held-out task](pair-2-analysis.md)
- [Pair 3: repair with/without observation](pair-3-analysis.md)
- [Frozen authentic browser observation](repair-preparation/observation.json)
- [Frozen unaided prompt](repair-preparation/slot-5-accepted-prompt.txt)
- [Frozen assisted prompt](repair-preparation/slot-6-accepted-prompt.txt)
- [Decision card](../../sprint-research/diagnostic-card.md)
- [Retained operator scripts and runtime identity](operator-glue/README.md)
- [Owned-process cleanup](cleanup.json)

The repair base sentence was preserved verbatim. The normal CLI treats each
physical LF as a new submission, so the assisted prompt inserts a space and the
recorded observation **before** its one terminal LF. This serialization was
frozen for both arms before dispatch and the relay matched actual first-wire
hashes. No embedded LF accidentally submitted an extra request. Slots 1–4 used
one-shot arguments; slots 5–6 kept ordinary CLI sessions open for possible owned
preview observation. Both ended without any preview and were then closed.

Operator corrections, app patches, tool-forcing messages and context resets:
zero in every arm. Slot 6 alone includes planned upfront assistance. Preparation
included creating isolated seed copies/profiles, materializing first-wire bodies,
running one shared runtime/relay and independently observing the seed. Evidence
work included scoring files, exporting journals and repairing slot 1's
postprocessing parser without rerunning it. This is not measured autonomous
end-to-end productivity; operator active time is unknown. Per-request and model
costs are recorded in the pair reports; whole-session waiting is not inference.

Forward progress is demonstrated only on the two small file criteria. Repair
remains failed, with unchanged artifacts and no behavioral pass. Relative
storefront progress is **unknown** because T-123 was correctly not run. Missing
full-workload observations cannot be counted as gains or lost passes.
