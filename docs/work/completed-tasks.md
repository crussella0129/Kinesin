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
- **Commit:** `f80a0321ae91732bba3577518d41afcb3da6fef8`

## T-001 (sprint 6)
- **Description:** authored `deny.toml` (cargo-deny v2 schema, deny-by-default) validated green against the current 221-crate tree via `cargo deny check` (advisories/bans/licenses/sources all ok) and `cargo audit` (0 vulnerabilities). License allow-list is tightened to the licenses actually present per `cargo deny list` (MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-3-Clause, ISC, Unicode-3.0, Zlib, CC0-1.0, CDLA-Permissive-2.0 for webpki-root-certs); multi-licensed copyleft/other identifiers (LGPL/Unlicense/BSL/MIT-0) are intentionally not allowed since those crates satisfy via their MIT/Apache option. `sources` allows crates.io only; `bans` denies wildcards, warns on duplicates. Added `publish = false` to Cargo.toml and `private.ignore = true` so the crate's own missing-license field is skipped. Note: the locked plan's `cargo audit --locked` was a misnomer — cargo-audit reads Cargo.lock by default and has no `--locked` flag; T-002 uses plain `cargo audit`.
- **Intent:** [INT-0013](../intents/INT-0013-supply-chain-security.md)
- **Completed:** 2026-09-11T19:15:56Z
- **Files modified:** deny.toml, Cargo.toml
- **Commit:** `d043acd98e8ca752cd76fc689ae42d4f2379e37d`

## T-002 (sprint 6)
- **Description:** added a blocking `supply-chain` CI job to `.github/workflows/ci.yml` (ubuntu-latest, top-level `permissions: contents: read`, `actions/checkout@v7` with `persist-credentials: false`, pinned toolchain 1.96.0). It installs cargo-deny + cargo-audit via `taiki-e/install-action@v2` (prebuilt binaries; pinned to the major tag, consistent with the repo's `checkout@v7`) and runs `cargo deny check` and `cargo audit` as blocking steps. The existing windows+ubuntu fmt/clippy/test `check` matrix is unchanged. YAML validated (two jobs). Corrected the plan's `cargo audit --locked` misnomer to plain `cargo audit`. Verification is non-unit: the job green on the checkpoint PR.
- **Intent:** [INT-0013](../intents/INT-0013-supply-chain-security.md)
- **Completed:** 2026-09-11T19:17:08Z
- **Files modified:** .github/workflows/ci.yml
- **Commit:** `550a3a55d861cb0283d73963b0a948cc208db36d`

## T-001 (sprint 7)
- **Description:** added `landlock` 0.4.7 + `seccompiler` 0.5.0 (and transitive `enumflags2`) under `[target.'cfg(target_os="linux")'.dependencies]` so they compile only on Linux and leave Windows/macos builds untouched. The INT-0013 supply-chain gate stays green with the new deps: `cargo deny check` all-ok (their MIT/Apache licenses were already allowed — no deny.toml edit) and `cargo audit` clean (225 deps). `cargo check` confirms the Windows build is unaffected (cfg-gated deps not pulled).
- **Intent:** [INT-0012](../intents/INT-0012-command-execution-sandboxing.md)
- **Completed:** 2026-09-11T20:33:54Z
- **Files modified:** Cargo.toml, Cargo.lock
- **Commit:** `3c58c9c1832673b84617a82b443af2ee1e8eb403`

## T-002 (sprint 7)
- **Description:** added the mandatory Linux command sandbox to `CommandRunner` (`src/tools.rs`, new `#[cfg(target_os="linux")] mod sandbox_linux`). Before `group_spawn`, `arm()` builds — in the parent — a Landlock ruleset (read-execute on existing system prefixes /usr,/lib,/lib64,/bin,/sbin,/etc,/proc plus the resolved command binary's directory; read-write on the workspace root; deny the rest) and compiles a seccomp-BPF filter denying network syscalls (socket/socketpair/connect/bind/listen/accept/accept4/sendto/sendmsg → EPERM), then applies them in an `unsafe` `pre_exec` closure (apply-only: `restrict_self` + `apply_filter`, no allocation). If Landlock is NotEnforced or the ruleset/filter can't be built, it refuses with a defined `sandbox_unavailable` error (mandatory, secure-by-default) — never spawns unconfined. `resolve_binary` PATH-resolves the allow-listed name so the binary (incl. a test fixture in target/) stays executable. Verified in WSL (kernel 6.6, Landlock+seccomp enforced) and clippy-clean on Linux; cfg-gated so Windows/macos are unchanged.
- **Intent:** [INT-0012](../intents/INT-0012-command-execution-sandboxing.md)
- **Completed:** 2026-09-11T20:51:38Z
- **Files modified:** src/tools.rs
- **Commit:** `e9fb97b2e3e4be8008dbff8a8fe8e3f3da38f489`

## T-003 (sprint 7)
- **Description:** Linux sandbox enforcement tests (`tests/sandbox_linux.rs`, `#![cfg(target_os="linux")]`) driving the real `cmd-fixture` through `CommandRunner`: `sandbox_denies_out_of_workspace_read` (Landlock denies a read of a file beside the workspace under /tmp → `READ_DENIED`/exit 21), `sandbox_denies_network_socket` (seccomp denies UDP socket creation → `SOCKET_DENIED`/exit 22), `sandbox_allows_in_workspace_work` (in-workspace read → `READ_OK`), and `sandbox_is_mandatory` (a command is either enforced-Ok or refused with `sandbox_unavailable`, never unconfined). Each enforcement test skips with a note on a kernel that cannot enforce (mandatory-refuse). Added `--read-file`/`--open-socket` probe directives to `cmd-fixture`. All 4 pass in WSL with real enforcement; full suite green on WSL + Windows; the ubuntu CI `check` job is the authoritative Landlock run.
- **Intent:** [INT-0012](../intents/INT-0012-command-execution-sandboxing.md)
- **Completed:** 2026-09-11T20:52:14Z
- **Files modified:** tests/sandbox_linux.rs, src/bin/cmd-fixture.rs
- **Commit:** `1d0dc92209d7e829b16965ef7727f6afed517252`

## T-001 (sprint 8)
- **Description:** authored `docs/threat-model.md` — the standards-mapped security-assurance package: an OWASP LLM Top-10 (2025) + Agentic Top-10 (2026) table mapping each risk to a mechanism / gap (owning intent) / accepted residual risk; a NIST AI-agent controls table (policy-based authz, task scoping, JIT-privilege gap→INT-0017, approval-gates gap→INT-0017, dev/test/prod residual, transport-auth mechanism+mTLS residual boundary); a CISA/NSA memory-safety statement enumerating every unsafe/FFI site (private_state.rs Windows security APIs + libc::geteuid, storage.rs CreateDirectoryW, signal.rs SetConsoleCtrlHandler, tools.rs Linux sandbox pre_exec) with the "no unsafe in the pure core" posture; and a red-team-corpus → release-evidence-matrix table naming the executed test(s) per row. Versioned (v1, owner, cadence). Linked from SUMMARY.
- **Intent:** [INT-0015](../intents/INT-0015-threat-model-assurance.md)
- **Completed:** 2026-09-11T22:01:43Z
- **Files modified:** docs/threat-model.md, docs/SUMMARY.md
- **Commit:** `5b4c631b679fa98d6ed63b88c149ec1c62789401`

## T-002 (sprint 8)
- **Description:** added `tests/redteam.rs`, the labeled consolidating red-team corpus: `redteam_denies_unauthorized` (a command not on the workspace allow-list is denied at the policy gate before any spawn — a forbidden effect cannot pass) and `redteam_treats_content_as_data` (a command argument crafted as an injected instruction with shell metacharacters is delivered verbatim as one argv element and echoed literally — argv-only, never a shell — creating no `owned.txt` side effect; tolerant of the mandatory-sandbox refuse path on a non-Landlock kernel). Both green on Windows and WSL; referenced by the docs/threat-model.md corpus→matrix map rather than duplicating the broader adversarial/auth/sandbox suites.
- **Intent:** [INT-0015](../intents/INT-0015-threat-model-assurance.md)
- **Completed:** 2026-09-11T22:03:43Z
- **Files modified:** tests/redteam.rs
- **Commit:** `14fc67a8a2f87adecc498e27d0acf18b286c19e0`

## T-001 (sprint 9)
- **Description:** generalized the tool allow-list identity to `ToolRef` (config.rs) — `Compiled(ToolName)` ∪ `Mcp { server, tool }` with string serde (`mcp__<server>__<tool>`), `wire_name`/`is_mutating` (MCP = true, barred from checked runs)/`mints_evidence` (MCP = false)/`parse`; added `ToolName::from_wire` to DRY the wire→variant map. Changed `WorkspaceConfig.tools` and the per-owner override to `Vec<ToolRef>`, generalized `allows_tool` (policy) and every allow-list call site in config/policy/runner/replay to `ToolRef` (compiled arms wrapped `ToolRef::Compiled`); existing TOML tool configs parse unchanged. Unit tests `toolref_compiled_roundtrip`, `toolref_mcp_parse`, `toolref_mcp_barred_from_checked_run`.
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T00:46:06Z
- **Files modified:** src/config.rs, src/policy.rs, src/runner.rs, src/replay.rs, docs/intents/INT-0005-mcp-tool-servers.md
- **Commit:** `8d13879d67c61fac9fb38bbc84ccc9a9f78a842e`

## T-002 (sprint 9)
- **Description:** added the operator MCP server declarations — a root `[[mcp.servers]]` section (`McpConfig`/`McpServer { id, command }`) on `Config`/`ConfigFile`, and validation in `Config::parse`: unique server ids (via `unique_ids`), a non-empty command with a bare executable, a server id free of the `__` namespacing separator, and the identity gate — every `mcp__server__tool` referenced by any workspace or owner allow-list must name a declared server (else parse fails). Tests `mcp_config_accepts_declared_server_and_tool`, `mcp_config_rejects_undeclared_server`, `mcp_config_rejects_dup_or_empty`.
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T00:52:00Z
- **Files modified:** src/config.rs
- **Commit:** `166ecdd482bc071929f27a9858e76f5b4da54700`

## T-003 (sprint 9)
- **Description:** added the `mcp` module (`src/mcp.rs`): `McpToolDef { server, tool, description, input_schema }` (the frozen tool record), `McpClientPool` with `connect` (spawn+`initialize` each needed operator-declared server over the official rmcp v3.3.0 `TokioChildProcess` stdio transport, bounded by `MCP_STARTUP_TIMEOUT`) and `discover` (list each server's tools once, extract the allow-listed tools' schemas, error on an absent tool or a list/connect timeout — C-003), plus `needed_servers`. Froze the discovered set into the run: `mcp_tools: Vec<McpToolDef>` on both `RunAuthority` (with `mcp_tools()`/`with_mcp_tools`) and `FrozenContext`, each `#[serde(default, skip_serializing_if = "Vec::is_empty")]` so a non-MCP run's frozen bytes and pre-feature journals stay byte-identical (replay parity). Added a per-run `mcp` pool field + `with_mcp` builder to `RunResources`, `Config::mcp_servers()`, and the CLI discovery-before-submit step (`Startup::prepare_mcp`) wired into the single, interactive-session, and batch run paths. Test `non_mcp_authority_omits_mcp_tools_while_added_ones_freeze`; non-MCP replay parity confirmed by the full replay suite staying green.
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T01:08:08Z
- **Files modified:** Cargo.toml, Cargo.lock, src/mcp.rs, src/lib.rs, src/policy.rs, src/replay.rs, src/runner.rs, src/config.rs, src/cli.rs
- **Commit:** `9e85b5388400b1dacc1c3e4eadd093d13e92d932`

## T-004 (sprint 9)
- **Description:** emit discovered MCP tool schemas to the model, identical on live and replay. Changed `ModelOptions.tools` from `Vec<String>` to `Vec<ToolDef>` (`Compiled(name)` | `Mcp { name, description, input_schema }`, with `From<&str>`/`From<String>`); the `prepare` compiler now emits the fixed schema for a compiled tool and the server-discovered `input_schema` verbatim for an MCP tool. Added the shared `model::tool_defs(allow, mcp)` builder (preserves allow-list order, pairs each MCP ref with its frozen def) and pointed both `runner::options` and `FrozenContext::options` at it, so both paths read the same frozen source and produce byte-identical requests. Compiled-tool schemas and replay parity confirmed unchanged (lib/model_protocol/replay/runner_tools green).
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T05:12:58Z
- **Files modified:** src/model.rs, src/runner.rs, src/replay.rs
- **Commit:** `a8f909ca0b7c1d62d2671c53f3ccaa8e009be46b`

## T-005 (sprint 9)
- **Description:** live MCP dispatch through the existing gate. Added `McpClientPool::call` (invoke `tools/call` over the run's stdio session, bounded by the tool permit's deadline + cancellation, mapping `CallToolResult` content into an untrusted `ToolResult` — text concatenated, non-text noted by kind, body bounded to `max_tool_result_bytes` on a char boundary, `is_error` → status Error, mints no evidence), plus `validate_args` (lightweight object + required-key check against the discovered `input_schema`; C-002) and `bound`. Restructured the runner dispatch ladder: `ToolName::from_wire` for compiled tools, else match the frozen `mcp_tools` set; an MCP call validates its args and is denied pre-dispatch when unapproved or malformed (no server contacted — C-001); execution routes an approved MCP call to the pool as a sibling of the reader/writer/command paths, holding the permit. Unit tests `validate_args_requires_object_and_required_keys`, `bound_truncates_on_a_char_boundary`, `tooldef_wire_name_matches_toolref` (fixture-driven approved/denied/byte-cap tests land with T-007).
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T05:19:04Z
- **Files modified:** src/mcp.rs, src/runner.rs
- **Commit:** `db3ee55fdd9ea93aef1db8b2e8aee0bfab196727`

## T-006 (sprint 9)
- **Description:** replay MCP dispatch without reconnecting. Extended the replay dispatch ladder (replay.rs) to recognize an MCP tool from the frozen `mcp_tools` set (never a live client) and mirror the live denial ladder (`ToolName::from_wire`-equivalent compiled match kept as-is incl. its run_command omission; else frozen-set lookup + `validate_args`; well-formed-but-ungranted `mcp__` → tool_denied; else unknown_tool), so the recorded `dispatch` classification validates consistently and the recorded observation is reproduced. No `McpClientPool` is constructed on the replay path. Existing replay suite green; the fixture-driven `replay_reproduces_mcp_run_without_reconnect` + e2e land with T-007.
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T05:20:50Z
- **Files modified:** src/replay.rs
- **Commit:** `4c9b2c2630da64049480034186870930b09f40d9`

## T-007 (sprint 9)
- **Description:** in-repo fixture MCP server + integration tests. Added `src/bin/mcp-fixture.rs` — a stdio MCP server built on the same rmcp SDK (guaranteeing the handshake) exposing `echo` (verbatim) and `poison` (prompt-injection text in both its description and result), plus a `--hang` mode for the discovery-timeout test. Enabled rmcp `server`/`transport-io`/`macros` features and named `schemars` directly (all already in the tree via rmcp — no new crates; `cargo deny`/`audit` stay green: 262 deps, 0 vulns). `tests/mcp.rs` drives real runs against the fixture via the scripted-model harness: `mcp_approved_call_bounded_no_evidence`, `mcp_result_truncated_at_byte_cap`, `mcp_unapproved_call_denied`, `mcp_poison_description_and_result_are_data` (injection is data — no owned.txt, allow-list unchanged), `mcp_missing_allowlisted_tool_fails_start`, `mcp_discovery_timeout_fails_start` (C-003), and `e2e_mcp_echo_run_and_replay` (replay reproduces consistent without reconnecting — T-006).
- **Intent:** [INT-0005](../intents/INT-0005-mcp-tool-servers.md)
- **Completed:** 2026-09-12T05:41:45Z
- **Files modified:** Cargo.toml, Cargo.lock, src/bin/mcp-fixture.rs, tests/mcp.rs
- **Commit:** `73fb0d781d845cdadac609e92e60ef19b46a642f`

## T-001 (sprint 10)
- **Description:** Completed the ordered all-intent audit and explicit intent revisions, preserved overclaimed historical evidence through supersession, and locked build/test plans after an independent critic resolved four concerns. Book validation and git diff checks pass; source remained unchanged until Build. No TaskCreate tool is exposed in this host, so this canonical ledger tracks each step.
- **Intent:** [INT-0021](../intents/INT-0021-harness-contract-review.md)
- **Completed:** 2026-09-12T15:07:13Z
- **Files modified:** docs/intents, docs/SUMMARY.md, docs/sprints/s10 research/plans/meta, docs/work ledgers
- **Commit:** `817f5bdc6bdce1c355fc6914999b03a05f477e45`

## T-002 (sprint 10)
- **Description:** Preserved independent optional token dimensions, honest mixed-call/error coverage and checked-add overflow without inventing zeros; per-event counts remain known where reported. Focused usage/token tests passed (10 tests), targeted rustfmt and all-target/all-feature clippy with warnings denied passed.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:15:20.6988646Z
- **Files modified:** src/model.rs, src/runner.rs, tests/model_protocol.rs, tests/runner_tools.rs, tests/cli_inspect.rs
- **Commit:** `30c1c13b5e236dad18dfef1027295fc813d32db7`

## T-004 (sprint 10)
- **Description:** Introduced shared owned process groups/Windows Jobs with suspended assignment, awaited normal cleanup and synchronous drop fallback; commands now avoid pre-cancelled spawn and fit the full nested JSON envelope, including escaping and invalid UTF8. Twelve command tests passed on Windows, targeted rustfmt and clippy passed. Linux enforcement is verified under T-005; no Windows AppContainer claim.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:15:38.4038221Z
- **Files modified:** src/process.rs, src/tools.rs, src/lib.rs, Cargo.toml, src/bin/cmd-fixture.rs, tests/command_tool.rs
- **Commit:** `5c6f5aca4e49e582e84d33176adc1083f17b7475`

## T-006 (sprint 10)
- **Description:** Bounded MCP input before JSON decoding with cancellation-safe line/aggregate counters; capped servers, pages, advertised tools and frozen metadata. Replaced SDK child lifecycle with scrubbed owned process groups/jobs, suppressed inherited stderr and awaited cleanup on initialization failure, call timeout/cancellation and explicit shutdown. Five MCP unit tests and 14 integration tests passed on Windows; isolated environment worker is invoked by its parent test. Targeted rustfmt and all-target/all-feature clippy with warnings denied passed. Full run ownership moves under admission in T-007.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:28:40Z
- **Files modified:** src/mcp.rs, src/mcp/transport.rs, src/bin/mcp-fixture.rs, tests/mcp.rs
- **Commit:** `133aa3f992de035065963bc36bae1c16bec5d0fb`

## T-003 (sprint 10)
- **Description:** Made unchecked reads evidence-free and compactable while protecting checked evidence; bounded the actual framed continuation before admission. Faithful session requests exposed and repaired replay's incorrect two-source assumption; replay now validates the prior-answer source and explicitly refuses old semantic versions. Red regressions reproduced oversized admission and long-read history failure. Windows library 159, CLI inspect 10, model protocol 17, runner tools 24, replay 16 and offline live-evaluation 2 tests passed, with targeted rustfmt and clippy. Actual-session cache timing remains manual evidence pending host preparation. Includes T-008's HTTPS fixture compatibility update.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:30:05Z
- **Files modified:** src/core.rs, src/model.rs, src/policy.rs, src/runner.rs, src/replay.rs, tests/replay.rs, tests/live_evaluation.rs, tests/runner_tools.rs
- **Commit:** `1f8c9a8f1be636fc37edefaa0d51e3f15a931581`

## T-005 (sprint 10)
- **Description:** Required fully enforced Landlock ABI v3 and seccomp, narrowed runtime grants and exact executable access, protected private config/state placement, closed inherited non-stdio descriptors, denied io_uring and process-group/namespace escape while preserving normal threads/processes. Actual WSL Linux command 12 and mandatory sandbox 8 tests passed without skips, plus forced-refusal/partial-enforcement and private-root regressions. Windows command 12 and config 27 tests passed; affected Windows/Linux clippy and rustfmt passed. Includes T-008's HTTPS origin rule and address-family regression. Documentation reconciles supported content operations and residual metadata/same-user limits in T-009.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:30:55Z
- **Files modified:** src/tools.rs, src/config.rs, src/bin/cmd-fixture.rs, tests/sandbox_linux.rs
- **Commit:** `9034980397513e1b7849fa900c90eb5785093bed`

## T-010 (sprint 10)
- **Description:** Published move destinations with capability-scoped atomic no-replace hard links, followed by source cleanup. Explicit move_source_cleanup_failed reports destination publication if cleanup fails; hard-link/same-filesystem and concurrent-source limitations remain documented. Windows tools 29/29 and Linux move 4/4 tests passed, including synchronized destination creation, simultaneous competitors and cleanup failure. Rustfmt, diff checks and affected Windows/Linux clippy passed. Final security/tool documentation is reconciled in T-009.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:45:43Z
- **Files modified:** src/tools.rs
- **Commit:** `2eda32dfac8407e26bd526b7d817b3b86996cbd4`

## T-008 (sprint 10)
- **Description:** Enforced HTTPS outside loopback with separate public opt-in (shared config source committed in T-005, HTTPS fixture in T-003), documented authenticated-loopback forwarding and replaced the same-host LAN probe with actual checked two-host/replay validation. Windows config 27 and model protocol 17 tests passed; live checked nighthawk run and corrected actual-session cache observation passed on capture v3, followed by offline live-evaluation 2 tests and affected clippy/rustfmt. Measured reuse in both extension modes, with no causal flag-speedup claim. Exact deployment, hashes and observations are in remote-deployment.md.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T15:49:05Z
- **Files modified:** tests/live_evaluation.rs, docs/configuration.md, docs/integration.md, docs/sprints/s10/sprint-tests/remote-deployment.md; shared prerequisites src/config.rs and tests/runner_tools.rs recorded in T-005/T-003
- **Commit:** `e131ed8bc979df56368e64342b9a39b178f4a2d3`

## T-007 (sprint 10)
- **Description:** Replaced pre-admission MCP sessions with declarative job configuration and one active-run-owned pool. Startup is cancelled/deadline bounded, durably freezes schemas before model dispatch, and cannot enlarge live/replay grants. Cleanup precedes terminal commit and still runs after journal errors; restart recovery handles storage-unavailable rows. Capture v3 explicitly records/replays startup and cleanup. Integrated the independent 4,096-frame ceiling; a tiny-frame flood is bounded and torn down by the deadline, without claiming immediate SDK response completion. Windows lib 164, CLI inspect 10, MCP 20, replay 16 and service 16 tests passed; isolated environment worker is invoked by its parent. Targeted clippy, rustfmt and diff checks passed.
- **Intent:** [INT-0022](../intents/INT-0022-completed-contract-repairs.md)
- **Completed:** 2026-09-12T16:01:45Z
- **Files modified:** src/cli.rs, src/scheduler.rs, src/service.rs, src/runner.rs, src/replay.rs, src/policy.rs, src/mcp.rs, src/mcp/transport.rs, src/bin/mcp-fixture.rs, tests/mcp.rs, tests/service.rs
- **Commit:** PENDING
