# INT-0025 — Credential isolation and explicit egress

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0025
- **State:** proposed
- **Work evidence:** [T-105 backlog](../work/tasks.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Keep credentials and private content within operator-declared data flows across model transport, local/remote MCP, command tools, diagnostic output, replay and telemetry. Non-goal: claiming operator-approved native binaries are sandboxed or that a hash hides low-entropy secrets.

## Acceptance criteria
- Inventory pre-opened handles/descriptors in supported embedding scenarios and test synthetic private descriptors; pathname confinement alone cannot revoke existing descriptor access. No current leak is established by the sprint 10 audit, and untrusted ambient descriptor inheritance is not a supported guarantee.
- Child processes receive a scrubbed baseline environment; any additional credential is explicit and scoped to its declared destination.
- Origin changes, redirects and tool descriptions/results cannot transfer credentials or expand destinations.
- Routine diagnostics and telemetry exclude credentials and content; replay export is explicit, private, owner-scoped and covered by retention.
- Credential rotation/revocation and malformed/untrusted server responses are tested without embedding real secrets.

## Rationale
Credential boundaries are scattered across transport, commands and future OAuth. A cross-cutting contract prevents a new adapter from inheriting ambient secrets by default.

## Alternatives
Trust every inherited environment value (rejected); invent an unrelated secret vault (not required; use operator facilities).

## Consequences
May require explicit per-server environment configuration and migration guidance. Command/MCP repair defaults are delivered by INT-0022; the full lifecycle remains proposed.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.
