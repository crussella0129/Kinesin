# Sprint 0 End-to-End Tests

- **Status:** possible — executed.
- **Intent:** [INT-0001](../../../intents/INT-0001-token-accounting.md)
- **Tested head:** `c66a23d0e1e03702480b7314e73fce3c6a69609d`
- **Runner:** `cargo test --locked --lib cli::tests::test_cli_run_records_token_totals`
- **Result:** green.

## Executed tests

| Test | File | Acceptance criterion | Result |
|------|------|----------------------|--------|
| `test_cli_run_records_token_totals` | `src/cli.rs` | A completed run's **stored record** includes prompt/completion token totals when the server reports them | ok |
| `inspect_surfaces_token_totals_when_reported` | `tests/cli_inspect.rs` | The **`inspect` output** includes the summed token totals when calls report usage | ok |
| `inspect_omits_token_totals_when_unreported` | `tests/cli_inspect.rs` | The `inspect` output omits token totals (honest absence) when no call reported usage | ok |

## What they drive, end to end

**Storage read-back (`test_cli_run_records_token_totals`).** Drives the real CLI
surface — `parse(["run", "--config", …, "--workspace", "practice", "--model",
"local", "--prompt", …, "--allow-unchecked"])` then `execute(…)` — against a
loopback HTTP server bound to `127.0.0.1:0`. The one reply carries
`"usage":{"prompt_tokens":17,"completion_tokens":4,"total_tokens":21}`. After the
run settles, the test reopens the on-disk SQLite journal, lists the run, reads
its events, and asserts the terminal `run_finished` `counters` show
`prompt_tokens == 17` and `completion_tokens == 4`. This exercises the full path
the acceptance criterion names: real HTTP transport → Koil `usage` parse → runner
journalling → SQLite persistence → read-back through the run record.

**Inspect output (`inspect_surfaces_token_totals_when_reported`).** Runs the
real `kinesin` binary (`CARGO_BIN_EXE_kinesin`). A checked run makes three model
calls, each reporting `Usage{13, 5}`; the test then runs `kinesin inspect --run
<id>` as a subprocess and parses its JSON, asserting `counters.prompt_tokens ==
39` and `counters.completion_tokens == 15` (summed), plus `model_turns == 3`.
This is the acceptance criterion's "`inspect` output include … token totals"
half, proved through the actual command output rather than a raw store query.

> Gap found and closed in this phase: `inspect` never surfaced the terminal
> counters at all — its event summaries drop event bodies, and the counters ride
> in the `run_finished` event data, not the `RunRecord`. The build-plan's
> assumption that "inspect already surfaces … so tokens appear with no new
> command surface" was false. The fix lifts the counters from the `run_finished`
> event during the existing inspect scan into a `counters` field on the inspect
> output line (commit `c66a23d`). See critique C-001.

**Honest absence (`inspect_omits_token_totals_when_unreported`).** The same
binary path over a run whose scripted calls report no usage: `inspect` output
carries `model_turns` but omits `prompt_tokens`/`completion_tokens` entirely
(asserted `is_none()`), so unreported usage stays unknown, never zero.

## Confirmation

```
test cli::tests::test_cli_run_records_token_totals ... ok
test inspect_surfaces_token_totals_when_reported ... ok
test inspect_omits_token_totals_when_unreported ... ok
```

Determinism: the `src/cli.rs` fixture provider serves exactly one request with a
bounded accept deadline and a 30s read timeout; the port is OS-assigned, so there
is no shared fixed port or external dependency. The `cli_inspect.rs` tests
produce the run in-process (scripted, `Duration::ZERO`) and then inspect it as a
subprocess, asserting `no_model_calls()` so the inspect step touches no network.
The broader `complete_cli_batch_checks_more_tasks_than_controller_capacity` test
independently exercises the same loopback decode path across six checked runs
(three model calls each) and its fixture reply also carries `usage`, so the
production HTTP decode path is covered for both the non-stream single-turn and
multi-turn checked shapes.
