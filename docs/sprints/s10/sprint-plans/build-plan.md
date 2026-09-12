Finalized - DO NOT EDIT

# Sprint 10 Build Plan

## Intents
- [INT-0021](../../../intents/INT-0021-harness-contract-review.md) — planned: audit baseline, roadmap and assurance revision.
- [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md) — planned: all concrete repair criteria below.

## Authorization and source freeze
The user's request explicitly authorizes Research, Plan, Build, TEST, final
verification, Loop and PR submission. That authorization applies to these repairs.
This Codex tool surface exposes neither EnterPlanMode nor ExitPlanMode; preserve
the phase's substantive source freeze and independent critique, without inventing
a tool call or requesting redundant permission. No implementation files changed
while these plans were drafted. The remote merge policy remains human-approve.

## Schema Tree
- Sprint goal: intent-complete audit and completed-contract repair
  - Intent ownership: T-001
  - Core/context: T-002, T-003
  - Commands/platform/workspace: T-004, T-005, T-010
  - MCP: T-006, T-007
  - Transport: T-008
  - Assurance and integration: T-009, T-011

## Execution Sequence

### T-001: Revise intent ownership and lifecycle evidence
- **Intent:** [INT-0021](../../../intents/INT-0021-harness-contract-review.md)
- **Touches:** docs/intents/*.md; docs/SUMMARY.md; sprint research/meta
- **Depends on:** none
- **Acceptance criterion:** Audit coverage, missing-outcome ownership and truthful provenance
- **Success criterion (EARS):**
  - A: **WHEN** the current Book is reviewed, **THEN** it SHALL retain an assessment for every original criterion, create missing-outcome chapters, amend proposed ambiguities and explicitly supersede overclaimed terminal revisions.

### T-002: Preserve optional usage and exact total coverage
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/model.rs; src/runner.rs; tests/model_protocol.rs; tests/runner_tools.rs; tests/cli_inspect.rs
- **Depends on:** T-001
- **Acceptance criterion:** Honest token dimensions and complete totals
- **Success criterion (EARS):**
  - A: **WHEN** usage fields are missing, partial, zero or overflowed, **THEN** the model/runner SHALL expose only independently known exact counts and totals with complete dispatched-call coverage.
  - B: **WHEN** streaming or nonstreaming calls mix reported usage with absent/error exchanges, **THEN** inspect and journal SHALL preserve known per-event values without inventing full run totals.

### T-003: Repair freeform read compaction and continuation admission
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/core.rs; src/policy.rs; src/runner.rs; src/replay.rs; src/verification.rs; tests/runner_tools.rs; tests/replay.rs; tests/live_evaluation.rs
- **Depends on:** T-002
- **Acceptance criterion:** Freeform compaction, checked evidence, replay and actual initial-context budgets
- **Success criterion (EARS):**
  - A: **WHEN** unchecked reads exceed historical history/evidence bottlenecks under enabled compaction, **THEN** the run SHALL discard only complete eligible old groups and continue while keeping checked-run evidence protected.
  - B: **WHEN** a continuation's actual framed prior answer makes initial context exceed budget, **THEN** authorization SHALL reject before admission/model dispatch.
  - C: **WHEN** replay loads old or current semantic captures, **THEN** it SHALL use an explicit compatible version or refuse unsupported versions without silently applying changed compaction.
  - D: **WHEN** cache validation is performed, **THEN** it SHALL use actual harness session-prepared requests and distinguish offline invariants from unexecuted live timing evidence.

### T-004: Repair command result bounds and owned process cleanup
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/tools.rs; src/process.rs; src/lib.rs; Cargo.toml; src/bin/cmd-fixture.rs; tests/command_tool.rs; tests/runner_tools.rs; tests/replay.rs
- **Depends on:** T-001
- **Acceptance criterion:** Encoded result limits and process lifecycle
- **Success criterion (EARS):**
  - A: **WHEN** stdout/stderr contain large, escaped or invalid-UTF8 bytes, **THEN** the command SHALL return valid nested JSON within the encoded result cap with honest truncation and exit status.
  - B: **WHEN** a command is cancelled before spawn, times out, its leader exits with descendants, or its owning future drops, **THEN** it SHALL avoid a pre-cancelled spawn and settle owned descendant cleanup before releasing normal completion resources; drop fallback SHALL stop further descendant effects.

### T-005: Close Linux sandbox truncation, network and private-state gaps
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/tools.rs; src/config.rs; tests/sandbox_linux.rs; src/bin/cmd-fixture.rs; docs/security.md
- **Depends on:** T-004
- **Acceptance criterion:** Mandatory effective Linux isolation
- **Success criterion (EARS):**
  - A: **WHEN** Linux cannot enforce required Landlock rights and seccomp rules, **THEN** the command SHALL refuse with a defined sandbox outcome and never run partly confined.
  - B: **WHEN** a Linux child attempts outside truncation, private-state reads, socket or io_uring network paths, or process-group escape, **THEN** the sandbox SHALL deny the operation while permitting ordinary workspace operations under the documented runtime read exceptions.

### T-006: Bound MCP protocol and own server process lifecycle
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/mcp.rs; src/bin/mcp-fixture.rs; tests/mcp.rs; Cargo.toml if required
- **Depends on:** T-001; shared process-owner integration from T-004
- **Acceptance criterion:** MCP pre-decoding resource bounds and explicit process trust
- **Success criterion (EARS):**
  - A: **WHEN** a server emits an oversized/unterminated frame, cyclic pagination, excessive tools, or aggregate schema metadata, **THEN** the client SHALL fail within explicit byte/count/deadline limits before unbounded accumulation.
  - B: **WHEN** an MCP server starts, fails initialization, times out, completes, or is cancelled, **THEN** its process SHALL receive a scrubbed environment and no inherited stderr channel, and owned descendant teardown SHALL be awaited for normal completion and failure.

### T-007: Move MCP preparation under bounded run ownership and enforce both gates
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/scheduler.rs; src/service.rs; src/cli.rs; src/runner.rs; src/policy.rs; src/replay.rs; tests/mcp.rs; tests/service.rs
- **Depends on:** T-003, T-006
- **Acceptance criterion:** MCP admission, frozen authority, cancellation/deadline and replay
- **Success criterion (EARS):**
  - A: **WHEN** capacity rejects a submission, a queued job is cancelled, or an idempotent retry resolves, **THEN** no extra MCP process SHALL start; accepted active preparation SHALL stay within bounded controller ownership and the run deadline.
  - B: **WHEN** frozen MCP definitions include a tool outside the run grant, **THEN** live dispatch and replay SHALL deny it independently of schema membership, and valid frozen calls SHALL reproduce offline.
  - C: **WHEN** an admitted run prepares MCP, **THEN** discovered definitions SHALL be durably frozen before the first model dispatch, and startup failure/timeout/cancellation SHALL settle a defined terminal outcome without dispatch.

### T-008: Enforce confidential remote model origins
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/config.rs; tests/runner_tools.rs; tests/model_protocol.rs; docs/configuration.md; docs/integration.md; tests/live_evaluation.rs
- **Depends on:** T-001
- **Acceptance criterion:** Confidential model URL policy
- **Success criterion (EARS):**
  - A: **WHEN** a model URL is non-loopback regardless of private/CGNAT/ULA/public address class, **THEN** configuration SHALL require HTTPS and retain separate public-destination opt-in; local HTTP, disabled redirects/proxies and request parity SHALL remain valid.

### T-009: Repair dependency gate and refresh assurance/roadmap
- **Intent:** [INT-0021](../../../intents/INT-0021-harness-contract-review.md), [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** deny.toml; docs/threat-model.md; docs/roadmap.md; docs/security.md; docs/loop-and-tools.md; docs/configuration.md; docs/supply-chain.md; README.md
- **Depends on:** T-001; final reconciliation after T-005, T-007, T-008
- **Acceptance criterion:** Blocking reviewed dependency policy and current assurance evidence
- **Success criterion (EARS):**
  - A: **WHEN** an unreviewed duplicate dependency appears, **THEN** the supply-chain gate SHALL fail; present required duplicates SHALL have explicit version-scoped reviewed exceptions and a durable cargo-vet/native-build decision.
  - B: **WHEN** the final roadmap and assurance package are reviewed, **THEN** they SHALL map both full risk taxonomies and native FFI, current mechanisms, residuals, owner/cadence, and proposed work without claiming unexecuted model/security evidence.

### T-010: Make workspace move preserve an existing destination atomically
- **Intent:** [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** src/tools.rs; tests/runner_tools.rs; tools unit tests; docs/security.md
- **Depends on:** T-004
- **Acceptance criterion:** Repair documented move no-clobber behavior without claiming lease completion
- **Success criterion (EARS):**
  - A: **WHEN** a destination exists or is concurrently created before move commits, **THEN** the operation SHALL fail without replacing its contents, using capability-scoped atomic no-replace semantics and a defined source/result outcome.

### T-011: Verify integrated contracts and finish the sprint checkpoint
- **Intent:** [INT-0021](../../../intents/INT-0021-harness-contract-review.md), [INT-0022](../../../intents/INT-0022-completed-contract-repairs.md)
- **Touches:** tests and docs/sprints/s10/sprint-tests; docs/work; sprint-meta; affected documentation
- **Depends on:** T-002, T-003, T-004, T-005, T-006, T-007, T-008, T-009, T-010
- **Acceptance criterion:** Integrated regression evidence, independent critique and PR
- **Success criterion (EARS):**
  - A: **WHEN** the coherent repair set is complete, **THEN** format, clippy, tests, dependency gates and supported Windows/Linux CI SHALL pass; any newly found contract regression SHALL be repaired and retested.
  - B: **WHEN** the sprint exits, **THEN** Book evidence SHALL reconcile tasks/intents/tests with an independent critic verdict, final verification and one dev-to-main PR; no unavailable two-host or live-cache evidence SHALL be marked passed.

## Execution ownership and integration

Core/replay edits T-002/T-003 complete before MCP integration T-007 touches
runner/policy/replay. Command tasks share one owner and serialize T-004/T-005/T-010.
MCP transport T-006 can proceed independently; T-007 integrates it after core work.
T-009 may draft independent deny/threat/roadmap changes early, but final statements
and shared security/configuration docs are reconciled after runtime changes.
Root owns all intent/ledger transitions and commits. No blanket source rewrites.

## Explicit outstanding capability and evidence

INT-0006/0007/0009/0010/0014/0016/0017/0018/0019/0020 remain proposed;
criteria amendments do not implement those features. INT-0023..0027 own newly
identified operational/evaluation/secret/session/deployment outcomes. Full session
history/token admission and real two-host/live-session cache proof remain proposed,
with the prior INT-0004/0008 completion overclaims explicitly superseded. T-003
repairs actual context admission and adds faithful benchmark coverage; it does not
claim that writing an ignored benchmark supplies a live result. Stale-edit fencing
and durable multi-writer coordination remain INT-0010;
T-010 repairs only the existing move no-clobber promise. Linux syscall/process
escape restrictions must be documented; Windows Job Objects are not AppContainer.

Shared process ownership may use a small audited Unix process-group / Windows Job
RAII helper. Unix cleanup kills the owned group and reaps its direct child;
orphan zombie reaping belongs to the OS, not a claim of harness subreaper support.
Trusted MCP binaries must not deliberately escape their owned group. Command
sandboxing denies group/session escape. A capability-scoped hard-link then remove
move is acceptable if destination publication is atomic no-replace and a failed
source cleanup is reported explicitly; do not call the two-name operation atomic.

The audit did not establish an application leak of already-open private descriptors.
Pathname/proc-alias denial is covered by T-005; comprehensive inherited-descriptor
closure and a synthetic descriptor inventory remain explicitly proposed under
INT-0025, before support for untrusted embedding/ambient descriptors. Standard Rust
files use close-on-exec and no intentional descriptor passing is offered. Do not
claim pathname sandboxing can revoke pre-opened descriptor access.
