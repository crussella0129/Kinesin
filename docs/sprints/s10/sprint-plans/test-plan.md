Finalized - DO NOT EDIT

# Sprint 10 Test Plan

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
| --- | --- | --- | --- |
| INT-0021 | Audit coverage, missing-outcome ownership and truthful provenance | T-001/A | book_intent_coverage_review |
| INT-0022 | Honest token dimensions and complete totals | T-002/A | usage_partial_fields_remain_unknown |
| INT-0022 | Honest token dimensions and complete totals | T-002/B | mixed_usage_never_claims_complete_totals |
| INT-0022 | Freeform compaction, checked evidence, replay and actual initial-context budgets | T-003/A | freeform_reads_compact_and_replay |
| INT-0022 | Freeform compaction, checked evidence, replay and actual initial-context budgets | T-003/B | continuation_initial_context_is_bounded |
| INT-0022 | Freeform compaction, checked evidence, replay and actual initial-context budgets | T-003/C | compaction_replay_version_compatibility |
| INT-0022 | Freeform compaction, checked evidence, replay and actual initial-context budgets | T-003/D | actual_session_cache_request_contract |
| INT-0022 | Encoded result limits and process lifecycle | T-004/A | command_encoded_result_is_bounded |
| INT-0022 | Encoded result limits and process lifecycle | T-004/B | command_lifecycle_reaps_descendants |
| INT-0022 | Mandatory effective Linux isolation | T-005/A | sandbox_requires_full_enforcement |
| INT-0022 | Mandatory effective Linux isolation | T-005/B | sandbox_adversarial_operations_are_denied |
| INT-0022 | MCP pre-decoding resource bounds and explicit process trust | T-006/A | mcp_protocol_and_discovery_are_bounded |
| INT-0022 | MCP pre-decoding resource bounds and explicit process trust | T-006/B | mcp_process_lifecycle_is_owned |
| INT-0022 | MCP admission, frozen authority, cancellation/deadline and replay | T-007/A | mcp_admission_precedes_spawn |
| INT-0022 | MCP admission, frozen authority, cancellation/deadline and replay | T-007/B | mcp_frozen_schema_cannot_expand_grants |
| INT-0022 | Durable definitions before dispatch; terminal startup failure | T-007/C | mcp_discovery_freeze_precedes_dispatch |
| INT-0022 | Confidential model URL policy | T-008/A | remote_plaintext_is_rejected |
| INT-0021, INT-0022 | Blocking reviewed dependency policy and current assurance evidence | T-009/A | dependency_gate_negative_policy |
| INT-0021, INT-0022 | Blocking reviewed dependency policy and current assurance evidence | T-009/B | assurance_mapping_review |
| INT-0022 | Repair documented move no-clobber behavior without claiming lease completion | T-010/A | move_collision_never_clobbers |
| INT-0021, INT-0022 | Integrated regression evidence, independent critique and PR | T-011/A | integrated_contract_verification |
| INT-0021, INT-0022 | Integrated regression evidence, independent critique and PR | T-011/B | sprint_close_evidence_review |

## Unit Tests

Each name below is a planned verification case; implementations may split a case
into multiple named Rust tests while preserving the same clause in the test report.
Documentation/CLI checks are named verification procedures, not fabricated Rust tests.

### T-001 unit tests
- **Intent:** INT-0021
- `book_intent_coverage_review` (T-001/A): the current Book is reviewed → it SHALL retain an assessment for every original criterion, create missing-outcome chapters, amend proposed ambiguities and explicitly supersede overclaimed terminal revisions.

### T-002 unit tests
- **Intent:** INT-0022
- `usage_partial_fields_remain_unknown` (T-002/A): usage fields are missing, partial, zero or overflowed → the model/runner SHALL expose only independently known exact counts and totals with complete dispatched-call coverage.
- `mixed_usage_never_claims_complete_totals` (T-002/B): streaming or nonstreaming calls mix reported usage with absent/error exchanges → inspect and journal SHALL preserve known per-event values without inventing full run totals.

### T-003 unit tests
- **Intent:** INT-0022
- `freeform_reads_compact_and_replay` (T-003/A): unchecked reads exceed historical history/evidence bottlenecks under enabled compaction → the run SHALL discard only complete eligible old groups and continue while keeping checked-run evidence protected.
- `continuation_initial_context_is_bounded` (T-003/B): a continuation's actual framed prior answer makes initial context exceed budget → authorization SHALL reject before admission/model dispatch.
- `compaction_replay_version_compatibility` (T-003/C): replay loads old or current semantic captures → it SHALL use an explicit compatible version or refuse unsupported versions without silently applying changed compaction.
- `actual_session_cache_request_contract` (T-003/D): cache validation is performed → it SHALL use actual harness session-prepared requests and distinguish offline invariants from unexecuted live timing evidence.

### T-004 unit tests
- **Intent:** INT-0022
- `command_encoded_result_is_bounded` (T-004/A): stdout/stderr contain large, escaped or invalid-UTF8 bytes → the command SHALL return valid nested JSON within the encoded result cap with honest truncation and exit status.
- `command_lifecycle_reaps_descendants` (T-004/B): a command is cancelled before spawn, times out, its leader exits with descendants, or its owning future drops → it SHALL avoid a pre-cancelled spawn and settle owned descendant cleanup before releasing normal completion resources; drop fallback SHALL stop further descendant effects.

### T-005 unit tests
- **Intent:** INT-0022
- `sandbox_requires_full_enforcement` (T-005/A): Linux cannot enforce required Landlock rights and seccomp rules → the command SHALL refuse with a defined sandbox outcome and never run partly confined.
- `sandbox_adversarial_operations_are_denied` (T-005/B): a Linux child attempts outside truncation, private-state reads, socket or io_uring network paths, or process-group escape → the sandbox SHALL deny the operation while permitting ordinary workspace operations under the documented runtime read exceptions.

### T-006 unit tests
- **Intent:** INT-0022
- `mcp_protocol_and_discovery_are_bounded` (T-006/A): a server emits an oversized/unterminated frame, cyclic pagination, excessive tools, or aggregate schema metadata → the client SHALL fail within explicit byte/count/deadline limits before unbounded accumulation.
- `mcp_process_lifecycle_is_owned` (T-006/B): an MCP server starts, fails initialization, times out, completes, or is cancelled → its process SHALL receive a scrubbed environment and no inherited stderr channel, and owned descendant teardown SHALL be awaited for normal completion and failure.

### T-007 unit tests
- **Intent:** INT-0022
- `mcp_admission_precedes_spawn` (T-007/A): capacity rejects a submission, a queued job is cancelled, or an idempotent retry resolves → no extra MCP process SHALL start; accepted active preparation SHALL stay within bounded controller ownership and the run deadline.
- `mcp_frozen_schema_cannot_expand_grants` (T-007/B): frozen MCP definitions include a tool outside the run grant → live dispatch and replay SHALL deny it independently of schema membership, and valid frozen calls SHALL reproduce offline.
- `mcp_discovery_freeze_precedes_dispatch` (T-007/C): startup completes/fails/times out/cancels → no model dispatch before durable definitions; failures settle terminal without model/tool dispatch, with pure replay and retained bounds.

### T-008 unit tests
- **Intent:** INT-0022
- `remote_plaintext_is_rejected` (T-008/A): a model URL is non-loopback regardless of private/CGNAT/ULA/public address class → configuration SHALL require HTTPS and retain separate public-destination opt-in; local HTTP, disabled redirects/proxies and request parity SHALL remain valid.

### T-009 unit tests
- **Intent:** INT-0021, INT-0022
- `dependency_gate_negative_policy` (T-009/A): an unreviewed duplicate dependency appears → the supply-chain gate SHALL fail; present required duplicates SHALL have explicit version-scoped reviewed exceptions and a durable cargo-vet/native-build decision.
- `assurance_mapping_review` (T-009/B): the final roadmap and assurance package are reviewed → they SHALL map both full risk taxonomies and native FFI, current mechanisms, residuals, owner/cadence, and proposed work without claiming unexecuted model/security evidence.

### T-010 unit tests
- **Intent:** INT-0022
- `move_collision_never_clobbers` (T-010/A): a destination exists or is concurrently created before move commits → the operation SHALL fail without replacing its contents, using capability-scoped atomic no-replace semantics and a defined source/result outcome.

### T-011 unit tests
- **Intent:** INT-0021, INT-0022
- `integrated_contract_verification` (T-011/A): the coherent repair set is complete → format, clippy, tests, dependency gates and supported Windows/Linux CI SHALL pass; any newly found contract regression SHALL be repaired and retested.
- `sprint_close_evidence_review` (T-011/B): the sprint exits → Book evidence SHALL reconcile tasks/intents/tests with an independent critic verdict, final verification and one dev-to-main PR; no unavailable two-host or live-cache evidence SHALL be marked passed.

## Integration Tests

- **INT-0022 / T-002 A–B:** model wire parsing + multi-call runner metadata + inspect;
  missing/partial/zero/error/overflow matrix for both response paths.
- **INT-0022 / T-003 A–D:** long freeform reads cross history and evidence ceilings,
  checked counterpart preserves verdict, repaired run replays, older semantic capture
  is explicitly compatible or refused, actual continuation requests stay bounded.
- **INT-0022 / T-004 A–B, T-005 A–B, T-010 A:** native command fixtures verify encoded
  envelopes, pre-cancel, normal-exit descendants, timeout/cancel/drop cleanup; Linux
  tests execute actual truncate/network/escape denial and supported workspace access;
  a synchronized move collision preserves destination bytes.
- **INT-0022 / T-006 A–B, T-007 A–B:** in-repo MCP fixture exercises giant frames,
  pagination/schema caps, startup failure, environment/stderr isolation, descendant
  teardown, capacity rejection, queued cancellation, idempotent retries and forged
  frozen schema. Valid calls keep journal/replay parity without reconnecting.
- **INT-0022 / T-008 A:** URL address-family matrix and unchanged HTTP destination,
  redirect, proxy and prepared-request tests.
- **INT-0021/0022 / T-009 A–B:** cargo-deny positive full policy and isolated negative
  duplicate policy, cargo audit; check all LLM/ASI risk IDs/native deps and source refs.

## End-to-End Tests

- **Status:** possible.
- Existing CLI run → inspect → export → pure replay plus command/MCP regressions
  verify T-002/T-003/T-004/T-006/T-007. Remove/disable the live server before replay.
- Service authenticated submit → bounded MCP preparation → cancel/terminal retrieval
  covers T-007 and preserves owner/idempotency/admission contracts.
- `integrated_contract_verification` (T-011/A): cargo fmt --check; cargo clippy
  --locked --all-targets -- -D warnings; cargo test --locked --all-targets; cargo
  deny check; cargo audit; Windows/Linux CI; focused reruns after coherent fixes.
- `sprint_close_evidence_review` (T-011/B): installed Book/phase/tracked validators,
  every affected criterion and task linked to actual tests, independent test critic,
  clean git state for committed artifacts and one dev-to-main PR.
- Live second-host transport proof is **not-yet-possible** unless the operator
  supplies that environment: unlocked by INT-0027. Actual cache timing is unlocked
  by INT-0026's provisioned pinned-model workload. Authoring tests does not count
  as executing these proofs; preserve unverified status and do not assert realization.

## Baseline and regression evidence

The unmodified Windows all-target suite passed during research. New negative cases
should first demonstrate the relevant old assumption where practical; the test
report must record actual commands/results and distinguish source-derived findings
from executed regressions. No deliberate secrets or vulnerable dependency additions
are needed. Linux enforcement tests must fail on a supposedly supporting CI kernel
when isolation is unavailable, while unsupported-kernel refusal is tested separately.
