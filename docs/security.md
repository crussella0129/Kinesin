# Security and authority

Kinesin's first complete release runs several agents for one owner. Its later
shared service accepts authenticated users on one controller host. Security,
latency, scalability, and a small understandable implementation are design goals
at both stages. This document defines the security claims each stage must earn;
the [build guide](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md) supplies the implementation sequence.

## Three trust levels

| Level | Supported workload | Boundary and limits |
|-------|--------------------|---------------------|
| Learning CLI | One owner; synthetic inputs; trusted compiled code | Validate all external data, restrict tools, and keep work bounded. |
| Concurrent personal harness | Several runs for the same owner; untrusted model replies and file contents | Each run owns its state and authority; only bounded infrastructure is shared. This is the first complete release. |
| Shared service | Authenticated owners; provisioned workspaces; trusted compiled read-only tools | Authorize every operation by owner and enforce shared/per-owner quotas. One controller process remains trusted with service data. |

The first shared service does not execute uploaded programs, arbitrary shell
commands, or untrusted native plugins. Protection from a compromised controller,
a hostile host administrator, or another process with the same OS account's full
authority is outside these claims. Supporting hostile executable workloads is a
separate deployment milestone below.

## A run receives authority; the model receives descriptions

Trusted startup constructs an immutable `RunAuthority`: owner, workspace
capability, model profile, permitted tools, budgets, capture policy, and frozen
`TaskContract`. CLI
startup supplies the local owner; service startup derives ownership from verified
credentials. A request may choose permitted aliases, lower its budgets, and
request private capture only if owner policy allows it. It cannot grant itself
another workspace, destination, credential, or larger limit.

`core` proposes an action. The per-run `runner` validates and authorizes it before
the effect. `WorkspaceReader` holds the filesystem capability; `ModelClient`
holds the selected transport configuration. Model text, tool output, and tool
schemas never grant permission. A valid JSON object can still request a forbidden
operation. See [architecture](architecture.md) and [tool contracts](loop-and-tools.md).

Reading is an information transfer. A file result enters the conversation and
may be sent to the configured model host. Selecting a workspace therefore means
allowing its exposed contents to reach that destination. Read-only tools protect
against writes; they do not make secrets safe to expose. Keep the chosen model
destination visible in the run summary, including for remote inference.

## Task acceptance has its own trust boundary

The [acceptance contract](verification.md) separates completed execution from a
passed task. Operator-controlled configuration defines task aliases and their
checker/specification versions. Admission authorizes the alias, its fixed
workspace, its required tools, and the selected model for the verified owner.
Keep this configuration outside tool roots. Freeze the effective contract before
running; model text, workspace content, and client-supplied claims cannot revise it.

Freeform and checked submissions are disjoint. Freeform accepts a bounded prompt
and remains unchecked. A checked task selects an allowed task alias and model;
the profile fixes the workspace, instructions, criteria, and output shape. Reject
extra prompts, instructions, criteria, expected answers, or workspace overrides.
Reject empty/duplicate criteria and unknown checkers before admission. Freeform
permission does not satisfy a policy requiring checked tasks.

Only the runner creates evidence records from actual tool observations, bound to
owner, run, effect, resource, observed content, status, and completeness. A
model-provided evidence ID only refers to that run's inventory; it cannot create
evidence or retrieve another owner's or an earlier run's records. Successful
reading alone does not validate an answer: the checker independently compares
the submitted field with its required source. Failed, denied, truncated, forged,
or wrong-source evidence cannot establish a complete-file claim.

**Only a complete successful read mints evidence.** `list_files` and
`search_files` return partial views of the workspace and deliberately carry no
evidence reference, even when the runner has one available. A search can tell a
run where a value lives; it can never certify what that value is, because it
returns matched lines rather than the observed file. Without this boundary a
candidate could cite a search hit as proof of a field it never observed
completely. Adding a further read-only tool does not change the rule: a new tool
mints evidence only if it returns a complete successful observation of the
resource a criterion names.

The first checker is compiled, pure, and bounded. Give it read-only typed inputs,
with no filesystem, network, database, model, or code-execution capabilities.
It neither follows source instructions nor accepts a model's `passed` field.
Unknown, missing, errored, or skipped required checks never become passes. The
controller derives `task_accepted` only from committed `completed + passed`.

A pass certifies the identified extraction contract over observed source bytes;
it does not certify the source's truth, the model's reasoning, or the filesystem
at completion. Do not reread changing files to manufacture verification evidence.
Receipts and evidence descriptors remain bounded and owner-scoped. Diagnostics
must not expose hidden expected values or unrelated contents. Metadata capture
retains the receipt and final candidate, including rejected candidates, but no
additional evidence bodies. Replay requires explicit capture permission and the
actual frozen inputs; a stored verdict alone is not reproducible proof.

## Write tools change the trust model

`list_files`, `read_file`, and `search_files` only observe the workspace. Directory
creation and file mutation cross the read-only boundary and require their own
tool grants.

**Consent is the operator's, at configuration time.** A workspace grants writes
only when its operator lists a write tool in that workspace's `tools`. The model
proposing a write cannot grant it permission. Human-session setup creates a
personal profile permitting listing, reading, searching, directory creation,
writing, and editing in the selected working folder. The session displays that
folder and its capabilities before accepting a task. Existing personal profiles
retain their grants; `--config` selects an explicit operator profile. Setup does
not grant deletion, moves, commands, or MCP servers. These are run-level grants,
not a per-effect approval protocol.

**The capability is separate by construction.** Writes run through a
`WorkspaceWriter` distinct from `WorkspaceReader`; the read tools have no method
that can change a file. A writer is built only for a workspace that granted a
write tool, so a run without that grant has no writer to reach, and an
unauthorized write is denied before any handler runs and is still journalled.

**`write_file` is bounded and structure-preserving.** It stays inside the
capability root, refuses a symbolic-link target or a directory,
does not create parent directories, and replaces the whole file atomically
through a temporary sibling and a rename, so a crash leaves either the old file
or the new one. Content is bounded and mints no evidence.

**`create_directory` creates one new directory, without replacing any entry.**
Its bounded, workspace-relative path may include spaces. Every existing parent
component is opened through `cap-fs-ext`'s `open_dir_nofollow`, retaining a directory
capability before resolving the next component; the final creation is relative
to that handle. Traversal, absolute paths, reserved device/stream syntax, and
symbolic-link parents are refused. Missing parents are not created implicitly.
An existing file, directory, or link produces an error and is left unchanged.
Directory creation mints no evidence. Replay consumes its recorded observation
without creating the directory again.

**A checked task cannot write.** Acceptance depends on observing files the run did
not author. A run that could edit a source and then read it back would certify its
own change, so authorization refuses a write tool in a checked task's workspace.

**`edit_file` uses the same write capability.** It replaces
one exact passage in an existing file and requires the passage to be **unique**:
an absent match cannot edit and an ambiguous one is refused, so the change never
lands in the wrong place. It runs through the same `WorkspaceWriter`, the same
symlink and directory guards, the same atomic replace, and the same bar on write
tools in checked runs.

**`delete_file` and `move_file` also require write grants.**
Delete removes only a regular file, refusing a directory or a symbolic link so it
cannot remove curated structure or follow a link outside the root; a missing file
is an error. Move publishes a hard link with atomic no-replace semantics and
then removes the source. An existing or concurrently created destination is
preserved. Hard links must be supported on the same filesystem. If source
cleanup fails, the explicit `move_source_cleanup_failed` outcome reports
`destination_created = true` and `source_cleanup = failed`. This is not an atomic
two-name rename; concurrent source replacement and stale edits remain part of
[INT-0010](intents/INT-0010-cross-agent-write-coordination.md). Both run through the same
`WorkspaceWriter` and the same checked-run bar.

**Command execution is an explicit freeform capability.** `run_command` requires
an operator allow-list of executable names and uses separate argv elements,
without invoking a shell to interpret them. Commands receive a scrubbed
environment. Encoded stdout/stderr, execution time and tool concurrency are
bounded. Pre-cancelled calls never spawn. Normal completion, timeout and
cancellation terminate the owned group/Windows Job and await the direct child;
drop supplies a kill fallback. Unix orphan zombie reaping belongs to the OS.
This lifecycle ownership does not itself confine a process's filesystem or
network access; the platform boundaries below apply.

## Filesystem tools from their first implementation

Use `cap-std` when the first file tools arrive. Open the trusted root once with
`Dir::open_ambient_dir`, then retain that `Dir` inside a private `WorkspaceReader`.
Expose only bounded listing and reading methods. Do not return the inner `Dir`
or use ambient `std::fs` calls on model-supplied paths. A `Dir` has mutation APIs;
the wrapper deliberately narrows what Kinesin's handlers can do.

The capability library protects path resolution against traversal and escaping
symlinks. It does not sandbox arbitrary Rust code that can call other APIs.
Avoid replacing it with canonicalize/check/open: separate checks and opens can
race. [cap-std's security model](https://github.com/bytecodealliance/cap-std),
[Rust filesystem race guidance](https://doc.rust-lang.org/std/fs/index.html#time-of-check-to-time-of-use-toctou)

Keep a small lexical validator for clear errors: bounded path length; no NUL,
parent traversal, rooted path, Windows prefix, or `:`. The colon rule also excludes
Windows alternate data stream syntax. Allow `.` for listing the root. Never
silently fall back to unrestricted access when capability operations fail.

Open relative to the capability, inspect metadata on the opened file, require a
regular file, and read a bounded prefix. For stable curated input trees, an early
type check can reject special files before opening; it does not replace the
opened-handle check. Restrict this release to ordinary local filesystems with no
attacker concurrently replacing files with devices or FIFOs. A post-open check
alone cannot prevent an open from blocking. The later worker isolation boundary
must address adversarial filesystem objects and enforce resource limits.

Directory authority does not classify data. A secret, hard link, or mounted
resource intentionally exposed within the tree can still disclose its contents.
Provision small input directories. Keep configuration containing credentials,
service state, databases, backups, and replay artifacts outside every tool root.
Validate this separation at trusted startup; use private OS permissions too.

Test on Windows and Linux, including symlinks/junctions, reserved device names,
absolute and drive-relative paths, and alternate streams. Supported test cases
must pass; explicitly report unavailable platform facilities. Use the library's
normal root constructor instead of adapting an arbitrary Windows handle:
`Dir::from_std_file` has a documented Windows sharing-mode precondition.
[Dir API](https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html)

Pin a maintained release and review advisories during dependency updates. A
previous Windows device-name bypass was fixed in 3.4.1; old tutorial pins are not
a security baseline. [cap-std advisory](https://github.com/bytecodealliance/cap-std/security/advisories/GHSA-hxf5-99xg-86hw)

## Network and credential boundaries

The model profile fixes the destination. Model output cannot edit URLs, request
headers, TLS settings, or proxy configuration. The first local profile uses a
loopback listener. Remote profiles name an operator-approved destination; network
access and transport authentication are configured separately from tool policy.

Configure `reqwest` explicitly: no redirects, no inherited/system proxy, no
automatic retries, a connection timeout, and a total exchange deadline including
body consumption. Bound headers, decoded response bytes, and outgoing bytes.
Keep certificate verification enabled for HTTPS. Current client defaults include
redirect following and protocol-NACK retries, so omitting retry code is not enough.
[reqwest ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html)

Resolve credential references in the trusted adapter. Do not serialize their
values with `RunAuthority`, configuration snapshots, `Debug`, events, or errors.
A secret wrapper can reduce accidental formatting; it does not make a process
containing secrets inaccessible to its owner or administrator.

WireGuard can provide the private route to inference. It does not establish
which service user may run a task or inspect its results. Keep the model API
inaccessible to untrusted clients that could bypass the harness's admission and
accounting. Any future URL-fetching tool gets its own destination/egress policy;
it does not inherit the model client's networking authority.

## Records and display

Context organization is not a trust boundary. A tool-readable `AGENTS.md`,
`CONTEXT.md`, reference guide, or previous model result can contain hostile
instructions. Keep it in its data/observation role; do not elevate it through
automatic folder discovery. Trusted instructions come from operator setup and
still cannot override the run's enforced owner, tools, destinations, or limits.

Task recipes and human-reviewed exports must identify their selected inputs.
Freeze initial configured text per run, preserve capture policy, and use a new
run for a revised recipe or edited result. Concurrent runs cannot share a mutable
`stage/output/` as their execution truth. The database remains authoritative.

Default capture is metadata: omit the input prompt and intermediate model/tool
text from the journal. A bounded final answer or rejected answer candidate is
still retained as the
owner-scoped run result so an asynchronous caller can retrieve it after completion
or restart. That answer can contain sensitive information copied from inputs;
metadata capture is not a content-free storage mode.

Private replay capture is an explicit per-run choice that additionally records
initial instructions, prompt, tool definitions, and normalized observations/input
deltas, including frozen checked-task specifications and required evidence.
Do not claim exact replay for missing or redacted required content. Do
not repeat complete history snapshots for every request. On controller restart,
unfinished runs become interrupted; they are not automatically resumed or queued.
The [journal contract](traces.md) owns schemas, transactions, and replay semantics.

Protect the state directory before writing sensitive records: owner-restricted
permissions on Unix, a restricted DACL on Windows. The SQLite database, WAL,
shared-memory sidecar, exports, and backups inherit the same confidentiality
requirement. WAL mode and durable transactions do not provide encryption or
owner-level access control. Apply retention to private payloads and operational
metadata deliberately; do not expose database files through the service API.

Human-session setup provisions new personal settings and state directories
outside the chosen workspace. On Unix, new directories use mode `0700` and the
settings file uses `0600`. On Windows, new directories receive a protected DACL
with inheritable access for the current user, SYSTEM, and Administrators. Existing
trees are checked with the private-state validator; setup does not broaden or
repair existing permissions. The settings file is created with no-replace
semantics after profile validation, so a competing first setup cannot overwrite
another profile. These checks assume trusted, stable ancestor directories and
do not provide encryption or isolation from the same OS account. Explicit
profiles and shared-service deployments retain their operator provisioning
requirements; the interactive setup flow does not run for them.

Never log bearer tokens, authorization headers, private keys, raw environment
dumps, or complete credential-bearing URLs. Metrics use bounded labels such as
outcome/tool/model profile, not prompts or file contents. For terminal display,
preserve ordinary newlines and escape control characters such as ESC; do not let
model text emit terminal control sequences. JSON serialization protects record
syntax but does not sanitize later display. Exported text must remain data.
[OWASP logging guidance](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)

## The shared-service admission gate

Add service access after the concurrent local runner passes its checks. Keep one
Axum controller process and trusted compiled read-only handlers. Users select
only operator-managed task, workspace, and model aliases allowed for their owner;
a checked task fixes its workspace through the profile.

The initial authentication scheme is operator-provisioned high-entropy bearer
tokens per principal. It supplies a concrete small service without building a
password database, login UI, OAuth server, or JWT issuer. Tokens authenticate
clients; the policy layer separately authorizes their operations.

1. Generate a nonsecret token ID plus a 32-byte random secret using the OS-backed
   `getrandom` crate; fail provisioning if randomness fails. Encode the secret
   unambiguously, for example as base64url without padding. Display the complete
   token once through the deliberate provisioning flow and store it securely on
   the client. [getrandom](https://docs.rs/getrandom/latest/getrandom/fn.fill.html)
2. Store the token ID, owner, expiry/revocation state, and SHA-256 digest of the
   secret. Use RustCrypto `sha2`, not a handwritten hash. The stored digest is a
   verifier; the API never accepts it as a token. This design relies on randomly
   generated high-entropy secrets and is not a password-storage recipe.
   [sha2](https://docs.rs/sha2/latest/sha2/)
3. Accept a single bounded `Authorization: Bearer` header. Decode exactly the
   expected secret length, compute its digest, and compare equal-length 32-byte
   digests using `subtle::ConstantTimeEq`. Use uniform authentication errors;
   never return the credential or its digest. Constant-time digest comparison
   does not make all HTTP handling constant-time.
   [subtle comparison](https://docs.rs/subtle/latest/subtle/trait.ConstantTimeEq.html)
4. Require HTTPS at the client-facing ingress, with certificate verification.
   TLS may terminate in a maintained reverse proxy on the controller host; the
   backend binds loopback and cannot be reached directly from the network.
   The application still verifies the bearer token. Do not trust a client-sent
   owner header. Tokens in URL query strings are unsupported.
   [Bearer token transport](https://www.rfc-editor.org/rfc/rfc6750.html#section-5)
5. Support explicit expiry, revocation, and rotation. Revocation rejects new
   authenticated requests; already admitted runs keep their fixed authority
   until completion, cancellation, or deadline. Operator cancellation handles
   active work. Bound stream lifetime, reauthenticate reconnects, and document
   this behavior rather than implying revocation undoes previous effects.

Load verifier records through an operator-controlled path outside tool roots.
Do not put plaintext bearer tokens in `kinesin.toml`, the run journal, shell
command history, examples, or Git. A credential's owner has access only to that
owner's resources; administrative provisioning is a separate operator action.
Move to an established OIDC integration when interactive login, federation, or
central account management becomes a requirement.

Authenticate every protected endpoint and authorize the requested operation on
its owner-scoped resource. Create, list, status, output/event streams, cancel,
replay/export, and any later deletion endpoint all need checks. A run ID is an
identifier, not permission. Filter database queries by owner and run ID together;
return the same not-found response for absent and inaccessible run IDs.
Descriptions or request bodies never supply authoritative ownership.
[OWASP authorization guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html)

The initial protected API is `POST /v1/runs`, `GET /v1/runs`,
`GET /v1/runs/{id}`, `GET /v1/runs/{id}/events`,
`POST /v1/runs/{id}/cancel`, and `GET /v1/runs/{id}/export`.
Export additionally requires permission to retrieve replay payloads. Retention
is an operator action; there is no initial client deletion endpoint. Keep global
metrics private or behind separate administrative authentication.

Scope submission idempotency keys to the verified principal. Store the key,
request digest, and created run atomically; the same key with a different request
is a conflict. This prevents duplicate run creation on HTTP retries. It does not
guarantee exactly-once external tool effects; no automatic resume is offered.

Enforce bounded request bodies and authentication attempts before expensive
processing, then per-owner and global run/queue/model/tool/storage budgets.
Authorize stream attachment before subscribing; cap slow subscribers so they
cannot stall run recording. Partition conversation state, cancellation handles,
artifacts, and any future caches by owner. Shared inference slots are capacity,
not a user-authorization mechanism.

## Future external tools and hostile code

MCP adapts into the same policy gate. Approve server identities and tool names;
bound discovery and results. Configuration attaches only server declarations.
Processes start after durable admission under an active controller slot, with
the run's cancellation/deadline and a separate startup deadline. Rejected,
queued-cancelled and idempotent-repeat submissions cannot start an extra server.
The acknowledged startup event freezes bounded schemas before model dispatch;
metadata capture keeps only their digest/count/status while replay capture
retains their untrusted bytes. Live and replay dispatch independently require
both a frozen definition and the run's grant. Normal shutdown precedes terminal
commit; the outer owner also awaits cleanup after unexpected execution/journal
errors. Storage failure may require startup recovery because a failed journal
cannot acknowledge a new terminal record.

Treat metadata as external input: a server's
read-only annotation is not proof that its implementation is read-only. MCP
roots communicate boundaries; enforcement still belongs in the implementation.
[MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools),
[MCP roots](https://modelcontextprotocol.io/specification/2025-11-25/client/roots)

Remote MCP has a separate authorization context. Do not forward an incoming
service bearer token to an MCP server or downstream API. Token passthrough is
explicitly prohibited by MCP's security guidance.
[MCP security practices](https://modelcontextprotocol.io/docs/2025-11-25/tutorials/security/security_best_practices)

An untrusted native plugin can bypass Rust module boundaries and capability APIs.
Before offering arbitrary code execution, use a separate constrained worker with
only necessary filesystem/network rights, no inherited credentials, bounded
CPU/memory/process/output resources, and verified process-tree cleanup.
`Command::env_clear` followed by explicit required variables helps prevent
credential inheritance. [Rust Command](https://doc.rust-lang.org/std/process/struct.Command.html#method.env_clear)

Linux commands require fully enforced Landlock ABI v3 rights and seccomp before
exec; partial/unsupported enforcement refuses launch. Workspace content access
is granted through the configured root, with narrowly documented runtime
exceptions for system binaries/libraries, `/etc/ld.so.cache`, `/etc/localtime`,
and the exact selected executable. Config/state cannot overlap broad runtime
grants. `/proc` is not granted. Non-stdio inherited descriptors are closed on
exec. Seccomp denies network creation, io_uring, group/session escape and
namespace creation; clone3 returns ENOSYS so ordinary thread/process creation
can use its inspected fallback.

This tier restricts the handled content operations, including truncation; it
does not provide universal hostile-code isolation. Landlock does not mediate
all filesystem metadata operations such as chmod/chown, and same-user process
interactions remain a residual boundary. Comprehensive embedding/descriptor
inventory remains [INT-0025](intents/INT-0025-secrets-and-egress.md). Commands
that require other ambient runtime paths may fail closed.
[Kernel Landlock documentation](https://docs.kernel.org/userspace-api/landlock.html)

Windows commands currently have Job lifecycle ownership and scrubbed environment,
without AppContainer filesystem/network isolation. AppContainer/LPAC remains
[INT-0019](intents/INT-0019-windows-command-sandboxing.md). Job membership alone
does not establish a file/network sandbox.
[AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer),
[Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)

## Evidence required before each release

| Boundary | Acceptance evidence |
|----------|---------------------|
| Tool authority | Traversal, escaping links, device/stream syntax, unknown tools, and malformed/oversized arguments execute no forbidden operation. |
| Untrusted content | A file requesting broader permissions or another destination changes neither policy nor transport configuration. |
| Task acceptance | Empty/unknown criteria, client overrides, forged verdicts, wrong values with genuine citations, and cross-run/owner evidence cannot produce a pass; incomplete sources stay unresolved. |
| Data handling | Metadata capture omits prompts/intermediate text; retained final candidates and receipts are bounded and owner-scoped; private replay is explicit; credentials and terminal controls do not leak through logs/display. |
| Concurrent runs | Interleaved scripted runs receive only their own replies, tool results, cancellation, and recorded events; configured shared limits hold. |
| Shared owners | Owner A cannot list, read, stream, cancel, or export owner B's run even with its correct ID; task/workspace/model aliases and required tools cannot escape owner policy. |
| Authentication | Missing, malformed, wrong, expired, and revoked credentials fail; rotation works; query tokens are rejected; direct backend exposure fails deployment checks. |
| Load and storage | Saturation rejects or expires work within the stated contract; slow streams do not block journals; private state stays outside tool access. |

Use synthetic fixtures and deterministic assertions for these checks. Prompt
injection evaluations measure model behavior; authorization tests prove that a
particular forbidden effect cannot pass the implemented gate. Passing either
alone is not evidence for every security boundary. See [testing](testing.md).
