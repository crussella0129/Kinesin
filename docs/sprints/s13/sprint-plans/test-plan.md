# Sprint 13 Test Plan

Live operation and repair come first. Official unit and integration
tests remain deferred until the live confidence gate in T-114 is satisfied.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
| --- | --- | --- | --- |
| [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md) | AC1 provenance and isolation | T-112 / workload begins | live_workspace_provenance |
| INT-0031 | AC2 generated app and follow-up | T-112 / receives requests | live_harness_authorship |
| INT-0031 | AC3 owned loopback server | T-113 / asked to serve | live_server_lifecycle; repaired_server_integration |
| INT-0031 | AC2/AC3 usable app and follow-up | T-113 / browser operates | live_storefront_browser |
| INT-0031 | AC4 repair preserves boundaries | T-113 / blocker reproduced | live_failure_recovery; repaired_boundary_units; repaired_server_integration |
| INT-0031 | AC3/AC4 confined preview | T-113 / ungranted or escapes | preview_denies_outside_paths; preview_capability_integration |
| INT-0031 | AC3 local CLI only | T-113 / service-mode grant | preview_service_config_rejected |
| INT-0031 | AC4 official checks after confidence | T-114 / gate passed | deferred_verification_record |

## Unit Tests
- **Intent:** [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md)
- `repaired_boundary_units`: T-113 third clause; after live confidence, exercise
  the precise affected authorization/path/process invariant. Reuse existing
  relevant coverage; add only regressions justified by a concrete repair.
- `preview_denies_outside_paths`: T-113 fourth clause; verify traversal,
  encoded traversal and symlink/outside-root rejection with no outside bytes.
  Do not introduce app unit tests that duplicate browser interactions.
- `preview_service_config_rejected`: T-113 fifth clause; otherwise valid
  service configuration with preview grant fails validation before runtime
  startup; local CLI configuration may grant the capability.

## Integration Tests
- **Intent:** [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md)
- `repaired_server_integration`: T-113 first/third clauses; after live confidence,
  focused real-process coverage of the affected launch/stop and denied-effects
  contract. Use existing suites when they already cover the changed behavior.
- `deferred_verification_record`: T-114; record the confidence-gate evidence
  before official test timestamps, and final formatter, Clippy and critic results.
- `preview_capability_integration`: T-113 fourth clause; absent grant causes no
  listener; granted preview serves only its selected subtree, binds loopback
  and closes on owned-resource shutdown. It remains usable across local
  conversational turns; session EOF/shutdown closes the listener. Include
  actual HTTP responses and verify response body bytes, not only status codes.
- Exact target filters are selected against the implemented preview modules.

## End-to-End Tests
- **Status:** possible
- **Intent:** [INT-0031](../../../intents/INT-0031-live-local-app-delivery.md)
- `live_workspace_provenance`: T-112 first clause; inspect dedicated workspace,
  private profile, model/configuration and resource limits before the first run.
- `live_harness_authorship`: T-112 second clause; retain real Kinesin tool events
  and inspect generated files plus a follow-up edit; reject prose-only claims.
- `live_server_lifecycle`: T-113 first clause; Kinesin triggers the server,
  loopback HTTP serves generated files across follow-up turns, and the listener
  stops on session shutdown. No unrelated listener may satisfy the observation.
- `live_storefront_browser`: T-113 second clause; internal browser exercises
  catalog filtering/search, add/change/remove cart items, correct totals,
  checkout confirmation and the follow-up. Capture actual visible outcomes.
- `live_failure_recovery`: T-113 third clause; retain failing run, correction
  and successful repetition of each repaired user path; identify model failures
  separately and do not hide inconclusive attempts.
- Live confidence gate: all five live scenarios above are evidenced. Then run
  `cargo fmt --all -- --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`, and
  only the official suites affected by the final implementation. Broaden only
  when failures or review findings justify it.
