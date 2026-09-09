# Plan Critique — Sprint 0

## Concerns

### C-001: Intent consequence contradicts the research finding
- **Where:** `intents/INT-0001-token-accounting.md` Consequences vs `sprint-research/research-report.md` §2 and §5.
- **Quote:** intent says "a storage-schema version bump and a replay compatibility decision for older captures"; research says "**No SQLite schema change**" because usage rides in the schemaless `data_json`.
- **Failure mode:** intent-drift
- **Why it matters:** a changed consequence discovered in research must live in the stable intent, not only in the sprint report. Locking with the stale consequence leaves the semantic authority wrong.
- **Suggested response:** fix-in-plan — amend the intent Consequences and append a Transition-history note.

### C-002: Streaming replay fingerprint risk from T-002 is not explicitly covered
- **Where:** `build-plan.md` T-002 / `test-plan.md` replay coverage note.
- **Quote:** "added event fields are additive and optional; existing replay tests must still pass unchanged."
- **Failure mode:** missing-risk
- **Why it matters:** T-002 changes the streaming request body, so its `sha256` changes. Replay recomputes the prepared request and compares it to the recorded `model_planned` fingerprint. Any streaming replay capture recorded before T-002 would mismatch and fail — a distinct failure from asserting request bytes. The plan must direct T-002 to confirm no streaming replay capture exists, or update it.
- **Suggested response:** fix-in-plan — add the replay-fingerprint check to T-002 and to the test-plan replay note.

## Confidence
proceed-with-caveats

## Resolution
- C-001: addressed — INT-0001 Consequences amended (no migration; streaming
  fingerprint is the only replay interaction) with a Transition-history entry.
- C-002: addressed — T-002 build notes and the test-plan replay note now direct
  a check for streaming replay captures and their re-recording or confirmed
  absence.
