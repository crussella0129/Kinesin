# Completed Tasks Log (Append-Only)

## T-001 (sprint 0)
- **Description:** parse and carry model token usage on both response paths, threading it through the send API without touching the pure-core ModelReply
- **Intent:** [INT-0001](../intents/INT-0001-token-accounting.md)
- **Completed:** 2026-09-09T00:47:59Z
- **Files modified:** src/model.rs, src/runner.rs, src/scheduler.rs, tests/model_protocol.rs, tests/process_recovery.rs, tests/adversarial_runtime.rs, tests/replay.rs, tests/runner_journal.rs, tests/service.rs, tests/service_load.rs, tests/settlement.rs, examples/measure.rs
- **Commit:** `65c171c8dcc9e644c63fe64144796d0ca2e7c856`

## T-002 (sprint 0)
- **Description:** request streamed usage via stream_options.include_usage; resolved C-002 by verifying no streaming replay capture exists (replay.rs has no streaming) and that serde_json sorts keys so no other bytes shift
- **Intent:** [INT-0001](../intents/INT-0001-token-accounting.md)
- **Completed:** 2026-09-09T00:52:31Z
- **Files modified:** src/model.rs, tests/fixtures/live/text-stream.request.json, tests/fixtures/live/tool-call-stream.request.json
- **Commit:** `6f144bc44066572a50be9c12d7451264d9ecd030`

## T-003 (sprint 0)
- **Description:** journal per-call usage into each model_finished event and accumulate summed prompt/completion totals into the terminal counters, omitting token totals entirely when no call reported usage (honest absence, never zero)
- **Intent:** [INT-0001](../intents/INT-0001-token-accounting.md)
- **Completed:** 2026-09-09T01:01:57Z
- **Files modified:** src/runner.rs, src/cli.rs, tests/runner_tools.rs
- **Commit:** `069ad585198c1dcd97ae37186e89354116caf604`

## T-001 (sprint 1)
- **Description:** add `ToolName::RunCommand` (mutating, mints no evidence) and a per-workspace `commands` allow-list, with validation that grant and allow-list agree and each entry is a bare name; the checked-run bar covers it for free via `is_mutating`. Also route the new variant to a denial in the read capability.
- **Intent:** [INT-0003](../intents/INT-0003-shell-execution.md)
- **Completed:** 2026-09-10T16:26:05Z
- **Files modified:** src/config.rs, src/tools.rs
- **Commit:** `f5e1611970b7844db8a26c628965ce7a05624912`

## T-002 (sprint 1)
- **Description:** add the argv `command` field to `TypedToolArgs`, make `path` disjoint (run_command has none), extend `shape_for` so run_command requires a non-empty argv and forbids file fields while the file tools forbid a command, and add the pure `validate_command` (bare allow-listed `argv[0]`, non-empty argv).
- **Intent:** [INT-0003](../intents/INT-0003-shell-execution.md)
- **Completed:** 2026-09-10T16:34:06Z
- **Files modified:** src/tools.rs
- **Commit:** `3c200b84caa02ff24c3aebe6ec6851f45ead2c05`

## T-003 (sprint 1)
- **Description:** add the `CommandRunner` capability — argv-only spawn via `command-group` (Unix process group / Windows Job Object), cwd = workspace root, scrubbed environment (PATH everywhere plus a minimal Windows set), stdout+stderr drained and capped at `MAX_COMMAND_OUTPUT_BYTES`, timeout/cancel that kills the whole group, and defined outcomes (non-zero exit is Ok, missing exe / timeout are Error). Adds the cross-platform `cmd-fixture` test binary.
- **Intent:** [INT-0003](../intents/INT-0003-shell-execution.md)
- **Completed:** 2026-09-10T16:51:02Z
- **Files modified:** src/tools.rs, src/bin/cmd-fixture.rs, tests/command_tool.rs, Cargo.toml, Cargo.lock
- **Commit:** `88598bc5b399d869d36eec2edb643a7fafe96b53`

## T-004 (sprint 1)
- **Description:** wire run_command into the runtime — map the tool name, add a third dispatch route that runs the command on the async (killable) path beside the reader/writer, build a per-workspace `command_runners` map in `RunResources` (plus a `with_command_workspace` test builder), and add the `run_command` tool schema the model is offered. The effect rides the existing `tool_planned`/`tool_finished` events. Model schema (src/model.rs) was a necessary touch beyond the plan's stated src/runner.rs — without it the run stopped with "unsupported compiled tool".
- **Intent:** [INT-0003](../intents/INT-0003-shell-execution.md)
- **Completed:** 2026-09-10T17:01:42Z
- **Files modified:** src/runner.rs, src/model.rs, tests/runner_tools.rs
- **Commit:** `6f0f58eb507707dba4ad2e4cc449836910e5d8c9`

## T-005 (sprint 1)
- **Description:** convert CI from a single windows-latest job to a `strategy.matrix.os` of `[windows-latest, ubuntu-latest]` (fail-fast disabled), keeping the pinned 1.96.0 toolchain (via rust-toolchain.toml), the fmt/clippy/`cargo test --locked` steps, and the 20-minute timeout, so the OS-specific process-tree cleanup and env scrubbing are exercised on both platforms. Verification is non-unit: the workflow content plus both OS jobs green at the checkpoint.
- **Intent:** [INT-0003](../intents/INT-0003-shell-execution.md)
- **Completed:** 2026-09-10T17:03:34Z
- **Files modified:** .github/workflows/ci.yml
- **Commit:** `1d072b623f1c5ba15869a94119bf68b98ab21f0d`
