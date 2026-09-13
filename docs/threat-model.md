# Threat model and security assurance

- **Version:** 3 (2026-09-12).
- **Owner:** maintainers. **Review cadence:** every security-affecting sprint, dependency/target change and release.
- **Current intent:** [INT-0021](intents/INT-0021-harness-contract-review.md).
- **Historical evidence:** [INT-0015](intents/INT-0015-threat-model-assurance.md) and [sprint 8](sprints/s8/sprint-tests/test-report.md) retain version 1's claims and executed checks. Sprint 10 supersedes that assurance revision; it does not rewrite its measurements.
- **Version 3 scope:** interactive personal setup, private-directory provisioning, private creation modes for Unix journal/export/backup files, and the explicitly granted `create_directory` tool. Earlier sprint evidence remains historical; [interactive repair verification](interactive-entry-repair.md) and the named new tests below exercise these additions.
- **Boundary details:** [security](security.md), [dependency policy](supply-chain.md), [sprint 10 audit](sprints/s10/sprint-research/assurance-audit.md).

A mechanism is an implemented gate with a named proving case. A gap is an
unimplemented outcome owned by an intent. A residual is a boundary accepted for
the current deployment. A test reference identifies what is exercised; the
sprint's test report records which revision/platform was actually executed.
This package is neither a certification nor a proof that a model resists all
prompt injection.

## Trust and deployment boundaries

Operator configuration grants authority. Model responses, workspace contents
and MCP descriptions/results do not enlarge that grant. They can still
influence what a model asks to do **within** the grant. Checked acceptance uses
a frozen task contract and independent observed-byte checker; a freeform coding
result stays unchecked.

Interactive personal setup saves the selected model and tool grants outside the
chosen working folder. Its default grant includes read/search, directory
creation, and file write/edit operations; it excludes delete/move, commands, and
MCP servers. The session displays the effective folder and capabilities before
the first task. Existing profiles retain their grants, and an explicit `--config`
bypasses this setup. Selecting another folder changes that session's workspace
root without rewriting the saved profile. These are operator-level grants;
model text cannot enlarge them and per-action approval remains INT-0017.

New personal directories are created with private Unix modes or protected
Windows DACLs; existing permissions are checked without alteration. Startup
rejects overlap between tool roots and its configuration/private state before
saving settings. Creating the settings file uses no-replace semantics. Ancestor
directories and other processes with the operator's full OS authority remain
trusted; this provisioning is not a sandbox against that account.

The service authenticates owners over loopback and scopes every run query and
action to its verified principal. Moving model inference to another host does
not expose Kinesin's API. Non-loopback API exposure remains gated on an explicit
mutual-identity/transport-authentication deployment decision and evidence.
Client model-origin checks require HTTPS outside loopback; actual server
binding and tunnel ownership need deployment evidence beyond those URL checks.
A [scoped two-host validation](sprints/s10/sprint-tests/remote-deployment.md) was
executed in sprint 10; broader deployment coverage remains
[INT-0027](intents/INT-0027-encrypted-remote-deployment.md).

Local MCP executables are operator-trusted native programs. Their protocol data
is untrusted, and per-run tool grants still apply; bounded stdio and process
ownership do not make the program itself a sandboxed adversary. Trusted servers
must not deliberately escape their owned process group. Linux command isolation
and Windows process Jobs are different controls; Windows filesystem/network
sandboxing remains [INT-0019](intents/INT-0019-windows-command-sandboxing.md).

The Linux tier confines selected filesystem operations and denies selected
network, asynchronous-I/O and group/namespace-escape syscalls. It is not complete
isolation from processes or metadata owned by the same OS user. Landlock does
not cover every metadata operation, including permission/owner changes and
extended attributes, and this policy does not enable its newer signal-scoping
facility. Existing OS permissions still apply; the harness makes no blanket
same-user process-isolation guarantee. These limits are separate from tested
file-content and socket denial. See the kernel's
[filesystem flags and limits](https://docs.kernel.org/userspace-api/landlock.html#filesystem-flags).

## OWASP LLM 2025 mapping

These ten IDs explicitly retain the version named by the original intent. The
[official LLM 2025 index](https://genai.owasp.org/llm-top-10/) defines the taxonomy;
the dispositions below are Kinesin-specific, not claims made by OWASP.

| Risk ID / surface | Disposition | Mechanism, evidence and remaining boundary |
|---|---|---|
| LLM01 — Instruction injection | Mechanism + residual | Immutable `RunAuthority` and dispatch allow-lists deny ungranted effects. `redteam_denies_unauthorized`, `run_command_denied_without_grant`, and MCP ungranted-tool tests exercise gates. Content may redirect a model among granted effects; scripted cards are not a live-model resistance measurement. |
| LLM02 — Sensitive data exposure | Mechanism + gap | Owner-scoped service/storage, metadata capture and explicit private export restrict retrieval. `owner_scope_covers_all_routes_and_body_cannot_claim_identity` and `metadata_rejects_private_observations_but_replay_retains_them_once` cover those boundaries. Final candidates can be sensitive; full credential/egress stewardship remains INT-0025. |
| LLM03 — Dependency and model provenance | Mechanism + gap | Blocking deny/audit plus reviewed lockfile; see the dependency policy and its negative duplicate proof. Native programs/models are still operator trust inputs. Signed release artifacts and model provenance controls are not supplied by an advisory check. |
| LLM04 — Training/data corruption | Residual + mechanism | Kinesin does not train or fine-tune models; upstream model integrity is an operator dependency. A checked verdict is limited to the frozen contract and observed source, not whether the source/model is truthful. `scripted_task_cards` exercises ambiguous/missing/truncated fixtures that constrain acceptance. |
| LLM05 — Unsafe interpretation of output | Mechanism | Typed tool arguments, capability paths and argv execution prevent shell-string interpretation. `redteam_treats_content_as_data`, command-tool tests and capability escape tests exercise these restrictions. MCP output mints no checker evidence. |
| LLM06 — Excessive autonomous authority | Mechanism + gap | Per-run/workspace grants, resource ceilings and checked-task mutation prohibition narrow authority. `checked_workspace_cannot_grant_run_command` covers the latter. Per-action approval and just-in-time authority remain INT-0017; a broad operator grant is still broad. |
| LLM07 — System instruction disclosure | Residual | Instructions are private task content but not a credential store. The model may repeat its own instructions to an authorized caller; no model-level secrecy guarantee is made. Metadata capture avoids retaining intermediate instructions; replay deliberately retains them. |
| LLM08 — Retrieval/vector weaknesses | Not implemented | No embedding index or vector retrieval exists. Ordinary file/MCP observations retain their own trust boundaries; future retrieval must receive a separate assessment. |
| LLM09 — Incorrect answers | Mechanism + residual | Checked tasks require independent evidence and frozen criteria; freeform results remain unchecked. `scripted_task_cards`, settlement and replay tests cover contract failures. Recorded live task-quality failures remain in [live evaluation](live-evaluation.md); general coding competence is INT-0024. |
| LLM10 — Unbounded resource consumption | Mechanism + residual | Controller/run/tool/model/storage bounds, ingress limits and bounded MCP framing/discovery constrain accepted work. Scheduler, `service_load`, `adversarial_runtime` and MCP limit tests cover these gates. Admission headroom is not a physical-disk quota, and trusted native server resource use is not a container limit. |

## OWASP Agentic 2026 mapping

The [official Agentic 2026 publication](https://genai.owasp.org/download/52117/?tmstv=1765059207)
defines ASI01–ASI10. Each dimension has its own disposition, including features
that are absent.

| Risk ID / surface | Disposition | Mechanism, evidence and remaining boundary |
|---|---|---|
| ASI01 — Goals redirected by input | Mechanism + residual | Frozen authority and checked criteria survive hostile content; command/MCP ungranted-tool tests prove the gate. The model may still pursue an undesirable permitted action, so this is not complete goal-integrity protection. |
| ASI02 — Tool misuse | Mechanism + gap | Typed dispatch, capability paths, bounds and checked-run mutation bars constrain built-in and MCP calls. `runner_tools`, `command_tool` and `mcp` suites cover them. Required-key MCP validation is not full JSON Schema validation; descriptions never authorize tools. |
| ASI03 — Identity/privilege misuse | Mechanism + gap | Authentication precedes request decoding; owner/run predicates and current owner policy constrain every route. `every_route_authenticates_before_body_or_storage_and_rejects_ambiguous_credentials` and owner-scope tests apply. Per-action approval is INT-0017; remote delegated MCP credentials are INT-0020. |
| ASI04 — Agent dependency compromise | Mechanism + residual | Dependency policy and frozen MCP definitions limit known supply-chain/protocol risks. Operator-approved executable identity is a trust decision, not package attestation or sandboxing. Release signing and comprehensive plugin/model provenance remain open. |
| ASI05 — Unintended native execution | Mechanism + gap | Commands use an argv allow-list, scrubbed environment and owned processes; Linux additionally requires Landlock/seccomp. `command_tool` and `sandbox_linux` exercise native effects. Windows Jobs control lifetime, not filesystem/network access; INT-0019 owns that gap. |
| ASI06 — Corrupted memory/context | Mechanism + gap | Immutable run inputs, whole-group compaction and checked evidence constrain within-run state. Replay/compaction tests apply. Persistent skills and sessions remain INT-0006/0026; signed journal authenticity remains INT-0014. |
| ASI07 — Untrusted messages between agents | Not implemented + gap | Model-spawned subagents and an inter-agent messaging protocol are absent. Independent concurrent runs retain owner scope. INT-0018 must add narrowed child authority, correlation and immutable result provenance before introducing this channel. MCP tool observations are not authenticated peer-agent assertions. |
| ASI08 — Failures spreading across work | Mechanism + gap | Per-owner quotas, independent cancellation and retained controller/writer ownership bound the impact of failure. `writer_failure_during_two_active_effects_rejects_more_work_and_owned_shutdown_joins`, service cancellation and process-recovery tests apply. Parent/child failure propagation remains INT-0018. |
| ASI09 — Manipulating operator trust | Mechanism + residual | CLI/API distinguish execution completion from task acceptance; receipts are scoped and freeform answers unchecked. CLI inspect/settlement tests prove output contracts. Output can still persuade an operator; per-action confirmation UX and prompt-injection behavior are not proven by literal-argv tests. |
| ASI10 — Agents exceeding their assigned role | Mechanism + gap | Immutable grants, finite run limits and cancellation contain model-selected work at enforced gates. Scheduler/adversarial tests exercise bounds. Model-spawned subagents, escalation and approvals require INT-0018/0017; a compromised trusted controller is outside the live authorization guarantee. |

## Project control mapping

The following are the specific identity, least-privilege and oversight controls
selected by INT-0015. This is a project crosswalk, not an assertion of conformance
to an issued NIST certification or a complete normative NIST agent standard.

| Control | Current disposition |
|---|---|
| Policy authorization | Config-derived immutable authority and both MCP gates; discovery/content cannot enlarge permissions. |
| Scoped tasks / least privilege | Owner/workspace/model/tool aliases and bounded contracts; read, mutation and command classes remain distinct. |
| Workspace mutation concurrency | Move publishes its destination with a capability-scoped no-replace hard link, then removes the source. A failed source cleanup is reported explicitly. The two-name operation is not atomic, and namespace/source races or stale writers still require INT-0010's lease/fencing work. |
| Directory creation | `create_directory` opens each existing parent without following links, then creates one leaf relative to the retained capability. Existing entries are not replaced and missing parents are not implicitly created. The tool requires a write grant, is barred from checked tasks, and mints no evidence. This does not add leases or fencing for other mutating tools. |
| Just-in-time privilege | Gap: INT-0017. Current grants last for the run. |
| Human approval | Gap: INT-0017. Existing operator configuration is not a per-effect approval protocol. |
| Dev/test/prod separation | Operator responsibility. Use separate credentials, models, workspace roots and private state; the runtime does not isolate multiple environments sharing a controller. |
| Service identity / transport | Bearer-over-loopback with provisioning, expiry/revocation and verifier reload is implemented. Non-loopback API exposure needs separately reviewed mutual identity/TLS; INT-0027 concerns model deployment, not API ingress. |

## Memory safety and native interfaces

Kinesin's decision, policy, model, runner, verification, scheduler and journal
logic use safe Rust. Native dependencies and localized OS calls remain in the
trusted computing base; Rust and rustls do not make the entire process free of
memory-unsafe code. The inventory groups unsafe blocks by owning function and
OS operation so each boundary has a review owner.

| First-party surface | Justification and containment |
|---|---|
| `private_state.rs`: Windows `LocalAllocation::drop`, `Policy::current`, `current_sid`, `sid_string`, `descriptor_owner`, `inspect` | Native ownership/ACL/token queries and security-descriptor parsing. LocalFree-owned allocations and OwnedHandle manage lifetimes; SID/descriptor/ACE lengths and types are checked before interpretation. These checks are an FFI boundary, not a claim of formal memory-safety proof. |
| `private_state.rs`: Unix `Policy::current` | `libc::geteuid` reads the effective identity for mode/owner validation. |
| `onboarding.rs`: Windows `Allocation::drop`, `create_private_directory`, `current_sid` | Queries the current token SID and converts a fixed-shape protected DACL before `CreateDirectoryW` creates a new personal directory. SID buffer lengths are bounded; aligned token storage, OwnedHandle, and LocalFree-owned allocations delimit the FFI lifetimes. Existing directories and settings files are not replaced. |
| `signal.rs`: `ctrl_c` on Windows | Clears the inherited Ctrl+C-ignore attribute after listener registration. No workspace/model bytes are interpreted by the OS callback. |
| `tools.rs`: Linux sandbox `arm` / child `pre_exec` | The parent prepares the Landlock ruleset and seccomp program; child setup applies the policy and marks non-stdio descriptors close-on-exec before executing the selected command. Unavailable enforcement is refused. New work in the forked child must remain async-signal-safe. |
| `process.rs`: Unix `OwnedProcess::peek_leader_status`, `terminate`; Windows `Job::new`, `assign_and_resume`, `terminate`, `wait_empty` | `waitid(WNOWAIT)` observes the owned leader through an initialized `siginfo_t` without consuming its status; retaining that PID prevents unrelated process-group reuse before `killpg`. Tokio reaps the direct child after group termination. Windows creates a kill-on-close Job, assigns the suspended child, resumes its thread, terminates the Job and queries active members; native handles are OwnedHandle-managed. Group/job termination stops descendant effects. Unix orphan-zombie reaping belongs to the OS; no subreaper guarantee is made. |

The native dependency inventory in [supply-chain.md](supply-chain.md) covers
SQLite through `libsqlite3-sys`, AWS-LC through `aws-lc-sys`, platform FFI and
target-specific alternatives. Those libraries require their own maintenance,
advisory monitoring and source review. Command/MCP executables are additional
native programs outside Rust's type system.

Test-only unsafe includes Windows ACL/directory fixtures in `private_state.rs`
and `storage.rs`, console-signal injection in `tests/windows_signal.rs`, and
fixture PATH environment setup in command/red-team/runner/sandbox/CLI test
binaries, and deliberate syscall/descriptor probes in command/sandbox fixtures.
The `CreateDirectoryW` block in `storage.rs` is a test fixture; production
personal-directory provisioning is in `onboarding.rs` as listed above. Tests must not present
an unsafe environment update as evidence of runtime credential isolation.

Maintain this inventory with `rg -n 'unsafe|extern' src tests examples`, inspect
the surrounding cfg/function, and recompute the target-aware native graph after
changes. Pre-opened descriptors/handles are a separate capability: pathname
confinement cannot revoke them. Linux command setup marks descriptors above
stdio close-on-exec; `sandbox_closes_inherited_non_stdio_descriptors` checks a
test-owned descriptor deliberately made inheritable. This is a synthetic probe,
not evidence that a private application descriptor previously leaked. The
comprehensive embedding inventory, Windows handle policy and credential/egress
stewardship remain proposed in INT-0025.

## Release-evidence map and empirical limits

| [security.md](security.md#evidence-required-before-each-release) row | Proving cases/suites |
|---|---|
| Tool authority | `redteam_denies_unauthorized`, `command_tool`, `sandbox_linux`, `runner_tools`, `mcp_unapproved_call_denied`, `mcp_startup_capture_rejects_removed_altered_and_ungranted_schemas`. Linux probes cover truncation, executable siblings/proc, io_uring, namespaces, inherited descriptors and `sandbox_denies_x32_syscalls_before_kernel_abi_dispatch`; the production BPF interpreter test does not depend on kernel x32 support. |
| Directory creation | `create_directory_requires_a_grant_and_records_its_real_effect`, `create_directory_never_follows_parent_or_leaf_links`, `create_directory_refuses_escapes_missing_parents_and_read_only_capability`, and `create_directory_preserves_spaced_names_and_existing_contents` exercise actual effects and refusals. `directory_creation_capture_replays_without_repeating_the_mutation` verifies replay creates no new directory. |
| Untrusted content | `redteam_treats_content_as_data` proves literal argv; `mcp_poison_description_and_result_are_data` proves no grant expansion; `mcp_protocol_and_discovery_are_bounded` and `mcp_tiny_frame_flood_is_refused_and_all_descendants_settle` exercise byte/count/deadline limits; `scripted_task_cards` proves deterministic checker behavior. |
| Task acceptance | `runner_tools`, `settlement`, `replay`: invalid/forged candidates, wrong facts and cross-owner/run evidence cannot manufacture acceptance. |
| Data handling | `metadata_rejects_private_observations_but_replay_retains_them_once`, `cli_inspect`, owner-scoped export, `mcp_process_environment_and_stderr_are_scrubbed`, and Linux `command_runtime_grants_cannot_cover_credential_verifiers`. |
| Personal setup | `first_use_creates_private_settings_and_never_changes_workspace_contents`, `existing_settings_preserve_model_tools_and_bytes_when_selecting_another_root`, `overlapping_workspace_and_invalid_server_never_save_settings`, and `settings_write_does_not_clobber_a_competing_first_run` cover provisioning and preservation. `permissive_existing_directory_is_rejected_without_changing_permissions` is Unix-specific; `platform_paths_are_independent_of_current_project_and_ignore_relative_xdg` covers path selection. |
| Concurrent runs | Scheduler tests, `adversarial_runtime`, `service_load`: ownership, quotas, storage failure and bounded settlement. `mcp_controller_capacity_queue_cancel_and_idempotency_never_start_extra_processes`, `mcp_preparation_cancel_deadline_and_pre_cancel_settle_and_replay_without_model`, and observed-marker shutdown tests cover MCP ownership. `leader_observation_retains_pid_until_group_cleanup` checks Unix PID retention. |
| Shared owners | `owner_scope_covers_all_routes_and_body_cannot_claim_identity`; authority aliases remain owner constrained. |
| Authentication | `every_route_authenticates_before_body_or_storage_and_rejects_ambiguous_credentials`, auth unit tests and verifier reload/readiness tests. |
| Load and storage | `process_recovery`, `settlement`, `service_load`, retention/backup and `mcp_journal_error_awaits_cleanup_and_recovery_retains_interrupted_admission`: defined terminal results and unknown effects after interruption. |

Normal CI executes the offline/scripted suites on Windows and Linux.
`pinned_live_task_cards` and the live cache/deployment tests are opt-in,
environment-dependent measurements and are not executed by ordinary CI.
A failed or unavailable live measurement remains recorded as such. Actual
session cache reuse and one two-host checked deployment are recorded in the
[remote measurements](sprints/s10/sprint-tests/remote-deployment.md); causal flag
speedup, full continuity and concurrent slots remain INT-0026. A same-host LAN
response is not two-host encrypted deployment proof. The [sprint 10 evidence](sprints/s10/sprint-meta.md)
must link each planned repair check to its actual test/procedure and platform
result before making a completion claim.
