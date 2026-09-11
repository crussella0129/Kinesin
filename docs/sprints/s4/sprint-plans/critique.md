# Plan Critique — Sprint 4

Adversarial read-only screen of `build-plan.md` and `test-plan.md` against the
research report and INT-0008.

## Concerns

### C-001: T-002 bundles an integration test, a readiness test, a live test, and a docs runbook
- **Where:** `build-plan.md` T-002 ("Uniform attach proof + 'add a machine' runbook").
- **Quote:** "an integration test … a readiness test … an `#[ignore]`d live attach … a short Tailscale 'add a machine' runbook."
- **Failure mode:** granularity.
- **Why it matters:** a task mixing verification and documentation can produce an incoherent diff.
- **Suggested response:** defer-with-rationale. All four deliverables serve one observable outcome — "a run attaches to a non-loopback backend the same way it attaches to localhost, and an operator can reproduce it." This matches the project's established shape (s1 T-005 CI, s3 T-002 offline-invariants + live harness). Each deliverable maps to a named EARS clause or acceptance row, so the coherence is real, not nominal.

### C-002: the "no code change to add a backend" criterion is verified only indirectly
- **Where:** `test-plan.md` Intent Traceability, row "adding a backend needs no code change" → `origin_accepts_private_and_overlay_http`.
- **Failure mode:** intent-drift (weak coverage).
- **Why it matters:** accepting an overlay address in `validate_origin` is a proxy for "no code change," not a direct proof that a run reaches a new backend via config alone.
- **Suggested response:** accept as layered. The direct proof is `uniform_attach_prepares_identically_across_local_and_overlay_backends` (a run built purely from a config differing only in `base_url`) plus the existing origin-keyed registry in `RunResources::from_config`; the unit test proves the new address is admissible. Together they establish config-only addition without a dedicated redundant test.

### C-003: the live acceptance depends on an operator-provided reachable backend
- **Where:** `test-plan.md` E2E / `attach_to_non_loopback_backend`.
- **Failure mode:** e2e-drift.
- **Why it matters:** the headline "attach over a real non-loopback address" cannot run in CI and needs a `llama-server` on the host's LAN/tailnet IP.
- **Suggested response:** defer-with-rationale. Same split INT-0004 used and the critic accepted: the offline integration test (`uniform_attach_prepares_identically…`) is the CI-verifiable stand-in for the code path; the live attach is an `#[ignore]`d recorded measurement. Using the host's own LAN IP makes it reproducible on one machine without a second host.

## Screen of the remaining failure modes
- **Vague/absent EARS:** none — T-001 has four measurable clauses, T-002 three (including the live clause), each `WHEN … THEN … SHALL …`.
- **Plan-test mismatch:** none — every EARS clause maps to a named test and every planned test traces to a clause or acceptance row (live clause → `attach_to_non_loopback_backend`).
- **Missing risk coverage:** none — the HTTPS-forcing gap, the CGNAT/ULA range definition, unreachable/partial-failure, and the Tailscale-vs-Koil unknown each land on a task, test, or explicit deferral (INT-0009, and multi-backend routing out of scope).
- **Hidden dependencies:** none — T-002 depends on T-001 (needs overlay addresses to be admissible); touched paths are disjoint (config.rs vs tests/docs).
- **Intent drift:** none — INT-0008 is linked and `planned`; the private-by-default posture and the `allow_public_endpoints` opt-in are recorded in the intent, not only in the plan.
- **E2E status drift:** none — status is `possible` with a named live test and an offline stand-in; no unlocking intent needed.

## Confidence
proceed-with-caveats
