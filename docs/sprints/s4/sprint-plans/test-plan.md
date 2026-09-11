Finalized - DO NOT EDIT

# Sprint 4 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | local and remote through the same config shape and code path | T-002 / WHEN a private/overlay backend is configured THEN it prepares/dispatches through the same path modulo origin | `uniform_attach_prepares_identically_across_local_and_overlay_backends` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | adding a backend needs no code change | T-001 / WHEN host is loopback/RFC1918/CGNAT/ULA THEN accept | `origin_accepts_private_and_overlay_http` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | encrypted/private; never a public non-overlay address | T-001 / WHEN host public AND opt-in false THEN reject | `origin_rejects_public_without_optin` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | explicit, opt-in exception only over https | T-001 / WHEN public AND opt-in true THEN accept https, reject http | `origin_accepts_public_https_with_optin`, `origin_rejects_public_http_with_optin` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | secure-by-default | T-001 / WHEN config omits the flag THEN default false | `allow_public_defaults_false` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | replay contract holds identically | T-002 / prepared-bytes identical modulo origin | `uniform_attach_prepares_identically_across_local_and_overlay_backends` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | unreachable backend → defined outcome | T-002 / WHEN unreachable THEN not-ready, no hang | `unreachable_backend_reports_not_ready` |
| [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) | measured/observed cross-machine attach | T-002 / live | `attach_to_non_loopback_backend` (ignored, live) |

## Unit Tests
### T-001 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- `origin_accepts_loopback_http`: `http://127.0.0.1:8080` and `http://localhost:8080` → accepted.
- `origin_accepts_private_and_overlay_http`: `http://192.168.1.10:8080` (RFC1918), `http://100.64.0.5:8080` (CGNAT), `http://[fd7a::1]:8080` (ULA) → accepted.
- `origin_rejects_public_without_optin`: `http://93.184.216.34:8080` and `https://api.example.com` → rejected when `allow_public_endpoints` is false.
- `origin_accepts_public_https_with_optin`: `https://api.example.com` → accepted when opt-in true.
- `origin_rejects_public_http_with_optin`: `http://93.184.216.34:8080` → rejected even when opt-in true (public plaintext).
- `allow_public_defaults_false`: config omitting the flag parses with it false.
- Stubs: none (pure function + config parse).

## Integration Tests
### Uniform transport integration
- **Intents:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- `uniform_attach_prepares_identically_across_local_and_overlay_backends`: two configs identical but for `base_url` (loopback vs. an accepted overlay address) produce byte-identical prepared requests; both run through the same admit/prepare path under a scripted client.
- `unreachable_backend_reports_not_ready`: a backend at a closed private/overlay port reports not-ready within the configured connect/read timeout, with no hang.

## End-to-End Tests
- **Status:** possible (live, manual — the offline integration tests are the CI stand-in, same split as INT-0004).
- `attach_to_non_loopback_backend`: with a `llama-server` reachable at the host's real LAN/tailnet address, a run attaches over that non-loopback address and returns the same answer as the loopback baseline; recorded with workload and machine. `#[ignore]`d; not a CI gate.
