# Sprint 8 Test Report — Threat model & security assurance (INT-0015)

- **Tested head:** `8ea22196b83af80007cc22e42641518fec4f7ca2`
- **Deliverable:** the assurance package (`docs/threat-model.md`) + a consolidating red-team test (`tests/redteam.rs`).
- **Result:** all green — redteam 2/2 on Windows and WSL; the doc-coverage checks pass; full suite green (prior runs) with the additive test.
- **Critic verdict:** `proceed-with-caveats` (see [critique.md](critique.md)).

## Intent acceptance → evidence
| INT-0015 acceptance criterion | Verification | Result |
|-------------------------------|--------------|--------|
| threat model maps each OWASP LLM/Agentic risk + NIST control (mechanism/gap/residual, incl. transport-auth) | `threat_model_present_and_mapped` (§1–2 tables, SUMMARY-linked) | ok |
| memory-safety statement enumerates the unsafe/FFI surface | `memory_safety_enumerates_unsafe` (4/4 files) | ok |
| red-team corpus maps to the release-evidence matrix; forbidden effect cannot pass | `corpus_maps_matrix` + `redteam_denies_unauthorized` + `redteam_treats_content_as_data` | ok |
| versioned with an owner/cadence | doc header (v1, owner, cadence) | ok |

Details: [integration-tests.md](integration-tests.md), [e2e-tests.md](e2e-tests.md).

## Caveats carried forward (from the critic)
- **C-001 (accepted):** mapping completeness is author-bounded; anchored to OWASP/NIST + the security.md matrix, and the doc is versioned for revision.
- **C-002 (deferred):** the executed corpus is two consolidating tests; the bulk of the corpus is the existing adversarial/auth/sandbox suites, mapped row-by-row.
- **C-003 (accepted):** content-as-data is asserted at the command boundary here; the model-facing prompt-injection property is proven by the cited `live_evaluation` hostile-text cards.

## Verdict
Test phase satisfied for INT-0015: the security posture is now a single
standards-mapped, versioned assurance package with a memory-safety statement and
a corpus tied to the release-evidence matrix; the two cross-cutting invariants are
executed in CI. Recorded gaps are owned by INT-0017/0014/0019. Proceed to loop.
