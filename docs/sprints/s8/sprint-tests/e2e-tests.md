# Sprint 8 End-to-End Tests

- **Intent:** [INT-0015](../../../intents/INT-0015-threat-model-assurance.md)
- **Tested head:** `8ea22196b83af80007cc22e42641518fec4f7ca2`
- **Status:** possible (CI) for the corpus; the assurance doc is verified by coverage.

The `tests/redteam.rs` corpus runs in the ubuntu + windows `check` jobs (and the
`supply-chain` job stays green — no dependency change). The assurance package
(`docs/threat-model.md`) is a documentation deliverable verified by the
structural/coverage checks in [integration-tests.md](integration-tests.md) — a
threat model is validated by mapping completeness and by the executed tests it
points at, not by running a program. The gaps it records (approval gates/JIT →
INT-0017, tamper-evidence → INT-0014, Windows sandbox → INT-0019, mTLS residual
boundary) are owned by their intents and are not asserted as delivered here.
