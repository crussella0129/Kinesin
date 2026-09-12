# Sprint 9 Unit Tests

Executed by `cargo test --lib` (Windows). All pass. These prove the pure,
server-free contracts: the tool-reference identity, config validation, the
freeze/back-compat property, and MCP argument/result helpers.

## T-001 — `ToolRef` identity (`src/config.rs::tests`)
- `toolref_compiled_roundtrip` — each compiled wire name parses to `ToolRef::Compiled` and `wire_name()` round-trips; an unknown bare name is rejected. **WHEN** a compiled name is in the allow-list **THEN** it behaves as before.
- `toolref_mcp_parse` — `mcp__files__grep` → `ToolRef::Mcp { server, tool }`; `mcp()` and `wire_name()` round-trip; `is_mutating()==true`, `mints_evidence()==false`; malformed `mcp__` names are rejected. **WHEN** an entry is `mcp__<server>__<tool>` **THEN** it parses to the MCP reference.
- `toolref_mcp_barred_from_checked_run` — an allow-list containing a `ToolRef::Mcp` trips the `is_mutating()` checked-run bar; a read-only compiled set does not. **WHEN** a `ToolRef::Mcp` is in a checked run's allow-list **THEN** admission rejects it.

## T-002 — operator server declarations (`src/config.rs::tests`)
- `mcp_config_accepts_declared_server_and_tool` — a declared `[[mcp.servers]]` plus an `mcp__docs__grep` allow-list entry parses and is retained.
- `mcp_config_rejects_undeclared_server` — `mcp__ghost__x` with no declared `ghost` server fails config validation. **WHEN** an allow-list names an undeclared server **THEN** validation fails.
- `mcp_config_rejects_dup_or_empty` — duplicate server id, empty command, and a server id containing `__` each fail. **WHEN** two servers share a name / a command is empty **THEN** validation fails.

## T-003 — freeze / back-compat (`src/policy.rs::tests`)
- `non_mcp_authority_omits_mcp_tools_while_added_ones_freeze` — a non-MCP authority serializes with no `mcp_tools` key (byte-identical to the pre-feature shape, preserving replay parity); `with_mcp_tools` attaches the frozen set, which then serializes. **WHEN** a run has no MCP tools **THEN** the frozen authority bytes are unchanged.

## T-005 — MCP dispatch helpers (`src/mcp.rs::tests`)
- `validate_args_requires_object_and_required_keys` — a JSON object with the schema's required keys passes and returns the argument map; a missing required key, a non-object, and malformed JSON are refused (before any server is contacted). **WHEN** MCP arguments do not match the discovered schema shape **THEN** the call is refused with `invalid_arguments`.
- `bound_truncates_on_a_char_boundary` — text within the cap is kept whole; a multi-byte char straddling the cap is dropped, never split, and `truncated` is reported. **WHEN** an MCP result exceeds the byte cap **THEN** it is bounded on a UTF-8 boundary.
- `tooldef_wire_name_matches_toolref` — `McpToolDef::wire_name()` equals the `ToolRef::Mcp` namespacing.

## Result
`cargo test --lib` → **155 passed / 0 failed** (includes the pre-existing suite plus the 10 tests above).
