Finalized - DO NOT EDIT

# Sprint 8 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) | OWASP/NIST/CISA-mapped threat model, SUMMARY-linked | T-001 / WHEN threat-model.md exists THEN maps each risk+control, linked | `threat_model_present_and_mapped` |
| [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) | memory-safety statement enumerates the unsafe/FFI surface | T-001 / WHEN read THEN every unsafe src file enumerated | `memory_safety_enumerates_unsafe` |
| [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) | red-team corpus maps to the release-evidence matrix | T-001 / WHEN corpus map read THEN each matrix row names a test | `corpus_maps_matrix` (doc coverage) |
| [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) | corpus runs in CI; forbidden effect cannot pass | T-002 / WHEN redteam runs THEN unauthorized denied + content-as-data | `redteam_denies_unauthorized`, `redteam_treats_content_as_data` |

## Unit Tests
Not a separate layer for the documentation (T-001); the mapping is verified by
structural/coverage checks below. The T-002 corpus assertions are integration
tests against the tools/policy layer.

## Integration Tests
### Red-team corpus (`tests/redteam.rs`)
- **Intents:** [INT-0015](../../../intents/INT-0015-threat-model-assurance.md)
- `redteam_denies_unauthorized`: a tool/path the run was not granted is denied by the capability/policy layer (a forbidden effect cannot pass the gate).
- `redteam_treats_content_as_data`: workspace/tool-result content containing instructions does not change policy, authority, or the executed effect.

### Assurance-doc coverage (structural)
- `threat_model_present_and_mapped`: check-book valid; `docs/threat-model.md` is SUMMARY-linked and contains the OWASP, NIST, memory-safety, and matrix-map sections.
- `memory_safety_enumerates_unsafe`: every source file containing `unsafe` (`private_state.rs`, `storage.rs`, `signal.rs`, `tools.rs`) is named in the memory-safety statement.
- `corpus_maps_matrix`: each security.md release-evidence-matrix row names an executed test in the corpus map.

## End-to-End Tests
- **Status:** possible (CI). The ubuntu + windows `check` jobs run `tests/redteam.rs`
  green; the `supply-chain` job stays green (no dependency change). The
  structural/coverage checks are run locally + by reading the committed docs (the
  same non-unit verification prior doc sprints used).
