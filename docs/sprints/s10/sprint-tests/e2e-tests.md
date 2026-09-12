# Sprint 10 end-to-end verification

**Status: possible and executed.** Ordinary E2E cases below are contained in
the complete integration suite and are not counted twice.

| Flow | Evidence and result |
| --- | --- |
| CLI → checked run → inspect → export → pure replay | `inspect_export_and_pure_replay_preserve_durable_acceptance_without_models` removes the original runtime dependencies and reproduces durable acceptance; metadata/tampered exports refuse exact replay |
| CLI → command → journal | `test_cli_run_command_effect_is_journalled` verifies the actual command observation; command bounds and cleanup have native integration coverage |
| CLI → repeated freeform reads → compaction | `test_cli_run_compacts_past_history_limit` continues past the historical bottleneck; runner/replay tests separately protect checked evidence |
| Authenticated service → bounded MCP startup → cancel/terminal retrieval | All 16 service tests include queued/rejected/retried submissions with no extra spawn and admitted startup failures that remain pollable; existing owner/auth/idempotency contracts pass |
| MCP discovery → tool call → frozen capture → pure replay | `e2e_mcp_echo_run_and_replay` and startup-control/tamper tests reproduce tool outcomes without an external server or new authority |
| Windows client → authenticated SSH forward → separate Debian GPU server → checked local read → replay | Manual `secure_two_host_model_serving`: three model turns, one read, checked acceptance passed, capture-v3 replay consistent; direct tailnet model port inaccessible |
| Actual two-turn harness sessions → remote prompt cache | Manual `actual_session_cache_timing_measurement`: warmed and counterbalanced six pairs; 1,549 cached / 48 evaluated tokens in each measured second turn, actual prepared hashes agree with journal/replay |
| Stopped model/forward → checked CLI failure → export/replay | Exit 1, model_connection_failed, failed/inconclusive, no accepted task or tool effects; five-event offline replay consistent |

The operator supplied and established the second physical host after planning,
so the conditional remote evidence became executable during Build. Exact runtime,
model hashes, hardware, launch/forward commands, timings and cleanup observations
are preserved in [remote-deployment.md](remote-deployment.md).

Cache reuse was observed in both extension modes. `cache_prompt = false` omits
the field and does not disable b6500's cache; the measurements establish no causal
speedup from emitting it. Broader full-history continuity, concurrent model slots,
arbitrary remote endpoints and delegated remote MCP authorization remain proposed.
Other historical manual live-evaluation, load and chaos probes were not rerun as
part of these two scoped live checks.
