# INT-0005 — MCP tool-server integration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** planned
- **Work evidence:** [sprint 9 build plan](../sprints/s9/sprint-plans/build-plan.md#t-001-generalize-the-tool-allow-list-identity-to-toolref)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

> **Roadmap:** theme D (SotA capability) — see [the roadmap](../roadmap.md) (INT-0011). Remote transport + delegated authorization split to [INT-0020](INT-0020-remote-mcp-delegated-auth.md).

## Intent
Reach an **operator-declared, local (stdio) tool server** through the Model
Context Protocol, adapting its discovered tools into the same authority,
allow-list, and resource gates the compiled tools use. The operator approves each
server identity in config — a trusted binary, like the model endpoint — and
allow-lists which of its tools a run may call; discovery and a server's own schema
never establish authority. A server's returned text **and its tool descriptions**
are untrusted data, exactly as file contents are; an MCP tool call mints no
evidence, like `run_command`. Deterministic replay stays pure: the discovered
tool set is frozen into the run config at capture and replay re-validates the
recorded call and observation against that frozen set without reconnecting.
Non-goals: trusting a server's `read-only` annotation; forwarding an incoming
credential onward; **remote (HTTP) transport and OAuth/delegated authorization,
which are [INT-0020](INT-0020-remote-mcp-delegated-auth.md)**.

## Acceptance criteria
- An operator-declared MCP server's allow-listed tools run under the same per-run
  allow-list, capability bounds, byte limits, timeout, and concurrency permit as a
  compiled tool, and mint no evidence.
- Both gates hold: a server not declared in operator config is denied, and a tool
  not in the run's allow-list is denied — regardless of what the server advertises.
- Server-supplied text (tool descriptions and results) cannot grant a tool or
  become installed policy; it is recorded as an untrusted observation.
- Replay of an MCP run reproduces from the frozen discovered-schema set and the
  recorded observation, without reconnecting to the server.
- Tests cover an approved call (bounded), a denied unapproved server / un-allow-
  listed tool, untrusted server text treated as data, and replay reproduction,
  against an in-repo fixture server (no external network).

## Rationale
MCP is the common tool-interface standard; adopting it unlocks a large existing
tool ecosystem without hand-writing each integration. The stdio, operator-approved
core is the smallest change that delivers the capability while preserving
capability scoping, deterministic replay, and untrusted-content-as-data.

## Alternatives
Keep compiled tools only (current; smallest surface). Wait until a demonstrated
task needs an external server (the documented gate). A bespoke JSON-RPC client
instead of the official `rmcp` SDK (rejected — re-implements the spec and adds
unaudited surface).

## Consequences
Adds a protocol and a stdio transport (the official `rmcp` client) and a
dependency; generalizes the closed `ToolName` enum into a tool reference across
config/model/runner/replay/policy; a new trust boundary on operator-declared
server identity and on untrusted server output/descriptions; freeze-at-capture of
the discovered tool set for replay. The MCP **server process** is operator-trusted
and is not force-confined by the INT-0012 sandbox (which exists to confine argv the
*model* proposes via `run_command`); confining a not-fully-trusted server is a
named follow-up.

## Transition history
- 2026-09-08: created as `proposed`.
- 2026-09-11: `proposed → planned`; selected for sprint 9 and linked to the build plan (T-001 `ToolRef` generalization, T-002 operator-approved `[[mcp.servers]]` + validation, T-003 `rmcp` stdio discovery frozen into the run authority, T-004 discovered-schema emission identical live/replay, T-005 live dispatch through the existing gate, T-006 replay-without-reconnect, T-007 in-repo fixture server + tests).
- 2026-09-11: scope refined (in the sprint 9 research phase, before the state change above) — narrowed to the **local stdio, operator-approved** MVP grounded on the official `rmcp` SDK (v3.3.0). The remote (HTTP) transport, OAuth resource-server auth, RFC 8707 resource indicators, and no-token-passthrough / confused-deputy defense were **split to [INT-0020](INT-0020-remote-mcp-delegated-auth.md)** (created this phase), which carries the former "remote MCP separate authorization context / credential not forwarded" acceptance criterion. Acceptance criteria rewritten to the stdio core (two-gate approval, untrusted description+output, replay-from-frozen-schema, in-repo fixture tests); the INT-0012 sandbox relationship clarified (MCP servers are operator-trusted). Rationale/Alternatives/Consequences updated.
