# Sprint 1 End-to-End Tests

- **Status:** possible — executed.
- **Intent:** [INT-0003](../../../intents/INT-0003-shell-execution.md)
- **Tested head:** `13ffdc3fcf1d7895d37629f3f46899780f7ce601`
- **Runner:** `cargo test --locked --test cli_inspect test_cli_run_command_effect_is_journalled`
- **Result:** green.

## Executed test (`tests/cli_inspect.rs`)
| Test | Acceptance criterion | Result |
|------|----------------------|--------|
| `test_cli_run_command_effect_is_journalled` | a freeform run in a granting workspace runs an allowed command with bounded output; the effect is journalled and surfaced | ok |

## What it drives, end to end
A real `kinesin run … --workspace practice --model local --prompt … --allow-unchecked`
subprocess drives a loopback HTTP model that returns a `run_command` tool call
(`["cmd-fixture","--print","hi"]`), then a prose answer once the tool result is
observed. The subprocess spawns the real fixture (inheriting the PATH that carries
it), captures its output, and persists to SQLite. The test then:
- opens the on-disk journal and asserts the `run_command` `tool_finished` event is
  `dispatch: executed`, `classification: ok`, with the replayed observation showing
  `exit_code: 0` and `stdout: "hi"`;
- runs a real `kinesin inspect --run <id>` subprocess and asserts its `counters`
  show `tool_calls == 1` and a `tool_finished` event summary is present.

This is the full path INT-0003 names: real CLI → real HTTP model → argv-only
process spawn → runner journalling → SQLite → read-back through the run record and
`inspect`.

## Inspect and event-body note
`inspect`'s event summaries deliberately drop event bodies (as the sprint-0
inspect work established), so the specific tool name and exit code are asserted
through the stored journal (`Command::Events`), while `inspect` output confirms the
run is surfaced via its `counters.tool_calls`. Surfacing per-event command detail
through `inspect` itself is out of this sprint's scope.

## Confirmation
```
test test_cli_run_command_effect_is_journalled ... ok
```

## CI matrix (T-005) — non-unit verification
The acceptance that the OS-specific process-tree cleanup and environment scrubbing
are exercised on both platforms is verified by the workflow content
(`.github/workflows/ci.yml` names a `strategy.matrix.os` of `[windows-latest,
ubuntu-latest]`) and by both OS jobs reporting green at the sprint checkpoint,
recorded under the test report's CI Confirmation. It has no `cargo` test because a
workflow matrix is configuration, not library behavior.
