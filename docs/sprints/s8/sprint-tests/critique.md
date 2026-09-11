# Test Critique — Sprint 8

Adversarial read-only screen of the sprint 8 assurance evidence against INT-0015.

## Concerns

### C-001: the assurance mapping's completeness is author-bounded
- **Where:** `docs/threat-model.md` §1–2; `corpus_maps_matrix`.
- **Failure mode:** weak-assertion.
- **Why it matters:** a threat-model table can omit a risk or overstate a mechanism.
- **Suggested response:** accept-with-rationale. Rows are mechanism / gap(owning intent) / accepted residual risk — never bare "covered"; the set is anchored to two published OWASP lists + the NIST controls + the security.md matrix; the doc is versioned/owned for revision. Same honest bound as any threat model.

### C-002: the executed corpus is two consolidating tests, not a new red-team framework
- **Where:** `tests/redteam.rs`.
- **Failure mode:** integration-drift.
- **Why it matters:** two assertions do not by themselves cover the whole matrix.
- **Suggested response:** defer-with-rationale (by design). The corpus is chiefly the existing, already-executing adversarial/auth/sandbox suites, mapped row-by-row in §4; `tests/redteam.rs` adds a labeled home for the two cross-cutting invariants (unauthorized-denied, content-as-data). The matrix map names the real proving tests for every row.

### C-003: content-as-data is asserted at the command boundary, not the model loop
- **Where:** `redteam_treats_content_as_data`.
- **Failure mode:** negative-path (screened).
- **Why it matters:** prompt-injection is model-facing; the test uses a command argument.
- **Suggested response:** accept-with-rationale. The command-arg test proves the concrete argv-only / no-shell guarantee (a forbidden effect from crafted content cannot execute); the model-facing prompt-injection-as-data property is proven by the `live_evaluation` hostile-text cards, which the §4 map cites. The two together cover the risk; neither alone claims to.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — each acceptance criterion maps to a named check (`threat_model_present_and_mapped`, `memory_safety_enumerates_unsafe`, `corpus_maps_matrix`, `redteam_denies_unauthorized`, `redteam_treats_content_as_data`).
- **Stub leakage:** none — real `CommandRunner`/policy path.
- **E2E cop-out:** none — corpus runs in CI; the doc is coverage-verified (appropriate for documentation).
- **Flake risk:** none — the denial is pre-spawn/deterministic; the content test is tolerant of the sandbox-refuse path and otherwise deterministic.
- **Evidence drift:** none — artifacts name tested head `8ea2219`, carry the confirmations, and identify the Test-evidence link on INT-0015.

## Confidence
proceed-with-caveats
