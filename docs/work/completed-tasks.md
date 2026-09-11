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

## T-001 (sprint 2)
- **Description:** add the pure `RunState::drop_oldest_compactable(floor)` to core — removes the oldest complete tool-call/result group or plain turn, preserving the system message, the initial user turn(s), the most-recent `floor` messages, and any evidence-bearing group (detected by a top-level `evidence_id` in the recorded tool result), with whole-group integrity. Add the `Compaction { enabled, floor }` config policy (default enabled, floor 6) with floor validation.
- **Intent:** [INT-0002](../intents/INT-0002-context-compaction.md)
- **Completed:** 2026-09-10T22:50:12Z
- **Files modified:** src/core.rs, src/config.rs
- **Commit:** `bf8825e400b30c9016dc93ee51ef90b170fbb1f5`

## T-002 (sprint 2)
- **Description:** compact at the history limit instead of stopping. Added `model::history_len` and the shared deterministic `model::compact_until_fits`; replaced the three runner stop sites and the mirrored replay sites with a compaction loop; added a `compactions` counter to the terminal counters (surfaced by inspect) rather than a separate event (a new event would break replay's positional planned/finished pair-walk). Refined replay's counter-divergence check to compare only the deterministically recomputable `model_turns`/`tool_calls`, ignoring runner-only observability totals (tokens, compactions). model.rs was a necessary touch beyond the plan's runner/replay for the shared pure helpers.
- **Intent:** [INT-0002](../intents/INT-0002-context-compaction.md)
- **Completed:** 2026-09-10T23:22:16Z
- **Files modified:** src/runner.rs, src/replay.rs, src/model.rs, tests/runner_tools.rs, tests/replay.rs
- **Commit:** `e095fd22ca9caba63b9b9e2ff68d1c422825af0f`

## T-001 (sprint 3)
- **Description:** emit llama.cpp `cache_prompt` in `prepare` (config-toggled via a new `ModelConfig.cache_prompt`, default on), threaded through `ModelOptions` and the runner/replay `options` builders; the flag is stored in the frozen config so replay recomputes the identical request. The static `tests/fixtures/live/*.request.json` files were left as dated provider captures (no offline test loads them — only the `.sse` responses are `include_bytes!`'d); current cache_prompt behavior is unit-tested instead.
- **Intent:** [INT-0004](../intents/INT-0004-kv-cache-reuse.md)
- **Completed:** 2026-09-11T05:12:14Z
- **Files modified:** src/model.rs, src/config.rs, src/runner.rs, src/replay.rs, tests/model_protocol.rs
- **Commit:** `56706885bd560d1c50a0745ddc6d190e6a61211d`

## T-002 (sprint 3)
- **Description:** offline reuse invariants + the live measurement harness. `prepared_request_of_each_turn_extends_the_previous` proves each turn's messages are a prefix of the next (the property the server's reuse relies on) and carry `cache_prompt`; `cache_prompt_does_not_change_run_outcome` shows the flag is transparent (identical candidate/acceptance on vs off); `replay_reproduces_a_cache_prompt_run` shows a cache_prompt capture replays consistent. The headline reduction is an `#[ignore]`d live benchmark (`kv_cache_reuse_reduces_prompt_eval_time`) that sends a prefix-extended pair and asserts the second evaluates fewer prompt tokens than its full prompt. The prefix-extension test landed in runner_tools.rs (its scripted `captured_requests` harness) rather than model_protocol.rs.
- **Intent:** [INT-0004](../intents/INT-0004-kv-cache-reuse.md)
- **Completed:** 2026-09-11T05:18:07Z
- **Files modified:** tests/runner_tools.rs, tests/replay.rs, tests/live_evaluation.rs
- **Commit:** `8be0e845b1c9fd9ff4bbb527e20349b1542ce929`

## T-001 (sprint 4)
- **Description:** address-privacy policy for model origins. Reworked `validate_origin` into explicit host classification (`origin_reach`: loopback, RFC1918, CGNAT `100.64.0.0/10`, and IPv6 ULA `fc00::/7` are private/overlay; a non-`localhost` domain or public IP is public) plus policy: private/overlay accepts http or https; a public host is rejected unless the new root `allow_public_endpoints` (serde default false) is set, and even then only over https. This lets a plain-HTTP overlay endpoint (e.g. a Tailscale `100.x` address) be configured exactly like localhost while public plaintext is still refused. CGNAT range implemented by hand (std helper unstable on 1.96.0).
- **Intent:** [INT-0008](../intents/INT-0008-remote-model-over-overlay.md)
- **Completed:** 2026-09-11T14:10:25Z
- **Files modified:** src/config.rs
- **Commit:** `32ab6d5aa3d86f7834a5aae3d24e580353630c7a`

## T-002 (sprint 4)
- **Description:** uniform-attach proof + "add a machine" runbook. `uniform_attach_prepares_identically_across_local_and_overlay_backends` runs the same scripted checked run against a loopback and a CGNAT-overlay `base_url` and asserts byte-identical prepared requests and identical acceptance (the address never enters the request body; the overlay config parsing at all is the T-001 win). `unreachable_backend_reports_not_ready` points a real HTTP client at a bound-then-dropped port and asserts `ready()` returns false within a bounded timeout (no hang). `attach_to_non_loopback_backend` (`#[ignore]`d, live) discovers the host's non-loopback IPv4 and asserts the pinned server answers identically over loopback and that address. Added the Tailscale "add a machine" runbook to docs/integration.md.
- **Intent:** [INT-0008](../intents/INT-0008-remote-model-over-overlay.md)
- **Completed:** 2026-09-11T14:16:05Z
- **Files modified:** tests/runner_tools.rs, tests/live_evaluation.rs, docs/integration.md
- **Commit:** `b2b19b401d6a6c2d0952ff3f544802326311eeb2`

## T-001 (sprint 5)
- **Description:** authored the seven workstream intent chapters for the roadmap — INT-0012 command-execution sandboxing (Landlock+seccomp / AppContainer+Job Object, mandatory-isolation mode), INT-0013 supply-chain security & signed releases (cargo-deny/audit/auditable/vet, SBOM, cosign), INT-0014 tamper-evident audit journal (hash-chain + signed receipts), INT-0015 threat model & security assurance (OWASP/NIST/CISA mapping + memory-safety statement + red-team corpus), INT-0016 observability (OpenTelemetry, no prompts), INT-0017 human-approval gates & JIT privilege, INT-0018 subagents & bounded parallel orchestration. Each is a well-formed v2 chapter; all `proposed` (INT-0018 lowest priority). Added SUMMARY navigation for each. check-book validates 18 chapters.
- **Intent:** [INT-0011](../intents/INT-0011-production-readiness-roadmap.md)
- **Completed:** 2026-09-11T16:59:42Z
- **Files modified:** docs/intents/INT-0012-command-execution-sandboxing.md, docs/intents/INT-0013-supply-chain-security.md, docs/intents/INT-0014-tamper-evident-journal.md, docs/intents/INT-0015-threat-model-assurance.md, docs/intents/INT-0016-observability.md, docs/intents/INT-0017-approval-gates-jit-privilege.md, docs/intents/INT-0018-subagents-parallel-orchestration.md, docs/SUMMARY.md
- **Commit:** `b18c7acb5286e6265106a9686fde0bd423a14b45`

## T-002 (sprint 5)
- **Description:** wrote `docs/roadmap.md` — the four-theme (A security hardening, B supply-chain & assurance, C operability, D SotA capability) prioritized roadmap with a current-state summary, recommended sequencing (INT-0013 → 0012 → 0015 → 0005/0006 → 0016/0017/0014/0010 → 0007/0009/0018), a SotA/standards mapping table, and a parking-lot (memory, programmatic tool results, ACP/A2A, Postgres/queue, vector retrieval). Added a one-line roadmap cross-reference to each existing proposed intent (INT-0005/0006/0007/0009/0010) and linked the roadmap from SUMMARY. Every research-report gap maps to a tracked intent.
- **Intent:** [INT-0011](../intents/INT-0011-production-readiness-roadmap.md)
- **Completed:** 2026-09-11T17:01:46Z
- **Files modified:** docs/roadmap.md, docs/intents/INT-0005-mcp-tool-servers.md, docs/intents/INT-0006-skills-progressive-disclosure.md, docs/intents/INT-0007-managed-model-process.md, docs/intents/INT-0009-koil-overlay-transport.md, docs/intents/INT-0010-cross-agent-write-coordination.md
- **Commit:** PENDING
