# INT-0008 — Uniform, secure model transport (local and remote alike)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0008
- **State:** superseded
- **Work evidence:** [T-001 build plan](../sprints/s4/sprint-plans/build-plan.md#t-001-address-privacy-policy-for-model-origins)
- **Completion evidence:** [T-001–T-002 completion log](../work/completed-tasks.md)
- **Code evidence:** [T-001 `32ab6d5`, T-002 `b2b19b4`](../work/completed-tasks.md) — address-privacy policy + uniform-attach proof and runbook
- **Test evidence:** [sprint 4 test report](../sprints/s4/sprint-tests/test-report.md) — all acceptance criteria verified; live non-loopback attach executed (tested head `a8de180`)
- **Documentation evidence:** none

## Intent
Make attaching to a model server on **another physical machine** exactly as
easy as attaching to a **localhost port** — one uniform interface where a
backend is named/registered, not special-cased by location ("location
transparency"). Adding a machine should be a config line, the way nodes are
added to an OpenClaw/Hermes setup, not a new code path. Cross-machine traffic
(prompts, completions, tool arguments) must be **confidential and private** in
transit over a secure, ideally Rust/WireGuard-based encrypted fabric
(Tailscale today; a bespoke overlay like Koil, INT-0009, later) and must never
sit on a raw public address. The transport is **pluggable behind the seam** —
loopback for same-box, an overlay for cross-box — so the runtime code does not
change when the backend moves. **Mechanism is deliberately open:** this sprint
researches the most elegant scalable architecture first and is not committed to
any option discussed so far. Non-goals: building the overlay protocol itself
(INT-0009); load-balancing/routing policy across many backends beyond a minimal
registry; exposing Kinesin's own API publicly (separate gated ingress).

## Acceptance criteria
- A run attaches to a local and a remote `llama-server` through the **same
  configuration shape and code path**; only an address/registration differs.
- Registering an additional backend machine requires **no code change** — a
  config entry, plus whatever the chosen transport needs to be reachable.
- Cross-machine model traffic is **encrypted and private** (carried by the
  overlay); a backend is never reachable on a public, non-overlay address, and
  the check is explicit.
- The immutable-run / pure-replay contract holds identically to the local case
  (same prepared-request fingerprints; the endpoint/transport is not part of the
  model decision).
- A research report surveys how the strongest projects serve models between
  machines and internally, and recommends the architecture (and whether a
  tunnel-for-localhost is worth it or the seam alone suffices).
- Tests cover a local attach, a remote attach, rejection of a disallowed public
  endpoint, and an unreachable backend; live cross-machine confirmation is
  recorded the way INT-0004's live benchmark was.

## Rationale
Uniform addressing/transport is the actual scalability lever — it is what lets
compute live wherever the GPU is while orchestration stays put, and it revives
the original Kineserve/Koil vision of "the channel" as a first-class concern.
The elegance is in the seam (one interface, pluggable transport), not in
tunneling everything.

## Alternatives
Local-only serving (current). Per-session ad hoc remoting (SSH port-forward):
works but is manual and per-connection, not a standing fabric. Mandatory tunnel
even for localhost (the thesis to test — likely rejected for pure latency cost
with no benefit on-box). Build-vs-adopt for the encrypted fabric itself is
tracked by INT-0009 (Koil) with Tailscale as the adopt-now baseline; local
process supervision is INT-0007.

## Consequences
Adds a backend registry and a pluggable-transport seam; a policy distinguishing
overlay/loopback from public addresses; latency and partial-failure semantics
absent from the local path; an operational dependency on an overlay being up
for the cross-machine case. Research may recommend folding in INT-0007
(supervision) or promoting INT-0009 (Koil) as the transport.

## Transition history
- 2026-09-11: created as `proposed`.
- 2026-09-11: scope refined to a uniform local/remote secure transport seam
  (location transparency; mechanism left open) and selected as the Sprint 4
  objective, research-first.
- 2026-09-11: `proposed → planned`; linked to the sprint 4 build plan (T-001
  address-privacy policy, T-002 uniform-attach proof + runbook). Approach:
  broaden the origin policy to accept loopback + private/overlay addresses over
  HTTP, reject public by default (opt-in public requires HTTPS), keep the
  encrypted fabric out-of-band (adopt Tailscale now; Koil deferred to INT-0009).
- 2026-09-11: `planned → active`; sprint 4 build began (T-001).
- 2026-09-11: `active → realized`; sprint 4 delivered the uniform seam — a broadened origin policy that admits loopback and private/overlay (`RFC1918`, CGNAT `100.64.0.0/10`, IPv6 ULA `fc00::/7`) addresses over HTTP while rejecting public plaintext (and public HTTPS unless `allow_public_endpoints` is set). A run over an overlay `base_url` prepares byte-identical requests to a loopback run, and the live attach was **executed**: the same pinned server answered identically over `127.0.0.1:8080` and the host's RFC1918 LAN address `192.168.86.20:8080`. Encrypted transport stays out-of-band (Tailscale runbook in docs/integration.md); building Koil (INT-0009) and managed local supervision (INT-0007) remain deferred, and multi-backend routing/MagicDNS-hostname support are future work, not gaps.
- 2026-09-12: `realized → superseded` by [INT-0027](INT-0027-encrypted-remote-deployment.md). Private-address plaintext and a same-host LAN comparison do not prove cross-machine confidentiality; INT-0022 repairs client policy and INT-0027 preserves deployment acceptance. Historical criteria and realization notes are retained as provenance, not current proof.
