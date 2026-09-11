# Plan Critique — Sprint 8

Adversarial read-only screen of `build-plan.md` and `test-plan.md` against the
research report and INT-0015.

## Concerns

### C-001: mapping completeness is bounded by the author's thoroughness
- **Where:** T-001 OWASP/NIST/matrix tables.
- **Failure mode:** intent-drift (weak coverage).
- **Why it matters:** a threat-model table can silently omit a risk or overstate a mechanism as "covered".
- **Suggested response:** accept-with-rationale. Every row is a mechanism, a **gap** naming its owning intent, or an accepted residual risk — never bare "covered"; the set is anchored to two published OWASP lists + the NIST controls + the security.md matrix (not an ad-hoc list); and the package is versioned/owned so a later-found gap is added. This is the same honest bound as any threat model.

### C-002: the new red-team test is small and overlaps the existing suite
- **Where:** T-002 `tests/redteam.rs`.
- **Failure mode:** integration-drift.
- **Why it matters:** the two assertions partly restate coverage already in `command_tool`/`service`/`adversarial_runtime`.
- **Suggested response:** defer-with-rationale (deliberate). The corpus is chiefly the *mapping* of matrix rows to the tests already proving them (T-001); `tests/redteam.rs` exists to give the corpus a labeled, discoverable home and to assert the two cross-cutting invariants (unauthorized-denied, content-as-data) in one place. It references — does not duplicate — the broader suites.

### C-003: the mTLS/transport-auth boundary is documented, not implemented
- **Where:** T-001 NIST table (transport-auth residual risk).
- **Failure mode:** intent-drift (screened).
- **Why it matters:** a reader might expect INT-0015 to deliver mTLS.
- **Suggested response:** reject (the concern misreads the intent). INT-0015 is the *assurance* intent; its job is to record the boundary as a residual risk with its trigger (non-loopback exposure), not to build mTLS. The intent text and the roadmap say so; implementation is future work when a non-loopback service is built.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — T-001 (3 clauses) and T-002 (1 clause) are measurable.
- **Plan-test mismatch:** none — each clause maps to a named check (`threat_model_present_and_mapped`, `memory_safety_enumerates_unsafe`, `corpus_maps_matrix`, `redteam_denies_unauthorized`, `redteam_treats_content_as_data`).
- **Missing risk coverage:** none — over-claim, corpus duplication, and mapping completeness each land on a mitigation or accepted caveat.
- **Granularity:** none — assurance doc vs. corpus test are distinct.
- **E2E status drift:** none — `possible`; `tests/redteam.rs` runs in the CI check jobs; structural doc checks are by-inspection (as prior doc sprints), with check-book CI-gated.

## Confidence
proceed-with-caveats
