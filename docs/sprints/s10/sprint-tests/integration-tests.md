# Sprint 10 integration verification

These are subsets of the complete platform suite, not extra tests to add to its
totals. Every locked T-002 through T-010 clause is mapped in
[unit-tests.md](unit-tests.md); this chapter records the cross-layer checks.

| Boundary / clauses | Executed checks | Observed contract |
| --- | --- | --- |
| Model → runner → inspect, T-002/A–B | `tests/model_protocol.rs`, usage cases in `tests/runner_tools.rs` and `tests/cli_inspect.rs` | Streaming/nonstreaming optional dimensions, mixed missing/error exchanges and terminal totals remain honest |
| Context → checker → replay, T-003/A–C | Long-read freeform compaction, checked evidence survival, semantic-version refusal and continuation-origin replay tests | Compaction permits continued freeform work without changing checked acceptance; actual initial context remains bounded |
| Harness session → prepared HTTP, T-003/D | `actual_session_cache_request_contract`; manual actual-session timing test | The benchmark executes normal continuation authorization and verifies journaled outbound hashes |
| Commands → OS process owner, T-004/A–B | All 12 command tests plus CLI/runner command journaling | Encoded bounds, pre-cancel, timeout, leader-exit descendants and dropped-owner teardown preserve recorded outcomes |
| Linux child → kernel, T-005/A–B | All nine mandatory sandbox tests, forced-enforcement failure, process/config/filter units | Real outside read/truncate, socket/io_uring, process-group/namespace and alternate-ABI attempts are denied; normal workspace operations succeed |
| MCP transport → process owner, T-006/A–B | Oversized/unterminated/tiny frames, discovery/page/schema/count caps; scrubbed environment/stderr; startup and call failure/cancellation teardown | Input limits precede decoding, and normal/failure completion awaits owned cleanup |
| Controller → MCP → durable journal, T-007/A,C | `mcp_controller_capacity_queue_cancel_and_idempotency_never_start_extra_processes`, service equivalent, preparation cancel/deadline/pre-cancel, journal-error recovery | Rejected/queued/cancelled/retried submissions start no extra server; accepted startup freezes schemas before dispatch or settles a defined stop |
| Frozen schemas → grant → replay, T-007/B–C | `mcp_prepared_authority_cannot_grant_a_tool_or_spawn`, altered/removed/ungranted startup captures, unapproved-call and echo replay tests | A schema cannot enlarge authority; replay validates startup ordering and both gates without reconnecting |
| Origin policy → HTTP, T-008/A | Address-family config matrix plus all 17 protocol tests | HTTPS required outside loopback; ambient proxies and redirects remain disabled; prepared bytes retain parity |
| Dependency graph → CI, T-009/A | Full deny/audit and removal of one exact reviewed exception in an isolated policy | Unreviewed duplicate versions produce a blocking failure; no broad subtree bypass remains |
| Capability directory → move publication, T-010/A | Four move units, including synchronized collision and competing publishers | Existing/concurrent destination contents survive; source cleanup failure records the published destination |

The isolated MCP environment worker has `#[ignore]` so that its parent can
execute it in a dedicated process with synthetic sentinel environment variables.
The parent invokes and checks it during the ordinary suite; it is not an omitted
environment test. Other ignored manual tests retain their separate status.

Unix cleanup terminates the owned process group and reaps its direct child;
orphan zombie reaping remains the OS's responsibility. Trusted MCP executables
must not deliberately escape the group. Linux content restrictions do not claim
complete metadata/same-user isolation, and Windows Jobs do not provide the
proposed AppContainer boundary. The atomic no-replace guarantee covers move
destination publication, not a transactional two-name move or stale-source lease.
