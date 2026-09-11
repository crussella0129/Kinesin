# INT-0009 — Koil: a pure-Rust tailscale-like overlay transport

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0009
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

> **Roadmap:** theme D (SotA capability) — see [the roadmap](../roadmap.md) (INT-0011). Behind Tailscale (INT-0008 done).

## Intent
Build **Koil**: a pure-Rust, "tailscale-like" encrypted overlay — WireGuard
data plane via a userspace implementation (GotaTun/boringtun lineage) plus a
minimal coordination/peer-discovery plane — that Kinesin dials to reach a remote
`llama-server` (INT-0008) without depending on the Tailscale daemon. This
revives the original design's "Koil = the channel" role, but as a modern
overlay rather than a bare tunnel. The transport crate almost certainly lives in
its **own repository** (`koil`), consumed here as a dependency behind a small
interface, so this chapter tracks the decision, the integration seam, and the
"build vs. adopt Tailscale" tradeoff — not necessarily in-tree code. Non-goal:
re-implementing WireGuard crypto by hand; a general-purpose VPN product; making
Koil mandatory (Tailscale and loopback remain first-class per INT-0008).

## Acceptance criteria
- A decision record chooses build-Koil vs. adopt-Tailscale for INT-0008, with
  the criteria (control, dependency footprint, NAT traversal, effort) that
  drove it.
- If built: Kinesin can reach a remote `llama-server` over a Koil link with
  confidentiality equivalent to WireGuard, exercised end-to-end between two
  hosts, and INT-0008's acceptance holds over that link.
- The overlay lives behind an interface such that swapping Koil for Tailscale (or
  plain loopback) changes no K-Core/runner code.
- Tests/evidence: a handshake + round-trip proof for the transport crate, and an
  integration proof that a run works over it (live, recorded like INT-0004).

## Rationale
A bespoke pure-Rust overlay is a stated long-term ambition and removes a
third-party daemon from the trust and deployment path, matching the project's
"mechanisms you can explain" minimality. Keeping it in its own repository stops
a large networking/crypto surface from swelling the runtime crate.

## Alternatives
Adopt Tailscale directly (recommended first step; zero protocol work, satisfies
INT-0008 immediately). Wrap system WireGuard (`wg`/`wg-quick`) as in the original
scaffold's "Path A" (tested, but shells out and needs host privilege). Hand-write
the Noise handshake (rejected: large and error-prone). This intent is
deliberately downstream of proving the need via INT-0008.

## Consequences
A substantial crypto/networking surface with its own security burden, TUN-device
privileges, and NAT-traversal problems; a cross-repository dependency and
release coordination; only worth starting once INT-0008 has demonstrated the
remote path and Tailscale's limits (if any) are concrete rather than assumed.

## Transition history
- 2026-09-11: created as `proposed`.
