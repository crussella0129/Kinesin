# Dependency and native-build policy

Owner: maintainers. Review this policy for every dependency/toolchain update and
before release. [INT-0013](intents/INT-0013-supply-chain-security.md) records the
initial gate; [INT-0022](intents/INT-0022-completed-contract-repairs.md) owns the
sprint 10 duplicate-policy repair. The committed `Cargo.lock` is the reviewed
resolution, including platform-specific dependencies.

## Blocking checks

The `supply-chain` job in [.github/workflows/ci.yml](../.github/workflows/ci.yml)
runs `cargo deny check` and `cargo audit`. Neither command may ignore its failure.
`deny.toml` admits only listed licenses and the crates.io registry, rejects
wildcard requirements, and rejects duplicate versions except individually
reviewed exact nodes. The manifest is unpublished (`publish = false`); ignoring
its own license metadata does not exempt third-party dependencies.

Advisory exceptions currently number zero. Any future exception needs its
specific advisory ID, affected version, justification, owner and removal trigger.
`cargo audit` has its own configuration; a cargo-deny advisory exception does not
automatically configure cargo-audit. Record and review both if an exception is
ever necessary.

## Duplicate exceptions

Each `bans.skip` entry in [deny.toml](../deny.toml) names an exact version and the
dependency paths requiring coexistence. These are temporary compatibility
exceptions, not proof that a crate's implementation is safe. They exempt only
that node from duplicate detection; advisory, license and source checks still
apply. There are no recursive `skip-tree` exemptions or version-range skips.

Cargo-deny detects duplicates among the remaining unexempt nodes. Updating the
single unexempt partner of an existing exception can therefore remain green;
ordinary lockfile review still applies to that update. A new unexempt duplicate
pair fails. The [cargo-deny policy reference](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html#the-skip-field-optional)
describes this exemption mechanism.

On dependency updates, run `cargo tree --locked --duplicates` and the full deny
gate, inspect each changed path, remove obsolete exceptions and explain any new
exact exception. Keep the all-platform graph: a duplicate needed only by the
Wasm dependency graph still requires an explicit disposition even though Kinesin
currently verifies Windows and Linux runtime builds.

The sprint 10 negative procedure copies the policy to an ignored temporary file,
removes one existing exact exemption, and runs:

```text
cargo deny --offline --locked --config TEMP_POLICY check bans
```

Removing `base64@=0.22.1` while keeping the committed lockfile makes its coexistence
with `base64@0.23.1` fail as `error[duplicate]` (cargo-deny 0.20.2: exit 2).
The original policy passes. This exercises the real command's failure path
without adding a vulnerable crate or editing the repository lockfile. The
temporary policy must be removed after the check.

## Build and native-code inventory

Kinesin has no first-party `build.rs`. Dependencies can execute build scripts and
procedural macros during compilation, regardless of whether their runtime API
is memory-safe. Advisory and source policies reduce known risks; they do not
detect all malicious build code or replace source review.

Inventory recorded from `cargo metadata --locked --offline --format-version 1`
and `cargo tree --locked --offline --target TARGET --edges normal,build` for
`x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`:

| Surface | Resolved components | Boundary and review |
|---|---|---|
| SQLite | `rusqlite 0.40.2` → `libsqlite3-sys 0.38.2` | `bundled` compiles native SQLite using `cc`. Rust wrappers do not make SQLite's C implementation memory-safe. Storage owns access, transactions and result conversion; engine versions/advisories remain part of review. |
| TLS cryptography | `reqwest 0.13.4` → `rustls 0.23.44` → `aws-lc-rs 1.18.1` / `aws-lc-sys 0.45.0` | AWS-LC contains native cryptography and build tooling (`cc`, `cmake`). Rustls avoids an OpenSSL dependency but is not a claim that the complete TLS stack contains no native code. |
| OS APIs and system libraries | `windows-sys`, Windows safe-wrapper crates, `libc`, `rustix` | Platform FFI under filesystem capabilities, networking, signals and process ownership. Review feature/target changes alongside the first-party unsafe inventory in [the threat model](threat-model.md). |
| Capability directory traversal | `cap-fs-ext 4.0.3` with `cap-std 4.0.3` | Adds the public no-follow directory-open API used by directory creation. One new lockfile package; existing capability primitives and platform dependencies are reused. |
| Other target/feature branches | `ring 0.17.14`, JNI/platform verifier branches, `sqlite-wasm-rs` and wasm-bindgen | Present in all-platform metadata/lock resolution; the reviewed Windows/Linux default runtime graphs select AWS-LC and libsqlite3-sys, not ring or Wasm SQLite. Recompute the graph before claiming a different target or feature is supported. |

The custom-build inventory for the two reviewed target graphs includes:

- Native/platform setup: `aws-lc-rs`, `aws-lc-sys`, `libsqlite3-sys`,
  `cap-fs-ext`, `cap-primitives`, `cap-std`, `getrandom`, `io-extras`, both resolved
  `io-lifetimes` versions, and `libc`; Windows also includes
  `windows_x86_64_msvc`, while Linux includes `rustix`.
- Compiler/configuration/data setup: `httparse`, `icu_normalizer_data`,
  `icu_properties_data`, `num-traits`, `proc-macro2`, `quote`, `ref-cast`,
  `rmcp`, `rustls`, `serde`, `serde_core`, `serde_json`, `thiserror` and `zmij`.
- Procedural macros (for example serde, Tokio, rmcp and schema derives) are
  additional executable compiler inputs. This inventory is not a source audit
  of every transitive macro or build script.

Recompute this list using the committed lockfile when dependencies or enabled
features change. Do not run builds with production credentials merely because
the advisory gate is green. Existing CI uses read-only repository permissions
and disables checkout credential persistence; action major tags and downloaded
tool versions remain an accepted build-tool trust boundary.

## cargo-vet decision and release boundary

**Decision, 2026-09-12: defer cargo-vet adoption.** The current baseline is reviewed
lockfile changes plus blocking deny/audit. No reviewed cargo-vet audit store,
trusted-import policy or maintainer capacity has yet been established; silently
adding exemptions for the whole graph would not provide a meaningful audit.

Revisit this decision before distributing the first release binary or approving
a shared-service deployment beyond loopback, and during any proposed major
change to native cryptography, SQLite, credential handling or build tooling.
Maintainers must record whether to fund source audits, which imported audits to
trust, and how exemptions expire. Such a review is a trigger to make a recorded
decision; this document does not claim cargo-vet currently blocks CI.

Release-artifact integrity remains a separate roadmap item: reproducible builds,
SBOMs, auditable binaries and signing require a release pipeline. This dependency
gate neither signs the binary nor establishes a security certification.
