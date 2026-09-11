# Sprint 4 Unit Tests

- **Intent:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- **Tested head:** `a8de1808ee9d74e62b6563c308c9568469441edd`
- **Runner:** `cargo test --locked --lib config::`
- **Result:** all green (21 config tests; 6 new for T-001).

## T-001 unit tests (`src/config.rs`)
| Test | EARS clause | Result |
|------|-------------|--------|
| `origin_accepts_loopback_http` | WHEN host is loopback THEN accept (http) | ok |
| `origin_accepts_private_and_overlay_http` | WHEN host is RFC1918 / CGNAT `100.64.0.0/10` / IPv6 ULA `fc00::/7` THEN accept (http) | ok |
| `origin_rejects_public_without_optin` | WHEN host is public AND `allow_public_endpoints` false THEN reject | ok |
| `origin_accepts_public_https_with_optin` | WHEN host is public AND opt-in true AND scheme https THEN accept | ok |
| `origin_rejects_public_http_with_optin` | WHEN host is public AND opt-in true AND scheme http THEN reject | ok |
| `allow_public_defaults_false` | WHEN config omits the flag THEN it defaults false (a public https origin is rejected) | ok |

The existing `model_origins_cannot_smuggle_paths_credentials_or_plaintext_remote_hosts`
was updated for the new signature and now asserts a public HTTPS origin is
admissible only with the opt-in — the deliberate policy tightening.

## Confirmation
```
test config::tests::origin_accepts_loopback_http ... ok
test config::tests::origin_accepts_private_and_overlay_http ... ok
test config::tests::origin_rejects_public_without_optin ... ok
test config::tests::origin_accepts_public_https_with_optin ... ok
test config::tests::origin_rejects_public_http_with_optin ... ok
test config::tests::allow_public_defaults_false ... ok
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 124 filtered out
```
