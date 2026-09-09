# INT-0005 — MCP tool-server integration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Reach external tool servers through the Model Context Protocol, adapting them
into the same authority, allow-list, and resource gates the compiled tools use.
The operator approves each server identity; discovery and a server's own schema
never establish authority; a server's returned text is untrusted data, exactly
as file contents are. Non-goal: trusting a server's `read-only` annotation, or
forwarding an incoming credential onward.

## Acceptance criteria
- An operator-approved MCP server's tools run under the same per-run allow-list,
  capability bounds, and byte limits as a compiled tool.
- Server-supplied text cannot grant a tool or become installed policy.
- Remote MCP carries a separate authorization context; a credential is not
  forwarded through the harness.
- Tests cover an approved call, a denied unapproved server, and untrusted
  server text treated as data.

## Rationale
MCP is the common tool-interface standard; adopting it unlocks a large existing
tool ecosystem without hand-writing each integration.

## Alternatives
Keep compiled tools only (current; smallest surface). Wait until a demonstrated
task needs an external server (the documented gate).

## Consequences
Adds a protocol and a transport; a new trust boundary on server identity and on
untrusted server output; lifecycle and cancellation of a remote call.

## Transition history
- 2026-09-08: created as `proposed`.
