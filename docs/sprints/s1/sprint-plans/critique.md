# Plan Critique — Sprint 1

Adversarial read-only screen of `build-plan.md` / `test-plan.md` against the research
report and INT-0003, per the failure-mode checklist.

## Concerns

### C-001: T-005's EARS clause has no cargo test
- **Where:** `build-plan.md` T-005 / `test-plan.md` End-to-End Tests.
- **Quote:** "WHEN CI runs on a push or pull request, THEN it SHALL run `fmt`, `clippy`, and `cargo test --locked` on both `windows-latest` and `ubuntu-latest`."
- **Failure mode:** plan-test-mismatch / e2e-drift.
- **Why it matters:** every other EARS clause maps to a named cargo test; this one is verified only by CI configuration and the checkpoint's dual-OS run, so an automated in-suite test cannot prove it.
- **Suggested response:** defer-with-rationale. A workflow matrix is CI configuration, not library behavior, so it has no unit/integration test by nature. Its verification is explicit and checkable: the workflow file names both OSes, and both jobs must report green at the sprint checkpoint (recorded under the Test Phase report's CI Confirmation). This is the same evidence class the Test Phase already uses for CI conclusions, so the clause is verifiable — just not by `cargo test`. Accepted as a declared non-unit verification rather than a coverage gap.

### C-002: run_command executes on tools.rs across T-002 and T-003
- **Where:** `build-plan.md` T-002 and T-003 both touch `src/tools.rs`.
- **Quote:** T-002 "Touches: src/tools.rs"; T-003 "Touches: src/tools.rs, Cargo.toml, src/bin/cmd-fixture.rs".
- **Failure mode:** hidden-dep.
- **Why it matters:** two tasks editing the same file can collide if built out of order.
- **Suggested response:** reject (the critique is wrong because ...). The dependency is explicit and ordered: T-003 `Depends on: T-001, T-002`. T-002 is the pure arg/shape/validation surface; T-003 adds the execution capability that consumes it. Sequential edits to one file under a declared dependency are the normal case, not a hidden dependency. No change needed.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — every `### T-NNN` carries at least one measurable `WHEN … THEN … SHALL` clause.
- **Plan-test mismatch (beyond C-001):** none — each code EARS clause maps to a named test in the traceability table, and each named test cites its clause.
- **Missing risk coverage:** none — the research risks (process-tree cleanup, kill-vs-detach execution model, cap-std/real-cwd, allow-list shape, timeout/non-zero classification, tokio-process dependency) each land on a task and a test.
- **Intent drift:** none — INT-0003 is linked and `planned`; every acceptance criterion has planned work and verification; the design choices (distinct `CommandRunner`, `command-group`, CI matrix) are recorded in the intent's Transition history, not only in plan prose.
- **Granularity:** none — tasks split cleanly into grant/policy, args, execution, integration, and infrastructure.

## Confidence
proceed-with-caveats
