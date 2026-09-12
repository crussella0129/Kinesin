# Sprint 9 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) — state: planned; acceptance criteria covered: all four (operator-declared server's allow-listed tools run under the same per-run gates and mint no evidence; two-gate approval — undeclared server denied, un-allow-listed tool denied; server text is untrusted and cannot grant a tool/policy; replay reproduces from the frozen schema set without reconnecting; fixture-based tests for approved/denied/untrusted/replay).

## Schema Tree
- Sprint Goal: reach an operator-approved local (stdio) MCP tool server and expose its discovered tools through the existing authority / allow-list / capability / replay machinery.
  - Tool identity & config
    - T-001: generalize the allow-list identity to `ToolRef` (compiled ∪ MCP)
    - T-002: operator-approved `[[mcp.servers]]` declarations + config validation
  - Discovery, freeze, model seam
    - T-003: `rmcp` stdio client + discovery, frozen into the run authority
    - T-004: emit discovered MCP schemas to the model, identical on live and replay
  - Dispatch
    - T-005: live MCP dispatch through the existing gate
    - T-006: replay MCP dispatch (no reconnect)
  - Verification infrastructure
    - T-007: in-repo fixture MCP server + tests

## Execution Sequence

### T-001: Generalize the tool allow-list identity to `ToolRef`
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** src/config.rs, src/policy.rs (+ compiled-tool call sites in runner.rs/replay.rs/tools.rs)
- **Depends on:** (none)
- **Acceptance criterion:** "An operator-declared MCP server's allow-listed tools run under the same per-run allow-list … as a compiled tool" — requires one allow-list type spanning compiled and MCP tools; and "a tool not in the run's allow-list is denied."
- **Success criterion (EARS):**
  - **WHEN** a `tools` entry is a compiled name (e.g. `read_file`), **THEN** it **SHALL** deserialize to `ToolRef::Compiled` and behave identically to today.
  - **WHEN** a `tools` entry is `mcp__<server>__<tool>`, **THEN** it **SHALL** deserialize to `ToolRef::Mcp { server, tool }`.
  - **WHEN** a `ToolRef::Mcp` is present in a checked run's allow-list, **THEN** admission **SHALL** reject it via the existing `is_mutating()` checked-run bar.
- **Notes:** `ToolRef` keeps `Clone/Debug/PartialEq/Eq/Hash` and custom string serde; `wire_name()` returns `read_file` or `mcp__server__tool`; `is_mutating()` = true and `mints_evidence()` = false for MCP (conservative). Reuse the existing `ToolName` for the compiled arm unchanged. Existing configs (arrays of compiled names) parse without change.

### T-002: Operator-approved server declarations + config validation
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** src/config.rs
- **Depends on:** T-001
- **Acceptance criterion:** "a server not declared in operator config is denied … regardless of what the server advertises" (the config-time half of the two-gate approval).
- **Success criterion (EARS):**
  - **WHEN** a config allow-lists `mcp__S__T` but declares no server named `S`, **THEN** config validation **SHALL** fail with a defined error.
  - **WHEN** two `[[mcp.servers]]` share a `name`, or a server's `command` is empty, **THEN** validation **SHALL** fail.
- **Notes:** add `#[serde(default)] mcp: Option<McpConfig>` to both `Config` and `ConfigFile` (config.rs:26/52); `McpConfig { servers: Vec<McpServer> }`, `McpServer { name, command: Vec<String> }`. Validate in the same pass as `unique_tools`/workspace checks. `deny_unknown_fields` is preserved (new optional section).

### T-003: `rmcp` stdio client + discovery, frozen into the run authority
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** Cargo.toml, src/mcp.rs (new), src/policy.rs, src/replay.rs, src/runner.rs
- **Depends on:** T-002
- **Acceptance criterion:** "Replay of an MCP run reproduces from the frozen discovered-schema set … without reconnecting" (freeze half) and the discovery needed for "allow-listed tools run under the same gates."
- **Success criterion (EARS):**
  - **WHEN** a run allow-lists an MCP tool, **THEN** the runner **SHALL** connect to its declared server, discover the tool's `{description, input_schema}`, and freeze it into the run authority before admission.
  - **WHEN** an allow-listed MCP tool is absent from the server's advertised `tools/list`, **THEN** the run **SHALL** fail to start with a defined error.
  - **WHEN** discovery (connect + initialize + list) exceeds a bound or the transport errors, **THEN** the run **SHALL** fail to start with the same defined error rather than hang (C-003).
  - **WHEN** a run has no MCP tools, **THEN** the serialized authority **SHALL** be byte-identical to today (`mcp_tools` omitted) and pre-feature journals **SHALL** still replay.
- **Notes:** `rmcp = { version = "3", features = ["client","transport-child-process"] }` (all-OS normal dep; vetted by the existing `supply-chain` job). `src/mcp.rs`: `McpClientPool` (one `TokioChildProcess` child per used server, `().serve(...)` → initialize), `discover(&[ToolRef]) -> Vec<McpToolDef>` filtered to allow-listed tools, `McpToolDef { server, tool, description, input_schema: serde_json::Value }`. Add `#[serde(default, skip_serializing_if = "Vec::is_empty")] mcp_tools: Vec<McpToolDef>` to **both** `RunAuthority` (policy.rs:166) and `FrozenContext` (replay.rs:172); `skip_serializing_if` is load-bearing for replay parity (the `json!(frozen) == accepted.data["replay"]["authority"]` equality at replay.rs:465). Runner discovers before `admit_once` (runner.rs:248) and holds the pool for the run loop.

### T-004: Emit discovered MCP schemas to the model, identical on live and replay
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** src/model.rs, src/runner.rs, src/replay.rs
- **Depends on:** T-003
- **Acceptance criterion:** "An operator-declared MCP server's allow-listed tools run under the same per-run gates" (the model must be offered the tool) and "Replay … reproduces … without reconnecting" (identical request bytes).
- **Success criterion (EARS):**
  - **WHEN** a frozen authority carries an MCP tool, **THEN** the model request `tools` array **SHALL** include a function named `mcp__server__tool` whose `parameters` are the discovered `input_schema`.
  - **WHEN** that run is replayed, **THEN** the emitted request bytes **SHALL** be identical (prepared-request fingerprint matches).
- **Notes:** change `ModelOptions.tools: Vec<String>` (model.rs:61) → `Vec<ToolDef>` = `Compiled { name: String }` | `Mcp { name, description, input_schema }`; the compiler (model.rs:198-258) matches on the variant — compiled emits the existing hardcoded schema, MCP emits `{name, description, parameters: input_schema}`. Both `runner::options` (runner.rs:218) and `FrozenContext::options` (replay.rs:213) build the same `Vec<ToolDef>` from `workspace.tools` + `mcp_tools`. Update model.rs unit tests that push `"read_file".into()`.

### T-005: Live MCP dispatch through the existing gate
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** src/runner.rs, src/tools.rs (result mapping helper), src/mcp.rs
- **Depends on:** T-004
- **Acceptance criterion:** "allow-listed tools run under the same per-run allow-list, bounds, and untrusted-output handling … and mint no evidence"; "a tool not in the run's allow-list is denied"; "server-supplied text … recorded as an untrusted observation."
- **Success criterion (EARS):**
  - **WHEN** the model calls an allow-listed MCP tool with valid args, **THEN** the runner **SHALL** invoke it and record its result as an observation that mints no evidence, within the byte/time/concurrency bounds.
  - **WHEN** the model calls an MCP tool absent from the run's allow-list, **THEN** the call **SHALL** be `Denied` without contacting any server.
  - **WHEN** an MCP result exceeds `max_tool_result_bytes`, **THEN** it **SHALL** be bounded/truncated like a compiled-tool result.
- **Notes:** extend the dispatch ladder at runner.rs:914-955. Resolve an MCP name against the frozen `mcp_tools`; denial ladder mirrors compiled: not-in-frozen-set → `unknown_tool`; not allow-listed → `tool_denied`; args fail a lightweight required-fields/object check from `input_schema` → `invalid_arguments`. Execute via `McpClientPool.call` under the existing `resources.tools` permit + `deadline` + `max_tool_result_bytes`. Map `CallToolResult` → `ToolResult` (untrusted content; `mints_evidence=false`). Journal `tool_planned`/`tool_finished` unchanged.

### T-006: Replay MCP dispatch (no reconnect)
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** src/replay.rs
- **Depends on:** T-003, T-005
- **Acceptance criterion:** "Replay of an MCP run reproduces from the frozen discovered-schema set and the recorded observation, without reconnecting to the server."
- **Success criterion (EARS):**
  - **WHEN** an MCP tool run is replayed, **THEN** replay **SHALL** re-validate the call against the frozen schema set + allow-list and reproduce the recorded observation without connecting to any server.
- **Notes:** add the MCP branch to replay.rs:633 mirroring the live denial ladder against `frozen.mcp_tools` + `frozen.workspace.tools`; read the observation from `finished.data["replay"]["observation"]` exactly as the file path does (replay.rs:629-631). No `McpClientPool` is constructed on the replay path.

### T-007: In-repo fixture MCP server + tests
- **Intent:** [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md)
- **Touches:** src/bin/mcp-fixture.rs (new), Cargo.toml (bin entry), tests/mcp.rs (new)
- **Depends on:** T-001, T-002, T-003, T-004, T-005, T-006
- **Acceptance criterion:** "Tests cover an approved call (bounded), a denied unapproved server / un-allow-listed tool, untrusted server text treated as data, and replay reproduction, against an in-repo fixture server (no external network)."
- **Success criterion (EARS):**
  - **WHEN** the fixture server is spawned and a run allow-lists `mcp__fixture__echo`, **THEN** discovery + a `tools/call` **SHALL** succeed and the observation **SHALL** match the echoed input.
  - **WHEN** a run calls a server/tool not approved, **THEN** the call **SHALL** be denied without spawning/contacting the server.
  - **WHEN** the `poison` tool's description/result carries injected instructions, **THEN** policy and authority **SHALL** be unchanged (treated as data) and it **SHALL** mint no evidence.
- **Notes:** `src/bin/mcp-fixture.rs` is a minimal stdio newline-delimited JSON-RPC server (no rmcp server feature) answering `initialize`/`tools/list`/`tools/call`, exposing `echo` and `poison`, plus a `--hang` mode that stalls before responding (for the discovery-timeout test, C-003). `tests/mcp.rs` uses the fake-model harness pattern from tests/model_protocol.rs + tests/runner_tools.rs. Reuses `cargo`'s bin-path env (`CARGO_BIN_EXE_mcp-fixture`) to locate the fixture, mirroring how tests/*.rs locate `cmd-fixture`.
