# INT-0008 — Remote model endpoints over a private overlay

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0008
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Let a run attach to a `llama-server` on another physical machine — a port at
another host's address — so model compute can live off the operator's box. The
security invariant is that such an endpoint is reachable only over a **private
overlay network** (a Tailscale/WireGuard tailnet, or the Koil overlay of
INT-0009), never a raw public IP or the open internet. Model traffic (prompts,
completions, tool arguments) is sensitive and must not cross an unencrypted or
publicly routable path. This is the "serve from another machine" half of the
model-serving goal; INT-0007 owns the local half. Non-goal: building the overlay
protocol itself (INT-0009); load-balancing across many remote backends;
exposing Kinesin's own service publicly (that is the separate, gated ingress).

## Acceptance criteria
- A model profile may name a remote endpoint; a preflight/readiness check
  confirms it responds before runs are admitted, with a defined failure when it
  does not.
- Configuration rejects (or loudly warns on) a model endpoint that is a public,
  non-overlay address; the accepted remote shape is an overlay/loopback address,
  and the check is explicit, not incidental.
- Model traffic to a remote endpoint is confidential in transit (carried by the
  overlay), and the trace/replay contract holds identically to the local case
  (same prepared-request fingerprints; the endpoint is not part of the model
  decision).
- Tests cover a reachable remote profile, an unreachable one, and rejection of a
  disallowed public endpoint. Live confirmation over a real tailnet is recorded
  the way INT-0004's live benchmark was.

## Rationale
The operator's workstation is not always where the GPU is. Reaching a remote
`llama-server` over a private overlay decouples orchestration from inference
while keeping prompts off the public internet — the practical shape of a
"local-first, machine-flexible" runtime.

## Alternatives
Local-only serving (current; INT-0007). Expose `llama-server` on a public port
with TLS + auth (rejected: larger attack surface, and it puts inference behind
the same gate as the whole box). SSH port-forward per session (works, but
per-connection and operator-driven, not a standing fabric). The overlay choice
itself is Tailscale (fastest to adopt) versus the bespoke Koil of INT-0009.

## Consequences
Adds remote-endpoint configuration and a preflight/readiness path; a policy that
must distinguish overlay/loopback from public addresses; latency and partial-
failure semantics the local path does not have; a dependency on an overlay being
up (Tailscale or Koil) as an operational precondition.

## Transition history
- 2026-09-11: created as `proposed`.
