# Sprint 4 Research Report

## Intents Reviewed
- [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) — revised; relevance: the sprint objective; current state: `proposed`. Scope was refined this session from "remote endpoint over an overlay" to a **uniform local/remote transport seam** (location transparency, pluggable transport). Related, not selected: [INT-0007](../../../intents/INT-0007-managed-model-process.md) (local supervision) and [INT-0009](../../../intents/INT-0009-koil-overlay-transport.md) (build Koil).

## 1. Sprint Goal
Make attaching to a `llama-server` on another machine exactly as easy as
attaching to a localhost port — one uniform, config-driven interface where a
backend differs only by address — with cross-machine traffic kept confidential
over a secure fabric, and never exposed on a public address. The sprint is
research-first and not committed to any specific overlay: the question is the
*most elegant scalable architecture* for serving models locally and between
machines, and whether a tunnel must wrap localhost or the uniform seam alone
carries the scalability.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| src/config.rs:148 `ModelConfig` | high | The backend profile: `base_url`, `model_id`, `context_size`, `verified_slots`, timeouts. **The uniform seam already lives here** — local and remote are the same shape, differing only by `base_url`. |
| src/config.rs:801 `validate_origin` | high | **The real gap.** Requires HTTPS for anything that is not loopback (`http` allowed only for loopback). A plain-HTTP Tailscale/overlay address (`http://100.x…`) is **rejected today**, even though it is already encrypted at L3. Also strips credentials/path/query — good. |
| src/model.rs:570 `ModelClient::http`, :536 `ready` | high | HTTP adapter + readiness probe; already address-agnostic — a remote `base_url` needs no adapter change, only the origin policy above. |
| src/runner.rs:86 `RunResources::from_config`, :138 backends keyed by `base_url` | high | Backends are already keyed/deduplicated by origin; the multi-backend registry is latent here. |
| src/dispatch.rs | medium | Fair assignment of real model capacity (`verified_slots`); where multi-backend *scale* (N endpoints) would land later. |
| src/scheduler.rs | medium | Bounded admission; unchanged by this sprint. |
| src/operator.rs:191 `readiness_monitor` | medium | Already pings each backend's readiness per profile — remote attaches the same way local does. |
| src/replay.rs:217 frozen `origin` | medium | Endpoint is carried in the frozen config but is **not** part of the request fingerprint — location transparency for replay already holds; keep it so. |
| docs/integration.md | medium | Existing model-API / remote-inference notes to reconcile. |
| docs/security.md, docs/model-preflight.md | low | Authority/isolation posture; the manual local launch (INT-0007 context). |

## 3. External Sources
- [llama.cpp RPC backend (`tools/rpc`)](https://github.com/ggml-org/llama.cpp/blob/master/tools/rpc/README.md) — distributes **one model across machines** at the ggml-op level over TCP; explicitly **"not secure by default… never expose to the open internet,"** and does not make tokens faster. Lesson: this shards a single model (a backend concern), it is *not* how Kinesin should reach whole endpoints — and its insecurity is exactly why a private fabric matters.
- [vLLM — Distributed Inference & Parallelism](https://docs.vllm.ai/en/stable/serving/parallelism_scaling/) — the scale reference: a stable **OpenAI-compatible HTTP frontend** with tensor/pipeline parallelism and Ray behind it. Lesson: the frontend is the seam; intra-model distribution is a backend detail Kinesin should treat as opaque.
- [How Tailscale works (WireGuard mesh)](https://tailscale.com/blog/how-tailscale-works) — **control-plane / data-plane split**: coordination server does identity, key exchange, peer discovery, ACLs, MagicDNS; data is **P2P, end-to-end-encrypted WireGuard** with DERP relays for NAT. Lesson: "add a machine" = sign in → stable `100.x` name; the *hard* part is the control plane, not the WireGuard data plane.
- [LiteLLM AI Gateway](https://docs.litellm.ai/docs/simple_proxy) — one OpenAI-compatible endpoint over 100+ backends via a **config-driven registry** with routing/fallback/load-balancing; "swap models without changing app code." Lesson: location transparency is achievable in pure software — the seam, not a tunnel, is what makes backends interchangeable.
- [Ollama FAQ (OLLAMA_HOST, scheduling)](https://docs.ollama.com/faq) — point a client at any host (local or remote) through one variable; a **memory-aware scheduler** that queues requests and **unloads idle models** to make room. Lesson: location-transparent UX + an idle/queue-aware scheduler (also prefigures INT-0010's idle preemption).

## 4. Risks, Unknowns, Dependencies
- **Risk (central):** `validate_origin` conflates "not loopback" with "must be HTTPS," so it blocks the cheapest secure path — plain HTTP over an already-encrypted overlay. Broadening it must not open a plaintext-to-public hole.
- **Risk:** defining "private/overlay" precisely — loopback, RFC1918, IPv6 ULA (`fc00::/7`), and Tailscale's CGNAT `100.64.0.0/10`. The CGNAT range is shared with real carrier NAT, so treat it as "private-enough for HTTP" only deliberately and document it.
- **Risk:** partial failure / latency semantics a remote path has and localhost does not; keep them as defined outcomes, not new hangs.
- **Unknown:** whether the operator runs Tailscale or wants Koil now. Adopt-Tailscale-first keeps the sprint testable immediately and defers the Koil build behind proven need (INT-0009).
- **Unknown:** multi-backend routing policy (load-balance/failover across N endpoints). Recommend **out of scope** for this sprint; the sprint delivers the registry + address policy + per-backend readiness, and routing is a follow-on.
- **Dependency:** live cross-machine evidence needs a second host on a tailnet; if unavailable, a second loopback backend proves the uniform code path and the tailnet leg is documented and recorded the way INT-0004's live benchmark was.
- **Dependency:** adjacent intents INT-0007 (supervision) and INT-0009 (Koil) — research recommends **not** bundling them into this sprint.

## 5. Recommended Approach
**Primary — a uniform opaque-backend seam with an address-privacy policy; the encrypted fabric stays out-of-band.** Concretely for this sprint:
1. Treat model config as a **named registry of opaque OpenAI-compatible backends** (it nearly is) so local and remote share one shape; only the address differs.
2. **Replace the HTTPS-vs-loopback rule in `validate_origin` with an explicit address-privacy policy:** allow HTTP to loopback **and** recognized private/overlay ranges (RFC1918, IPv6 ULA, Tailscale CGNAT `100.64/10`), require HTTPS for public hosts, and — per INT-0008 — **reject (or loudly warn on) a public endpoint by default.** This one change is the crux; the rest of the seam already exists.
3. Keep per-backend preflight/readiness (already present) so a remote attaches exactly like local.
4. Ship an **"add a machine" runbook** over Tailscale (the adopt-now fabric), documenting the `100.x`/MagicDNS address as the `base_url`.
5. Evidence: unit tests for the address policy (loopback + private + overlay accepted; public rejected/warned); an integration test attaching to two local backends through the uniform path; a live cross-machine attach recorded like INT-0004 (or a documented loopback stand-in if no second host).

**Alternative considered — build Koil now (INT-0009):** rejected for this sprint. The WireGuard *data plane* is a solved library (GotaTun/boringtun), but the *control plane* (identity, key exchange, peer discovery, NAT traversal/DERP) is the real work and is exactly what Tailscale already provides; building it is a separate product and belongs in its own repository, deferred behind a proven need. **Embedding a tunnel in the runtime:** rejected — it couples the runtime to networking/crypto, adds latency with zero benefit on localhost, and contradicts the "mechanisms you can explain" minimality. **A LiteLLM-style gateway process:** interesting for multi-backend *routing* later, heavier than needed for the seam now.

**Rationale.** This validates the user's instinct that a **uniform seam is what makes the system scale** (location transparency), while correcting "must tunnel localhost": the seam is the scalability, the tunnel is a pluggable, out-of-band transport. It honors the Koil ambition with correct scoping (own repo, later), and it lands a **small, testable change** — the address-privacy policy — as the sprint's buildable core, with security enforced in the network layer where it belongs.

## Artifacts
- No code artifacts were needed; the code survey (Section 2) is inline against the current tree, and the architectural evidence is the five external sources (Section 3). The decisive local finding is `src/config.rs:801` `validate_origin`.
