# INT-0027 — Verified encrypted remote-model deployment

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0027
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** [sprint 10 scoped two-host observation](../sprints/s10/sprint-tests/remote-deployment.md); broader deployment criteria remain unverified
- **Documentation evidence:** none

## Intent
Establish and demonstrate confidential model transport between two physical machines using the same model configuration and request path as local attachment. This preserves the unproven deployment outcome of INT-0008 rather than counting a same-host LAN request as cross-machine proof. Non-goals: building Koil or treating an RFC1918/CGNAT/ULA address as cryptographic evidence.

## Acceptance criteria
- Non-loopback URLs use authenticated TLS; any loopback tunnel has separately verified encrypted endpoint and route ownership. Public-destination opt-in is distinct from confidentiality and never bypasses it.
- A two-host run records client/server identity, TLS/overlay configuration, intended binding/firewall and rejection from unintended paths, without capturing keys or private prompts.
- The same prepared decision bytes, immutable record and offline replay work locally and across that verified transport; unreachable/invalid-identity failures have defined outcomes.
- Documentation distinguishes enforceable client URL checks from server exposure and overlay properties that require deployment evidence.

## Rationale
INT-0008's recorded realization admits private plaintext and demonstrates one host. Neither proves its original cross-machine confidentiality requirement.

## Alternatives
Infer encryption from address ranges (rejected); build a new VPN in the harness (INT-0009 owns that separate decision).

## Consequences
Sprint 10 provisioned a second host and recorded a checked run, pure replay, loopback binding, authenticated SSH forwarding and rejected direct access. INT-0022 repairs client policy. This chapter stays proposed pending the broader deployment contract, including invalid-identity and route-ownership failure cases across supported configurations.

## Transition history
- 2026-09-12: created as `proposed` following the sprint 10 intent-first review and implementation audit.
