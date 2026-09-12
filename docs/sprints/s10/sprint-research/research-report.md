# Sprint 10 Research Report

## Intents Reviewed
- [INT-0001](../../../intents/INT-0001-token-accounting.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0002](../../../intents/INT-0002-context-compaction.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0003](../../../intents/INT-0003-shell-execution.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0005](../../../intents/INT-0005-mcp-tool-servers.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0006](../../../intents/INT-0006-skills-progressive-disclosure.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0007](../../../intents/INT-0007-managed-model-process.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0009](../../../intents/INT-0009-koil-overlay-transport.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0010](../../../intents/INT-0010-cross-agent-write-coordination.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0011](../../../intents/INT-0011-production-readiness-roadmap.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0012](../../../intents/INT-0012-command-execution-sandboxing.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0013](../../../intents/INT-0013-supply-chain-security.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0014](../../../intents/INT-0014-tamper-evident-journal.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0015](../../../intents/INT-0015-threat-model-assurance.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0016](../../../intents/INT-0016-observability.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0017](../../../intents/INT-0017-approval-gates-jit-privilege.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0018](../../../intents/INT-0018-subagents-parallel-orchestration.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0019](../../../intents/INT-0019-windows-command-sandboxing.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0020](../../../intents/INT-0020-remote-mcp-delegated-auth.md) — selected for criterion-level audit; historical state preserved during research; see the linked audit artifacts.
- [INT-0021](../../../intents/INT-0021-harness-contract-review.md) — created as proposed; selected for this sprint's plan.
- [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md) — created as proposed; selected for this sprint's plan.
- [INT-0023](../../../intents/INT-0023-lifecycle-data-stewardship.md) — created as proposed; newly tracked outcome, not selected for feature implementation.
- [INT-0024](../../../intents/INT-0024-harness-evaluation.md) — created as proposed; newly tracked outcome, not selected for feature implementation.
- [INT-0025](../../../intents/INT-0025-secrets-and-egress.md) — created as proposed; newly tracked outcome, not selected for feature implementation.
- [INT-0026](../../../intents/INT-0026-session-context-continuity.md) — created as proposed; newly tracked outcome, not selected for feature implementation.
- [INT-0027](../../../intents/INT-0027-encrypted-remote-deployment.md) — created as proposed; newly tracked outcome, not selected for feature implementation.

## 1. Sprint Goal

Review the complete harness intent set before reading implementation, identify
missing outcomes, then measure code/tests against all original intents. Plan and
execute concrete repairs to already completed contracts, revise intent ownership
and assurance, run TEST and final verification, and submit one dev-to-main PR.
Implementing every proposed roadmap feature is outside this repair sprint.

## 2. Existing Code Survey

| File | Relevance | Notes |
| --- | --- | --- |
| src/core.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/model.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/policy.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/runner.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/replay.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/config.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/tools.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/mcp.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/scheduler.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/service.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/cli.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/auth.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/operator.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/private_state.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/storage.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/storage_schema.sql | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/signal.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/verification.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/dispatch.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/ingress.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/bin/cmd-fixture.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| src/bin/mcp-fixture.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| Cargo.toml | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| Cargo.lock | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| deny.toml | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| .github/workflows/ci.yml | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/command_tool.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/sandbox_linux.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/runner_tools.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/replay.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/mcp.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/model_protocol.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/cli_inspect.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/process_recovery.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/service.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/redteam.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/live_evaluation.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| tests/settlement.rs | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| docs/threat-model.md | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| docs/roadmap.md | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| docs/security.md | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| docs/integration.md | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| docs/loop-and-tools.md | high | Criterion-level source/test assessment in the specialized audit artifacts below. |
| README.md | high | Criterion-level source/test assessment in the specialized audit artifacts below. |

The independent intent-only review completed before source inspection. Audits
confirm useful foundations rather than a missing runtime: pure core, immutable
owner-scoped admission/journal, checked receipts, recovery, retention/backup and
bounded concurrent runs are substantive. Ten historical intents are realized;
ten are proposed. The realized claims have material gaps in accounting,
compaction, command confinement/lifecycle, MCP bounds, transport proof, dependency
policy and assurance documentation. Detailed tables assess each criterion rather
than using one aggregate completion percentage.

## 3. External Sources
- [MCP stdio transport](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports) — protocol framing and stderr are distinct surfaces.
- [MCP security best practices](https://modelcontextprotocol.io/docs/2025-11-25/tutorials/security/security_best_practices) — destination-scoped credentials and local-process trust.
- [Landlock userspace API](https://docs.kernel.org/userspace-api/landlock.html) — truncation is a separate handled right; require sufficiently strong ABI.
- [OWASP Agentic 2026](https://genai.owasp.org/download/52117/?tmstv=1765059207) — all ten Agentic risks need explicit dispositions.
- [OWASP LLM index](https://genai.owasp.org/llm-top-10/) — keep original LLM2025 mapping version explicit while checking taxonomy.

Pinned local rmcp 3.3.0, command-group 5.0.1 and landlock 0.4.7 source was also
inspected; these are implementation evidence, not assumptions about latest APIs.
An OpenAI eval-guide lookup supplied no substantive evidence and is not relied on.

## 4. Risks, Unknowns, Dependencies

- **Risk:** historical green tests encode or miss some faulty assumptions. Initial
  `cargo test --locked --all-targets` passed on Windows before repairs, including
  155 unit tests and the integration suites; ignored live tests remain unexecuted.
- **Risk:** Linux ABI v1 fails to cover truncation; system/executable directory
  grants are too broad; io_uring can evade a syscall-only network filter.
- **Risk:** process cleanup must include normal leader exit and cancelled/dropped
  futures; command result bounds must cover the encoded envelope.
- **Risk:** MCP pre-admission spawns, unbounded SDK line/page accumulation,
  inherited environment/stderr and plain-child teardown violate claimed bounds.
- **Risk:** freeform read evidence blocks compaction; changing these semantics
  requires explicit replay compatibility, not silent reinterpretation.
- **Unknown:** no provisioned second physical model host has been established.
  Same-host LAN evidence is insufficient; retain INT-0027 as proposed.
- **Unknown:** original cache benchmark bypasses session assembly and does not
  establish actual session speed/concurrency claims. INT-0026 preserves that gap;
  add a faithful benchmark without claiming an unexecuted live result.
- **Dependency:** actual Linux enforcement will require WSL or CI; Windows
  AppContainer remains INT-0019, not a delivered consequence of Linux fixes.
- **Dependency:** GitHub profile is dev -> main, human-approve. User explicitly
  requested submitting the PR; merging is left to the operator. No updater PRs
  were open at sprint initialization.

## 5. Recommended Approach

Primary: advance INT-0021 and INT-0022 through a cross-cutting repair sprint.
Amend proposed skills, leases, signing, telemetry, approvals, subagents and remote
MCP criteria where the intent-only pass found ambiguity; add explicit missing
outcomes. Preserve terminal history via follow-on chapters and supersession where
appropriate. Repair actual defects with focused negative regressions and a full
cross-boundary validation run. Update threat/roadmap/operator guidance to the
resulting behavior and exact evidence limits. Track genuine unfinished capability
outcomes as proposed, never as a substitute for repairing concrete defects.

Alternative considered: only edit completion prose, or rewrite the entire harness.
Neither meets the request: known defects need fixes, while the existing core and
journal architecture already provide useful contracts worth preserving.

## Artifacts

- [Intent-only baseline](intent-first-review.md) — all intents reviewed before code.
- [Core audit](core-audit.md) — accounting, context, cache, replay and proposed capabilities.
- [Command audit](command-audit.md) — commands, sandbox, workspace races and OS parity.
- [Transport/MCP audit](transport-mcp-audit.md) — confidentiality, discovery, process and authority gates.
- [Assurance audit](assurance-audit.md) — roadmap, dependency policy, threat model and foundational contracts.

## Budget Override

The user explicitly requires all intents and a whole-harness implementation audit.
That crosses core, effects, platform isolation, protocol, storage, service, tests
and CI, so the twenty-file default cannot support the requested criterion-level
assessment. The survey lists 44 first-party files plus the 20 original intent
chapters, historical evidence and pinned dependency source. Parallel reviewers
kept this within the thirty-minute research window (started 14:43 UTC); five
substantive primary external sources are used. Extra file coverage is justified
by the cross-cutting request, not permission to enlarge feature-build scope.
