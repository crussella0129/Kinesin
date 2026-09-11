# Plan Critique — Sprint 6

Adversarial read-only screen of `build-plan.md` and `test-plan.md` against the
research report and INT-0013 (re-scoped this phase to the CI dependency gate).

## Concerns

### C-001: "CI fails on a violation" is verified structurally, not by fault injection
- **Where:** `build-plan.md` T-002 clause 2; `test-plan.md` `deny_denies_by_default`.
- **Quote:** "WHEN a dependency carries a known advisory or violates the `deny.toml` policy, THEN that job SHALL fail."
- **Failure mode:** weak-assertion.
- **Why it matters:** the plan proves the gate is deny-by-default and runs as a blocking step, not that a real violation turns CI red (no bad crate is injected).
- **Suggested response:** accept-with-rationale. Injecting a real advisory/banned crate to watch CI fail would pollute the dependency tree and the lockfile; the blocking behavior follows from `cargo deny check`/`cargo audit` exiting non-zero on a violation (no `|| true`/`continue-on-error`) plus a deny-by-default `licenses`/`sources` policy — both verifiable by reading the config and the job. This is the standard way CI policy gates are verified.

### C-002: the tool-install action is added trust on a supply-chain sprint
- **Where:** T-002 (install `cargo-deny`/`cargo-audit` via `taiki-e/install-action`).
- **Failure mode:** hidden-dep (screened).
- **Why it matters:** adding a third-party GitHub action to a *supply-chain* gate is itself supply-chain surface.
- **Suggested response:** defer-with-rationale. The action is pinned to a version (matching the repo's pinned-toolchain discipline), and the plan names `cargo install --locked` as the fallback if the pin is judged insufficient. The trade is deliberate and recorded, not incidental.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — T-001 (2 clauses) and T-002 (2 clauses) are measurable.
- **Plan-test mismatch:** none — every clause maps to a named check (`deny_check_passes_on_current_tree`, `audit_clean_or_documented`, `ci_has_blocking_supply_chain_job`, `supply_chain_job_green_at_checkpoint`, `deny_denies_by_default`), and each check traces to a clause.
- **Missing risk coverage:** none — advisory noise (author from current tree + justified ignores), action trust (pinned), CI placement (dedicated ubuntu job) all covered; release-artifact integrity was split out to the roadmap parking-lot with a stated trigger, not silently dropped.
- **Intent drift:** none — INT-0013 was re-scoped to the CI dependency gate with the change recorded in its transition history and the roadmap parking-lot; its acceptance now matches the sprint's work (no weakening-to-fit — a distinct outcome was separated).
- **Granularity:** none — T-001 policy, T-002 enforcement; distinct and coherent.
- **E2E status drift:** none — `possible` (CI), with the named `supply_chain_job_green_at_checkpoint`.

## Confidence
proceed-with-caveats
