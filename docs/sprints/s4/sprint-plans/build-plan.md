Finalized - DO NOT EDIT

# Sprint 4 Build Plan

## Intents
- [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) — state: planned; acceptance criteria covered: uniform local/remote code path, no-code-change backend registration, encrypted/private-never-public policy, replay contract unchanged, local/remote/reject-public/unreachable test coverage.

## Schema Tree
- Sprint Goal: uniform, secure model transport (local == remote)
  - Origin policy
    - T-001: address-privacy policy for model origins
  - Uniform attach + docs
    - T-002: uniform attach proof + "add a machine" runbook

## Execution Sequence

### T-001: Address-privacy policy for model origins
- **Intent:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- **Touches:** src/config.rs
- **Depends on:** (none)
- **Acceptance criterion:** cross-machine backends are reachable through the same config shape as local, and a backend is never reachable on a public, non-overlay address unless explicitly opted in — the check is explicit.
- **Success criterion (EARS):**
  - **WHEN** `validate_origin` receives an `http` or `https` URL whose host is loopback, an RFC1918 IPv4, the CGNAT range `100.64.0.0/10`, or an IPv6 unique-local address (`fc00::/7`), **THEN** it **SHALL** accept the origin.
  - **WHEN** the host is a public address and `allow_public_endpoints` is false, **THEN** `validate_origin` **SHALL** reject the origin with an explicit error.
  - **WHEN** the host is public and `allow_public_endpoints` is true, **THEN** `validate_origin` **SHALL** accept the origin only if the scheme is `https`, and **SHALL** reject public `http`.
  - **WHEN** the config omits `allow_public_endpoints`, **THEN** it **SHALL** default to false.
- **Notes:** rework the existing `validate_origin` (src/config.rs:801) into host classification + policy; add `allow_public_endpoints: bool` (serde default false) to the config root, thread it into the models validation loop (src/config.rs:509). Use `Ipv4Addr::is_loopback`/`is_private`; hand-implement CGNAT `100.64.0.0/10` and ULA `fc00::/7` (std helpers unstable on pinned 1.96.0). Preserve the existing credential/path/query/fragment rejection.

### T-002: Uniform attach proof + "add a machine" runbook
- **Intent:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- **Touches:** tests/ (new integration + live tests), docs/integration.md
- **Depends on:** T-001
- **Acceptance criterion:** a run attaches to a local and a remote backend through the same code path (only the address differs); the trace/replay contract holds identically; an unreachable backend produces a defined outcome.
- **Success criterion (EARS):**
  - **WHEN** a run is configured with a backend at a private/overlay address, **THEN** it **SHALL** prepare and dispatch requests through the same code path as a loopback backend, with prepared-request bytes identical modulo the origin.
  - **WHEN** a configured backend is unreachable, **THEN** readiness **SHALL** report not-ready with a defined outcome and no hang.
  - **WHEN** the live attach test runs against a `llama-server` at the host's real non-loopback address, **THEN** it **SHALL** complete a run over that address and return the loopback baseline's answer (manual/live; not a CI gate).
- **Notes:** integration test compares prepared-request bytes for two configs differing only in `base_url` (loopback vs. an accepted overlay address); readiness test points at a closed port and asserts not-ready within the configured timeout; an `#[ignore]`d live test attaches to a real non-loopback address (the host's own LAN/tailnet IP) and is recorded like INT-0004's benchmark; add a concise Tailscale "add a machine" runbook to docs/integration.md. Reuse `ModelClient::http`/`ready` (src/model.rs) unchanged.
