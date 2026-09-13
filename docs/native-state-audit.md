# Native state and service-process audit

Machine-local account names, host identifiers, and absolute paths in this
historical report are normalized to role labels or explicit placeholders for
privacy. Recorded outcomes and source-evidence references are unchanged. Replace
the placeholders in reproduction commands with your own operator and state path.

The existing development `state` directory does **not** meet the private-state
startup policy. On 2026-09-08 the read-only Rust auditor rejected it with
`private_state_inherits_parent_permissions`. Its records at this checkpoint are
synthetic. This is a remaining provisioning requirement, not a successful
shared-service deployment result.

## Observed Windows permissions

The tool process ran as the Windows audit host's offline sandbox account,
distinct from its enabled operator account. A sandbox-users group contained
separate offline and online sandbox accounts. These role labels replace the
machine-local account names and identifiers.

| Object | Owner | Observed DACL |
|---|---|---|
| Project directory | Administrators | Inherits SYSTEM, Administrators, and the operator account's FullControl; additionally grants Modify/Synchronize to the sandbox-users group and two unresolved sandbox SIDs |
| `state` | Offline sandbox account | Unprotected; inherits all six grants above |
| `validation-output` | Offline sandbox account | Unprotected; inherits the same grants |
| `C:\Users\<operator>` | Not inspected | Reading its ACL was denied to the current sandbox token |

The state, validation-output, and practice workspace entries were ordinary
directories, without a reparse attribute at those named entries. That does not
prove the entire ancestry safe. The inherited grants are broader than the
operator/current-controller principal set. No effective-access test under the
other account was performed, so this audit does not claim a demonstrated data
read by that account. No user, group, firewall rule, existing directory ACL, or
existing owner was changed.

## Private loopback service fixture

The subsequent service test created a new
`validation-output/service-validation/private` directory with a protected DACL
at creation. It trusts only the current controller account, the operator, SYSTEM and
Administrators, with inheritable grants for SQLite sidecars and the verifier
file. The service accepted the tree through its production startup validator,
provisioned two synthetic-owner credentials, and served a real checked task.
An independently broadened child and an outside hard link remain negative test
cases; existing development state was not repaired or treated as private.

The service tests exercised live revocation, replacement credential loading,
owner isolation and process-crash recovery in this tree. Temporary bearer tokens
and all SQLite files remain in ignored private test state. They are not examples
or committed evidence. This fixture does not establish a dedicated production
identity, trusted full ancestry, forbidden-egress policy, or TLS deployment.

## Implemented validator

[`private_state.rs`](../src/private_state.rs) provides
`PrivateStatePolicy::current(additional_trusted_sids)` and
`validate_private_tree(absolute_path, policy)`. Its production API is read-only:
it never creates objects, repairs ACLs, or broadens permissions. Additional SIDs
are trusted operator configuration, not request fields. The caller remains
responsible for trusting every selected principal and its group membership.

On Windows it opens each object for security/attribute inspection, checks the
descriptor through that handle, and rejects an unprotected root DACL,
untrusted owner or allow ACE, absent/null DACL, unsupported ACE forms, reparse
points, and multiply linked state files. Directory grants must propagate to
both files and subdirectories without a no-propagate restriction. Existing
children are checked individually; a private parent alone cannot certify a
child's independently assigned descriptor. Windows distinguishes a null DACL,
which permits access, from an empty DACL, which denies it. This implementation
also rejects an empty directory DACL because it cannot establish usable private
inheritance for future sidecars. See Microsoft's
[handle-based security query](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo),
[DACL distinction](https://learn.microsoft.com/en-us/windows/win32/secauthz/null-dacls-and-empty-dacls),
and [inheritance rules](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-inheritance-rules).

The default allowed identities are the current process user, SYSTEM, and
Administrators. The operator's SID can be supplied explicitly when it differs
from the controller identity. Unresolved inherited sandbox SIDs are not silently
added to this list. Administrative identities remain trusted; this is not
isolation against an administrator or a compromised controller.

The walk is capped at 1,024 objects and 32 directory levels, with at most 64 ACEs
per descriptor, eight additional SIDs, and a 64 KiB security descriptor. These
are work/allocation limits, not hard deadlines on individual filesystem calls.
Run startup inspection off the async request path. A successful report retains
the root handle without Windows delete sharing; keep it through controller
shutdown. This helps preserve the inspected directory entry but does not replace
a trusted, stable parent tree. The relevant opening/sharing semantics are
documented by [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).

The Unix implementation checks the effective UID, zero group/other permission
bits, ordinary file/directory types, no-follow opens, and single-link state
files. Provision directories as 0700 and files as 0600 before sensitive writes;
the validator does not silently chmod existing data. Its Unix-specific test is
present but was **not run on this Windows host**.

## Native evidence and reproduction

Four native Windows tests passed, including actual filesystem/descriptor calls:

- A fresh temporary directory was created with a protected DACL at creation,
  granting the current process, the operator, SYSTEM, and Administrators access.
  A normally created child file inherited private permissions and passed.
- A different fresh child was created with an explicit broad read ACE; checking
  the whole tree rejected it despite its private parent.
- A hard-link alias to an outside synthetic sentinel was rejected.
- In-memory descriptor fixtures distinguished unprotected, null, empty,
  unsupported object-ACE, broad-principal, and nonpropagating cases.

Only the newly created synthetic objects received descriptors. Cleanup verified
the absolute temporary prefix and removed that fixture tree. The test did not
change any preexisting object's ACL.

```powershell
# Optional when the operator differs from the account executing the test:
$env:KINESIN_TEST_OPERATOR_SID = (Get-LocalUser -Name 'YOUR_OPERATOR_ACCOUNT').SID.Value
cargo test --lib private_state:: -- --nocapture

# Read-only audit of an existing, explicitly chosen tree:
cargo run --example audit-private-state -- 'C:\path\to\private-state' --trust-sid $env:KINESIN_TEST_OPERATOR_SID
```

The audit command was run against the development state recorded above and exited
with `private_state_inherits_parent_permissions`. Passing a known operator SID
does not suppress the inherited-root check. The
[example auditor](../examples/audit-private-state.rs) reports the stable failure
code and never performs a repair.

## Concrete provisioning proposal

Provision a **fresh, empty** destination before moving sensitive operation to
it. Do not run an ACL reset recursively against the project or an existing state
tree. The proposed local synthetic probe would change only its newly created
directory; the proposal has not been applied to the existing state.

| Property | Current development state | Proposed fresh private directory |
|---|---|---|
| Root inheritance | Enabled | Disabled with a protected DACL |
| Allow principals | Inherited sandbox group, two unresolved sandbox SIDs, operator, SYSTEM, Administrators | Explicit controller SID, operator SID, SYSTEM, Administrators |
| Propagation | Broad inherited grants propagate | Only selected grants propagate to files and subdirectories |
| Owner | Sandbox account | Deliberately selected trusted controller or operator account |
| Existing objects | Synthetic journal already exists | None until the descriptor and audit pass |

Resolve the exact operator and intended service SIDs administratively. Preserve
the operator's explicit access; do not confuse the Codex sandbox identity with
the operator account. Windows provisioning can supply a security descriptor at directory
creation, as the native test does. A new empty directory may alternatively be
created and tightened before **any** sensitive file is written. Do not copy the
old inherited grants merely to make the audit pass. Protect every destination
used for credential verifiers, SQLite/WAL/shared-memory files, backups, retained
exports, and replay payloads. Validate restored/copied descendants too.

For existing journals, first close admission and settle/stop the controller,
then use the supported backup/restore path into the fresh private destination.
Recheck integrity, schema, owner-scoped records, and permissions before starting
there. Removing old artifacts requires a separate retention decision; this audit
does not delete existing data. WAL durability and an API owner check do not
encrypt or protect database files from OS-level readers.

## Readiness and exposure requirements

Treat process liveness and readiness to accept work separately. The service's
authenticated `/ready` endpoint currently reflects the trusted readiness flag
and whether the journal accepts work. Startup/health orchestration must set that
flag from actual prerequisite checks, rather than equating a constructed service
object with a ready deployment.

The passing native service integration test
`readiness_blocks_new_work_but_preserves_authorized_retry_and_retrieval` in
[`tests/service.rs`](../tests/service.rs) verifies authenticated `/ready` 503,
rejection of new submissions before dispatch, continued authorized retrieval and
known idempotent retry, then readiness/admission restoration. Reusing an existing
run does not admit a new one. That test exercises the readiness control path;
it does not substitute for deployment health checks.

| Condition | Liveness | Readiness/admission | Existing results |
|---|---|---|---|
| Private state, writer, and required model healthy | Respond if the process is alive | Ready; bounded admission allowed | Authorized retrieval allowed |
| Private-state audit fails before startup | Process may exit clearly | No listener/admission should open | Operator remediation required |
| Required model unavailable while process lives | Alive | Unready; no new runs | Preserve authorized retrieval when storage remains healthy |
| Journal unavailable or shutdown begins | Alive while settling | Unready; admission closed | Only what the remaining healthy storage path can actually return |
| Overload | Alive | Reject/expire work under the queue contract | Do not expose data or expand buffers |

Model-health work must use bounded timeouts and a small separate monitoring
budget; it must not consume every inference slot or flood the journal. A public
liveness response should reveal no owner, configuration, credential, or storage
detail. A successful ACL audit is only one readiness input; writability, disk
space, model compatibility, process lock, and writer health remain independent.

Keep state and credentials physically outside all tool roots. Canonical path
separation alone cannot recognize a hard link to the same file: Windows hard
links give multiple names to one file, while junctions redirect directory
resolution. [Microsoft's link model](https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions)
explains why curated input trees and link checks are separate from lexical
traversal rejection. An intentionally exposed secret inside a tool tree is still
readable data; the capability library cannot classify its sensitivity.

The deployed service still needs a dedicated OS identity, controlled input-tree
provisioning, cross-owner sentinel/link tests under that identity, and verified
network restrictions. The Codex sandbox token is not that deployment identity.
This pass did not prove denial of unrelated operator credentials, permitted-only
egress, parent-directory mutation resistance, Linux behavior, or arbitrary-code
isolation. No firewall or user-account changes were made. Until those checks are
performed on the actual deployment, the shared-service exposure gate remains
open work; local synthetic functional tests must not be presented as its proof.
