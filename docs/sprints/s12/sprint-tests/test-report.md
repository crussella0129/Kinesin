# Sprint 12 test report

**Outcome: pass for INT-0030's scoped local assistant workflow.** Implementation
and a working real-model session preceded automated testing, as requested.

## Tested source and artifact

- Frozen context implementation: `b0ca1093239d6ecb0c50ef3a81f6f7f0ca6a4422`.
- Complete product implementation: `2fb3631f48a084f9379fa96bb678ca5c9e6436c1`.
- Final JSON assertion and evidence correction: `b46b352` (no product change).
- Tests ran against the working-tree content subsequently committed at those
  revisions. Later commits only finalize Book evidence and lifecycle state.
- Built/installed executable SHA-256:
  `751b64497a0c7bebc28c0d81c24a1029acc3fca0bb1272b46bf7eef191ce671c`.

## Verification

[Unit results](unit-tests.md): 206 passed.
[Integration results](integration-tests.md): CLI 17, replay 20, file-tool runner
25 and managed CLI 3 passed. Total **271 passing tests**, zero ignored in the
selected suites. Final JSON notice regression passed again after strengthening
its assertions. Formatting, all-target/all-feature Clippy with warnings denied,
and whitespace checks passed. The independent [final critique](critique.md) is
clean after C-001's JSON coverage correction.

[Native E2E](e2e-tests.md) verifies the installed bare command, working-folder
and model selection, owned startup, memory across requests, actual file creation
and edit, /context, /new and clean exit. Independent disk inspection and recorded
tool events establish effects; offline replay of the real edit is consistent.
The first model-quality failure is preserved alongside the successful runs.

## Acceptance reconciliation

- Bounded recent user/answer pairs retain run origin, with Unicode-safe clipping,
  oldest eviction, visible omissions and recovery after oversized/failed turns.
- Fresh authority and current workspace grants remain separate from history;
  metadata excludes transcript bodies, replay freezes and validates them, and
  legacy no-context captures remain supported.
- /new and /clear reset live history; /context reports bounds; JSON notices stay
  off stdout. Native local file work succeeds in a real conversation.
- Windows build, installation and implementation-first verification completed.

Hosted CI and native Linux tests were not run; no remote update is part of this
request. Memory is process-local and byte-bounded, not persistent history or real
token-window admission. Broader INT-0026 outcomes remain open. The local model
can still claim an operation without issuing its tool call; explicit edit
instructions produced the demonstrated real effect. This is not a general
task-reliability guarantee.
