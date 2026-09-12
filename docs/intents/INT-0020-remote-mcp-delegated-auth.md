# INT-0020 — Remote MCP transport & delegated authorization

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0020
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

> **Roadmap:** theme D (SotA capability) — see [the roadmap](../roadmap.md) (INT-0011). Split from [INT-0005](INT-0005-mcp-tool-servers.md); depends on it and on the transport-privacy boundary ([INT-0008](INT-0008-remote-model-over-overlay.md)/[INT-0009](INT-0009-koil-overlay-transport.md)).

## Intent
Reach an MCP tool server over the network (Streamable HTTP transport), extending
the local stdio integration ([INT-0005](INT-0005-mcp-tool-servers.md)) to a
remote server that is an **OAuth resource server**. The operator still approves
each remote server identity; the harness authorizes to that specific server and
**never forwards an inbound credential onward** (no token passthrough), and a
token minted for one server cannot be replayed to another. Everything INT-0005
established — two-gate approval, untrusted descriptions/output, replay from a
frozen discovered-schema set, no evidence minted — holds unchanged over the
remote transport. Non-goals: acting as an authorization server ourselves;
deprecated SSE transport; trusting a `read-only` annotation.

## Acceptance criteria
- An operator-approved **remote** MCP server's allow-listed tools run under the
  same per-run allow-list, bounds, and untrusted-output handling as a local
  (stdio) server, over the Streamable HTTP transport.
- The client implements **RFC 8707 resource indicators**: a token is bound to the
  intended server (audience-restricted), so a malicious or compromised server
  cannot obtain or replay a token issued for another (confused-deputy defense).
- **No token passthrough:** an inbound credential presented to Kinesin is not
  forwarded to an MCP server; the harness holds a separate authorization context
  per remote server.
- The remote endpoint honors the address-privacy policy already enforced for the
  model endpoint (loopback/overlay/private over plain HTTP; public requires
  `allow_public_endpoints` + HTTPS) — consistent with [INT-0008](INT-0008-remote-model-over-overlay.md).
- Tests cover an approved remote call, a rejected token-audience mismatch, and the
  no-passthrough guarantee (an inbound credential never reaches the server).

## Rationale
Remote MCP is where the ecosystem's hosted tools live, and it is the half of
MCP with real security teeth: the 2025-06-18 spec classifies servers as OAuth
resource servers and mandates resource indicators precisely to stop token theft
and confused-deputy attacks. That is a delegated-authorization problem large
enough to dwarf the stdio core, so it is tracked separately rather than rushed
into INT-0005's realization.

## Alternatives
Local stdio only (INT-0005; smallest surface, no network). Forwarding the
operator's own token to servers (rejected — the token-passthrough anti-pattern
the spec and the NSA MCP guidance name explicitly). A bespoke bearer scheme
(rejected — reinvents OAuth resource-server semantics the spec standardizes).

## Consequences
Adds an HTTP MCP client transport (`rmcp` `transport-streamable-http-client-*`)
and an OAuth client/resource-indicator flow; a credential-handling surface that
must never leak across servers; interaction with the existing address-privacy
policy and the encrypted-fabric story (Tailscale/Koil). More moving parts and a
larger attack surface than the stdio core, which is why it is gated behind it.

## Transition history
- 2026-09-11: created as `proposed` (sprint 9 research phase; split from INT-0005, theme D — SotA capability). Carries the remote-transport + delegated-authorization scope (OAuth resource server, RFC 8707 resource indicators, no token passthrough / confused-deputy defense) moved out of INT-0005 so its stdio MVP can be realized in one sprint.
