# Sprint 10 — transport and MCP audit

Implementation opened only after `intent-first-review.md` was recorded and the
independent intent-only pass completed. Findings refer to the pre-repair dev head.

## INT-0008: partially executed, completion overclaimed

The same `ModelConfig`/`Koil` client supports local and remote origins without
runner changes (`src/config.rs:651`, `src/model.rs:403`, `src/runner.rs:237`).
Request preparation keeps the destination separate from model decision bytes;
additional named profiles need only configuration. Redirects, proxy inheritance,
and retries are disabled (`src/model.rs:417-419`), and protocol tests exercise
redirect refusal and mismatched prepared destinations.

Confidentiality fails the original acceptance: `validate_origin` at
`src/config.rs:1020-1041` admits plaintext on RFC1918, CGNAT, and ULA addresses
without proving an encrypted overlay. The comment and `docs/integration.md`
incorrectly equate address privacy with encryption. Public HTTPS opt-in also
departs from the original non-public deployment requirement. The live fixture
compares loopback to the same host's LAN address, not a second physical host.

Repair: require HTTPS for every non-loopback model origin, retaining explicit
public-destination opt-in separately. This supplies an enforceable confidentiality
boundary without trying to infer routes or tunnel encryption from address bits.
An operator may reach a remote model through an encrypted tunnel terminating at
loopback; any direct remote URL needs TLS. Add all address-family regression
cases and preserve an explicit outstanding two-host encrypted deployment proof.
Do not describe a local fixture as that proof.

## INT-0005: core capability works; lifecycle/resource gates are incomplete

Satisfied core paths: operator server declarations and reference validation
(`src/config.rs:702-747`); checked-run bar via mutating `ToolRef`; frozen schema
and request emission (`src/policy.rs:242`, `src/model.rs:82`); fixture tests for
approved, unapproved, poison-as-data, result truncation, and pure replay
(`tests/mcp.rs`). Service and CLI share `Job::discover_mcp`, so the capability
is available through both interfaces. Replay never reconnects (`src/replay.rs`).

Concrete defects against equal resource/authority gates:

1. `src/service.rs:316-327` and CLI submission prepare MCP before
   `ControllerHandle::try_submit`. Every attempted submission can spawn child
   servers before admission rejects capacity, and queued jobs retain processes.
   Discovery must be owned by bounded controller execution, cancellation, and
   the run deadline, with frozen definitions recorded before model dispatch.
2. `src/mcp.rs:162` uses SDK `list_all_tools`, accumulating arbitrary pages before
   checking allowed schema sizes. rmcp 3.3.0 `transport/async_rw.rs:137` uses an
   unbounded `read_until` buffer. Output truncation after decoding does not bound
   memory for protocol frames or metadata. Bound wire frames before SDK decoding,
   discovery pages/tool count/cursors and aggregate frozen metadata.
3. `src/mcp.rs:128-143` inherits the environment, working directory, and default
   stderr. rmcp's child builder defaults stderr to inherit, permitting untrusted
   text/control bytes to reach routine terminal output outside framed observations.
   Scrub the child environment and discard stderr by default; any credential
   configuration must be explicit and destination-scoped.
4. `src/mcp.rs:94-100` claims drop tears down a child, but rmcp's child wrapper is
   a plain process, with cleanup spawned in Drop and no process-group wrapper.
   Pool drop signals async cleanup rather than waiting for all descendants.
   Own process-tree teardown and join it before releasing the run's resources,
   including startup failure, cancellation, and timeout.
5. `RunAuthority::with_mcp_tools` accepts arbitrary definitions while live/replay
   MCP dispatch checks frozen membership without independently checking
   `allows_tool` (`src/runner.rs:941-997`, `src/replay.rs:646-674`). Trusted callers
   currently discover from the allow-list, but the claimed two independent gates
   must hold even for malformed frozen sets. Validate freeze consistency and
   explicitly intersect dispatch with the run grant.

Lightweight required-key JSON validation is expressly documented by sprint 9;
full JSON Schema validation is a follow-up interoperability choice, not a covert
new acceptance requirement. MCP processes remain operator-trusted binaries;
this audit does not pretend command sandboxing confines them.

## INT-0020: proposed, not implemented

The only configured MCP shape is a local argv server (`src/config.rs:505-514`),
and live pools use stdio. There is no HTTP/OAuth/resource-indicator client.
Those outcomes remain proposed. Amend the plaintext-private-address criterion
to authenticated confidential transport and keep inbound credentials separate.

## Evidence limits

The initial all-target Rust suite was started before repairs. Existing passing
tests demonstrate their asserted paths, not the omitted overload, giant-frame,
descendant-cleanup, confidentiality, or forged-definition cases above. SDK
source inspected is the pinned locally resolved rmcp 3.3.0, not latest-version
assumptions. No external server or private credential was contacted.
