# Sprint 10 command, sandbox, and workspace audit

Implementation baseline: `cab59aa4a2ed6ca79380e88035a549b50a177dc2`.
The intent-only baseline was completed before this code review. This pass read
INT-0003, INT-0010, INT-0012, INT-0019, the frozen intent-first review, their
implementations, test sources, and the installed `command-group` 5.0.1 and
`landlock` 0.4.7 sources. No implementation was changed and no new enforcement
test was executed during this read-only research pass. Prior platform evidence
is distinguished below from current source inspection.

## Criterion assessment

| Intent / criterion | Assessment against current implementation |
| --- | --- |
| INT-0003: granted freeform command, captured output, timeout, journal | Partial. The capability and dispatch are present, tool effects are journalled, and raw captures are capped. Encoded result bytes exceed the operator's result budget; process ownership has lifecycle gaps described below. Existing coverage: `tests/command_tool.rs`, `command_effect_is_journalled` in `tests/runner_tools.rs`. |
| INT-0003: argv, no shell string | Satisfied by source and literal-metacharacter test: `Command::new(argv[0]).args(argv[1..])`; allow-list and bare executable validation. The operator must still trust PATH and the selected executable. |
| INT-0003: deny without grant; bar checked runs | Satisfied by capability construction, authority checks, config checked-task validation, and `run_command_denied_without_grant` / `checked_workspace_cannot_grant_run_command`. |
| INT-0003: defined timeout, oversize, nonzero, missing executable, midrun cancellation outcomes | Present, with test sources covering each. Cancellation test proves bounded settlement, not independently that every descendant died; timeout marker test does not first prove that the grandchild actually wrote. Already-cancelled dispatch, normal leader exit with a live descendant, and dropped execution future lack coverage. |
| INT-0012: actual Linux read/socket denial | Existing tests and prior sprint evidence establish a conventional out-of-workspace read and ordinary socket denial. This is partial confinement: ABI-v1 truncation and alternate network submission are omitted, and broad read grants can include private state. |
| INT-0012: mandatory isolation and defined unavailable outcome | Partial. Parent setup failure returns `sandbox_unavailable`; child `pre_exec` setup failures become generic `spawn_failed`. The child rejects `NotEnforced` but accepts `PartiallyEnforced`. Tests allow an unavailable sandbox to count as a passing enforcement test and do not force failure of either primitive. |
| INT-0012: normal in-workspace command | Present; `sandbox_allows_in_workspace_work` tests an in-workspace read and command-tool tests exercise additional work. Cross-directory rename is restricted by ABI v1 despite the intended workspace read/write grant. |
| INT-0012: retain tree kill, bounds, scrubbed environment, no private state path, Linux-only gating | Linux gating and environment scrubbing are implemented. Bounds and lifecycle are partial. The no-private-state-path claim is contradicted by the grant construction. Extra inherited descriptors are not closed explicitly and have no dedicated regression test. |
| INT-0010: exclusive writer, release/fair handoff, presence, TTL reclaim, cross-session matrix, journalled waits | All absent, consistent with `proposed`. `WorkspaceWriter` is shared by `Arc`, with no per-resource arbitration or durable lease registry. The global tool semaphore limits count, not overlapping writers. No lease events or overlapping-writer matrix exist. |
| INT-0019: AppContainer/LPAC + Job Object; mandatory failure; in-workspace success; existing guarantees | AppContainer/LPAC and mandatory refusal are absent, consistent with `proposed`. Windows currently has the command-group Job Object, argv execution and environment scrubbing, with command tests; it has no file/network enforcement tests or profile-creation code. Linux fixes must not be represented as Windows parity. |

## Prioritized concrete repairs

### P1: handle filesystem truncation as a denied operation

`src/tools.rs:1183` selects `ABI::V1`. That omits `AccessFs::Truncate`, which
arrived in ABI v3. A same-UID program can invoke `truncate(path, 0)` on an
otherwise writable out-of-workspace file without opening it for write. The
filesystem ruleset does not deny an unhandled operation and the seccomp filter
does not block it. Thus conventional read-denial tests do not prove the intended
filesystem boundary. The kernel explicitly distinguishes truncation from write
access. [Linux Landlock truncation contract](https://docs.kernel.org/userspace-api/landlock.html#truncating-files).

Require a sufficient ABI with hard compatibility and `FullyEnforced`, handle
Truncate and Refer, and grant those rights only inside the workspace. Add an
actual outside-workspace truncate probe which verifies the marker content
remains unchanged, an inside-workspace write/truncate probe, and a
cross-directory workspace rename probe. Exercise unavailable/partial enforcement
as refusal, separately from the supporting-kernel enforcement suite.

### P1: narrow executable/system read grants and exclude private roots

`SYSTEM_RX` at `src/tools.rs:1129` includes all of `/etc` and `/proc`;
`build_ruleset` adds the entire parent directory of the resolved executable.
`Config::parse` only rejects overlap between workspace and state, and config
inside workspace (`src/config.rs:615`); it does not reject state/config in any
of these additional read-granted subtrees. An operator-provided binary in a
directory with private state therefore grants the command access to that state.
The helper receives no private root with which to enforce the stated exclusion.

Grant the executable file itself, retain only necessary runtime files/system
libraries, remove the broad `/proc` grant, and fail validation when private state
or credential/config material overlaps any remaining broad read grant. Test a
synthetic secret sibling of a non-system executable and an overlap-rejection
case. Test `/proc` aliases and a deliberately inherited private descriptor using
synthetic data. Existing Rust/SQLite descriptors may already use CLOEXEC; this
audit did not establish an actual inherited application descriptor leak and
does not claim one. Kernel rules do not retroactively restrict open descriptors.
[Landlock descriptor behavior](https://docs.kernel.org/userspace-api/landlock.html#rights-associated-with-file-descriptors).

### P1: close alternate network submission paths

`network_syscalls` at `src/tools.rs:1131` denies nine ordinary socket calls but
leaves `io_uring_setup`, `io_uring_enter`, and `io_uring_register` available.
io_uring provides socket creation and connection operations through submission
queues. The inference from those APIs and this default-allow filter is that
direct-syscall filtering does not establish the promised universal socket
denial; no exploit was executed in this review. Deny the io_uring entry points
for this tier and add a Linux probe proving refusal. Also review remaining
socket operations and inherited sockets as part of the same defined boundary.
[liburing socket operation](https://github.com/axboe/liburing/blob/master/man/io_uring_prep_socket.3),
[kernel seccomp filter interface](https://docs.kernel.org/userspace-api/seccomp_filter.html).

### P1: retain and terminate descendants on every command completion path

`src/tools.rs:1071` kills only in timeout/cancellation branches. Successful
`wait()` is followed by up to two five-second reader waits; a lingering reader
is aborted, with no subsequent group kill. In installed `command-group` 5.0.1,
the Unix waiter awaits the leader and then `waitpid(-pgid)`, which cannot reap
grandchildren reparented outside the harness (there is no subreaper setup).
The Windows waiter returns after any completion-port notification rather than
checking an active-process-zero notification. The builder defaults
`kill_on_drop` to false. A successful parent that starts the existing fixture's
`--spawn-grandchild` mode and exits can therefore outlive the recorded effect
and continue modifying its workspace after the run settles.

Use explicit group/job ownership, kill remaining descendants when the leader
finishes or the future is dropped, and reap safely. Avoid cancelling a
dependency group-wait future with a live blocking wait. Preserve the leader's
actual exit status while ensuring descendants are settled. Check cancellation
and zero budget before spawn (`execute` currently spawns first). Add parent-exit
and already-cancelled marker tests on Windows and Linux, plus cancellation with
an observed live descendant. Test dropped-future cleanup where the API permits
future cancellation. Do not claim arbitrary hostile process-tree confinement:
Linux `setsid`/`setpgid` remain allowed and can escape a process group. Preventing
that requires a separate confinement decision or stricter process creation
path; simply enabling the dependency's kill-on-drop is insufficient on Unix.

### P2: bound the serialized command result, not only raw captures

`src/tools.rs:1066` caps raw stdout/stderr. `command_result` at line 1304 adds
JSON fields and converts bytes lossily to UTF-8, then `ToolResult::encoded`
serializes that JSON string again. Even ordinary ASCII output at the cap is
larger than `max_tool_result_bytes`; control bytes/quotes and invalid UTF-8 can
expand it further. `src/runner.rs:1221` records the resulting size but does not
enforce the limit before delivering it to the model. Read tools already measure
their full encoded envelope, so command behavior is inconsistent.

Fit stdout/stderr into the fully encoded ToolResult budget, preserving valid
JSON, exit status, and an honest truncation flag. Test `encoded().len()` for the
minimum valid budget, combined streams, escaping, and invalid UTF-8. Do not
truncate serialized JSON blindly. Timeouts/cancellation currently discard all
captured output; decide and document whether bounded diagnostic prefixes should
be retained rather than silently promising that feature.

### P2: give move its documented no-clobber semantics

`WorkspaceWriter::move_file` checks that the destination is absent at
`src/tools.rs:898`, then uses ordinary rename at line 908. Another writer can
create a destination between those operations, and rename can replace it.
This violates the documented never-overwrites behavior independently of the
larger proposed lease system. Use an OS-supported atomic no-replace operation
through capability-scoped paths, with explicit platform handling; test a
deterministically synchronized collision. Do not present a process-local mutex
as completion of cross-session INT-0010.

`edit_file` also reads a snapshot and later replaces it without checking for
intervening changes. `atomic_replace` avoids partial byte visibility but offers
no stale-read conflict detection, and does not sync data/directory metadata for
power-loss durability. Those are distinct contracts for a workspace correctness
intent; the current curated-tree limitation is documented in `security.md`.

## Documentation and follow-on planning

The security guide still calls command execution deferred
(`docs/security.md:135`) and the tool guide still lists seven tools without
`run_command` (`docs/loop-and-tools.md:136`). Refresh these descriptions and
state platform-specific isolation honestly. Preserve terminal intent history;
add a follow-on repair intent for INT-0003/0012, and an explicit workspace
mutation-correctness intent (atomic no-clobber and stale-input behavior) alongside
INT-0010's larger lease system. Keep INT-0019 proposed until Windows-specific
enforcement exists and is actually tested.

This research identifies necessary repairs and test targets. It does not certify
that a new sandbox policy is correct, claim new Linux/Windows test passes, or
mark proposed coordination and Windows isolation as completed.
