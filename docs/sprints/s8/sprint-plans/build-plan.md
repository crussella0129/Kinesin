Finalized - DO NOT EDIT

# Sprint 8 Build Plan

## Intents
- [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) — state: planned; acceptance criteria covered: OWASP/NIST/CISA-mapped threat model, memory-safety statement enumerating the unsafe/FFI surface, a red-team corpus mapped to the release-evidence matrix and running in CI, versioned with an owner.

## Schema Tree
- Sprint Goal: a maintained, standards-mapped security-assurance package
  - Assurance doc
    - T-001: author docs/threat-model.md (OWASP/NIST/memory-safety/matrix map)
  - Corpus
    - T-002: consolidate the red-team corpus (tests/redteam.rs)

## Execution Sequence

### T-001: Author the assurance package
- **Intent:** [INT-0015](../../../intents/INT-0015-threat-model-assurance.md)
- **Touches:** docs/threat-model.md, docs/SUMMARY.md
- **Depends on:** (none)
- **Acceptance criterion:** the design maps to OWASP LLM/Agentic + NIST controls (mechanism/gap/residual-risk incl. transport-auth), the unsafe/FFI surface is enumerated, and the corpus maps to the release-evidence matrix; versioned + SUMMARY-linked.
- **Success criterion (EARS):**
  - **WHEN** `docs/threat-model.md` exists, **THEN** it **SHALL** map each OWASP LLM Top-10 (2025) / Agentic Top-10 (2026) risk and each NIST AI-agent control to a Kinesin mechanism, a gap naming the owning intent, or an accepted residual risk (including the service transport-auth/identity boundary), and **SHALL** be reachable from `docs/SUMMARY.md`.
  - **WHEN** the memory-safety statement is read, **THEN** it **SHALL** enumerate every `unsafe`/FFI source file (`private_state.rs`, `storage.rs`, `signal.rs`, `tools.rs`, `libc`) with a justification and record that the pure core carries no `unsafe`.
  - **WHEN** the corpus map is read, **THEN** each security.md release-evidence-matrix row **SHALL** name the executed test(s) proving it.
- **Notes:** include a version/owner/update-cadence header; gaps point at INT-0017 (approval gates/JIT), INT-0014 (tamper-evidence), INT-0019 (Windows sandbox).

### T-002: Consolidate the red-team corpus
- **Intent:** [INT-0015](../../../intents/INT-0015-threat-model-assurance.md)
- **Touches:** tests/redteam.rs
- **Depends on:** T-001
- **Acceptance criterion:** clearly-labeled red-team assertions run in CI proving a forbidden effect cannot pass the gate and content-borne instructions do not alter policy.
- **Success criterion (EARS):**
  - **WHEN** `tests/redteam.rs` runs, **THEN** an ungranted authorization attempt **SHALL** be denied and content-borne instructions **SHALL NOT** alter policy or authority.
- **Notes:** reuse the lightest existing harness (tools/policy layer); reference — not duplicate — the broader adversarial/auth/sandbox suites named in the T-001 matrix map.
