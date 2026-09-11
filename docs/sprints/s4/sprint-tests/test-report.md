# Sprint 4 Test Report — Uniform, secure model transport (INT-0008)

- **Tested head:** `a8de1808ee9d74e62b6563c308c9568469441edd`
- **Toolchain:** pinned 1.96.0 (rust-toolchain.toml)
- **Commands:** `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked`
- **Suite:** all binaries green — **0 failed** (145 lib incl. 6 new config tests; runner_tools 23 incl. 2 new; live_evaluation 5 ignored incl. the new attach test); fmt + clippy clean.
- **Critic verdict:** `proceed-with-caveats` (see [critique.md](critique.md)).

## Intent acceptance → evidence
| INT-0008 acceptance criterion | Verification | Layer |
|-------------------------------|--------------|-------|
| Local and remote through the same config shape / code path | `uniform_attach_prepares_identically_across_local_and_overlay_backends` (byte-identical prepared requests + identical acceptance) | integration (CI) |
| Adding a backend needs no code change | `origin_accepts_private_and_overlay_http` (overlay origin admissible) + the origin-keyed registry | unit (CI) |
| Encrypted/private; never a public non-overlay address | `origin_rejects_public_without_optin` | unit (CI) |
| Explicit opt-in exception, HTTPS only | `origin_accepts_public_https_with_optin`, `origin_rejects_public_http_with_optin` | unit (CI) |
| Secure by default | `allow_public_defaults_false` | unit (CI) |
| Replay contract holds identically | prepared bytes identical modulo origin (`uniform_attach…`) | integration (CI) |
| Unreachable backend → defined outcome | `unreachable_backend_reports_not_ready` | integration (CI) |
| Attach over a real non-loopback address (measured/observed) | `attach_to_non_loopback_backend` **run** 2026-09-11: identical answer over `127.0.0.1:8080` and LAN `192.168.86.20:8080` | e2e (live, executed) |

## Named tests (present at the tested head)
**Unit (`cargo test --lib config::`)** — `origin_accepts_loopback_http`, `origin_accepts_private_and_overlay_http`, `origin_rejects_public_without_optin`, `origin_accepts_public_https_with_optin`, `origin_rejects_public_http_with_optin`, `allow_public_defaults_false` — all ok.

**Integration (`runner_tools`)** — `uniform_attach_prepares_identically_across_local_and_overlay_backends`, `unreachable_backend_reports_not_ready` — all ok.

**E2E (live, `#[ignore]`d, `live_evaluation`)** — `attach_to_non_loopback_backend` — **run** against the pinned b6500 server bound to `0.0.0.0`: identical answer (`"ready"`) via loopback and the host LAN IP `192.168.86.20:8080`. Not a CI gate; executed manually this session.

## Caveats carried forward (from the critic)
- **C-001 (accepted, layered):** the uniform-attach test uses a scripted client, so it proves address-independence of request preparation and overlay admissibility, not real-server behavior — the latter is the `#[ignore]`d live attach.
- **C-002 (deferred):** the unreachable-backend test binds-then-drops a port; the race window is negligible on loopback and bounded by a 5 s timeout.
- **C-003 (deferred):** the live attach test needs an external server and a non-loopback interface; it is manual/ignored, the CI code path is covered offline.

## Verdict
Test phase satisfied for INT-0008: every acceptance criterion maps to a named, present test; the CI-verifiable set is green on the pinned toolchain; the one inherently-live criterion is a named, compiling `#[ignore]`d test. Proceed to loop.
