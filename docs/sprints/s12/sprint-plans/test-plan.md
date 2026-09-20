Finalized - DO NOT EDIT

# Sprint 12 Test Plan

Implementation completes before any tests are written or run, as requested.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
| --- | --- | --- | --- |
| [INT-0030](../../../intents/INT-0030-usable-local-session-memory.md) | bounded ordered memory | T-110 / recent turns | session_memory_bounds_and_order |
| INT-0030 | fresh authority, no checked evidence | T-110 / invalid context | session_authority_rejects_invalid_context |
| INT-0030 | capture and replay | T-110 / metadata or replay | session_capture_and_replay; legacy_continuation_replay |
| INT-0030 | failure and oversize recovery, admission | T-111 / oversized or failed | session_followup_recovers_and_fits |
| INT-0030 | reset and status | T-111 / commands | session_context_commands |
| INT-0030 | local launch, recall and files | T-111 / installed model | local_assistant_walkthrough |

## Unit Tests
- `session_memory_bounds_and_order`: T-110; preserve prompts, answers, origin,
  Unicode-safe clipping, count/byte bounds and explicit omitted history.
- `session_authority_rejects_invalid_context`: T-110; malformed origin, oversized
  context, checked tasks and simultaneous legacy continuation are rejected.
- `session_followup_recovers_and_fits`: T-111; actual compiled-tool schemas counted,
  recent context shrinks without changing authority or poisoning future input.
  Dynamic MCP schema fitting remains under INT-0026; the runner's existing
  post-discovery request limit remains enforced.

## Integration Tests
- `session_capture_and_replay`: T-110; capture three turns, verify frozen model
  requests, no metadata prompt leakage, effect-free replay and digest rejection.
- `legacy_continuation_replay`: T-110; existing no-context captures still replay.
- `session_context_commands`: T-111; CLI process reset/status, JSON output and
  failure recovery. Inspect actual requests through the existing fake server.

## End-to-End Tests
- **Status:** possible
- `local_assistant_walkthrough`: T-111; build native Windows product, run local
  owned model with a disposable workspace, provide a unique detail, do file
  work, then request a follow-up that requires the detail. Independently inspect
  final disk contents and clean model-process shutdown.
- Run `cargo fmt`, `cargo clippy --locked --all-targets --all-features -- -D warnings`
  and focused affected suites after implementation. Broaden only if new failures
  or review findings justify it. Native Linux execution is outside this pass.
