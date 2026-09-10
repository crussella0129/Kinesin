# Test Critique — Sprint 2

Adversarial read-only screen of the context-compaction evidence against the locked
plans and INT-0002's acceptance criteria.

## Concerns

### C-001: the replay counter-divergence check was weakened
- **Where:** `src/replay.rs` (T-002) — the terminal counter check.
- **Quote:** was `terminal.data["counters"] == json!({"model_turns":…,"tool_calls":…})`; now compares only `model_turns` and `tool_calls`.
- **Failure mode:** intent-drift (a consistency guard changed).
- **Why it matters:** replay is the immutability/reproducibility guardian; loosening a check there deserves scrutiny.
- **Suggested response:** defer-with-rationale / accept. The check now verifies exactly the counters replay can deterministically recompute (`model_turns`, `tool_calls`); the ignored fields (`compactions`, and the token totals from sprint 0) are runner-recorded observability that replay does not and cannot reproduce (it neither calls the model for usage nor journals compaction counts). The old exact-equality was already latently wrong for a usage-bearing capture; this refinement fixes that too. `replay_reproduces_a_compacted_run` proves the fingerprints still match, which is the substantive replay guarantee.

### C-002: the boundary tests depend on byte-tuned overflow timing
- **Where:** `tests/runner_tools.rs` compaction tests / `tests/cli_inspect.rs` E2E.
- **Quote:** `"x".repeat(700)` content with `max_history_bytes = 2600`, `floor = 2`.
- **Failure mode:** flake-risk (really: brittleness).
- **Suggested response:** defer-with-rationale. The tests are fully deterministic — fixed inputs produce identical serialized bytes every run, so they cannot flake; they are only *brittle* to a future change in conversation serialization overhead, which would fail them loudly (a `stopped` vs `completed` mismatch), not intermittently. The margins are chosen so overflow lands after the third bulky group, giving an old droppable group outside the two-message floor.

### C-003: observability is a counter, not a `history_compacted` event
- **Where:** `test-plan.md` (named a `history_compacted` event) vs. the implementation's `compactions` terminal counter.
- **Quote:** test-plan: "a `history_compacted` event is journalled"; "`kinesin inspect` shows a `history_compacted` event."
- **Failure mode:** e2e-drift.
- **Suggested response:** reject (the plan wording over-specified) with rationale. Replay walks the journal as strict `model_planned`/`model_finished` (and tool) pairs with positional `index += 2`; inserting a new event type between pairs would break that walk and the replay contract. A terminal `compactions` counter (surfaced by `inspect`, exactly like sprint 0's token totals and `tool_calls`) is the replay-safe equivalent and satisfies the intent's acceptance criteria, which require the run to *continue under a bounded policy*, not a specific event. The tests and E2E assert the counter accordingly.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — all four acceptance criteria (continue under a bounded ceiling; complete groups preserved; checked acceptance unaffected / evidence never dropped; replay+immutability hold) map to named executed tests across unit, integration, replay, and E2E.
- **Assertion weakness:** none — tests assert `phase`, `acceptance_status == passed`, `counters.compactions`, `report.consistency == consistent`, and group integrity (no dangling `tool_call_id`).
- **Stub leakage:** none — scripted clients carry contract data; the E2E uses a real binary, real HTTP, real SQLite, real `inspect`.
- **Integration drift:** none — the core drop unit tests, runner continue/stop/checked integration, replay reproduction, and CLI E2E each cover a distinct layer.
- **Negative-path absence:** none — `compaction_stops_when_nothing_droppable` and `drop_oldest_returns_false_when_only_protected_remain` cover the bounded-stop path.

## Confidence
proceed-with-caveats
