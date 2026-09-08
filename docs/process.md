# Working on the harness yourself

Implement the [build guide](build-guide.md) one proof at a time. Its code is
intentionally yours to write. The architecture and contracts are proposed
requirements; a checked box means you have demonstrated behavior, not merely
created a file with the expected name.

## A repeatable session

1. Select one guide step and state its observable success condition.
2. Read only the linked material needed for that step. Try unfamiliar Rust
   ownership/async behavior in a separate scratch project.
3. Write the smallest change that satisfies the contract. Keep a compiling
   checkpoint before widening the behavior.
4. Run that step's normal proof and failure exercise. Use deterministic fakes
   for failure timing and saved fixtures for protocol boundaries.
5. Run the relevant format/lint/test checks, inspect the diff for credentials
   and generated state, then commit a coherent result.
6. Check the corresponding [roadmap](roadmap.md) box and leave a sentence about
   the next unresolved proof.

Split a step into several commits if necessary. Ask for help with a diagnostic
or a design decision while keeping implementation ownership: show the error,
the smallest relevant code, your expected behavior, and your current hypothesis.

## Rust checkpoints

Once the package exists:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run `cargo fmt --all` to apply formatting. During an edit, a focused test or
`cargo check` provides faster feedback. Run the full small package checks at
coherent checkpoints; do not run a model/GPU-dependent evaluation for every
formatting change.

Set a tested stable toolchain and edition in the implementation. The proposed
`std::fs::File::try_lock` requires Rust 1.89 or later; dependency requirements
may raise that floor. Record the actual tested versions rather than treating
a date in a tutorial as a compiler specification.
[Rust toolchains](https://rust-lang.github.io/rustup/concepts/toolchains.html),
[Cargo rust-version](https://doc.rust-lang.org/cargo/reference/rust-version.html)

Add CI at guide step 3. Initially it only builds and tests the available package;
expand it with the implementation. Keep fast tests independent of model downloads,
API credentials, GPUs, and remote services. Add native Windows and Linux
filesystem/lifecycle checks when those boundaries exist and both platforms are
claimed as supported. A WSL run does not prove native Windows behavior.

## Dependencies and interfaces

Commit `Cargo.lock` for this application. Add a dependency when its guide step
needs it; check current features and minimum Rust requirements before copying
a command. Use maintained libraries for HTTP, TOML, JSON, SQLite, cryptography,
and capability path resolution. Hand-writing the harness means writing its
policy and orchestration, not implementing every underlying protocol yourself.

Keep provider JSON in `model.rs`, SQLite in `storage.rs`, and filesystem access
in `tools.rs`. Prefer owned domain values across asynchronous boundaries.
An `Arc` means shared ownership; it does not establish access authority or a
capacity limit. Introduce a trait or separate crate after a real second use
makes its contract clearer.

Document an architectural change in [decisions](decisions.md), update the
authoritative contract, and then update the affected guide/checklist references.
Do not leave two conflicting descriptions of the same behavior.

## Tests and experiments have different jobs

Unit and integration tests prove specified behavior under controlled inputs.
Live evaluations measure whether a selected model/template can use that behavior
to solve useful tasks. Benchmarks measure a named workload on a named machine.
Neither passing fake tests nor a convincing live answer establishes all three.

Follow [testing](testing.md) for invariants and evaluation gates. Save the
baseline with its server/model/template identity, limits, hardware, offered load,
latency distributions, useful completion count, errors, and rejected work.
Change one important variable per comparison.

Keep a short experiment note: hypothesis, setup, observation, decision. Write
“not measured” when it is not measured. Do not infer secure deployment or
latency guarantees from a documentation review.

## Repository hygiene

Exclude credentials, private configs, model weights, runtime state, databases
and their sidecars, exported captures, real workspace data, and benchmark inputs
containing private content. Sanitize fixtures before committing them.

Use ordinary feature commits or branches. Version control protects committed
history; it does not automatically preserve an uncommitted edit or secret that
was accidentally published. Review `git diff` before staging.

When changing a storage schema or authority contract, update the version and
migration/compatibility tests deliberately. Replay incompatibility should be
explicit; old captures must not silently acquire today's policy or tool behavior.
