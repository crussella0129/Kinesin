# Module and learning map

One package can have a binary target and a library target. Modules are the first
organization boundary; separate packages/processes must earn their cost.
[The Rust Book](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)

| Module | Responsibility | Rust concepts to learn here |
|--------|----------------|-----------------------------|
| `main.rs` / `lib.rs` / `cli.rs` | CLI composition and public library API | Modules, visibility, `Result`, exit status |
| `core.rs` | K-Core: messages, state, pure decisions | Owned `String`/`Vec`, structs/enums, pattern matching, borrowing |
| `runner.rs` | Execute one run's effects in order | `async`/`await`, `select!`, cancellation, RAII |
| `model.rs` | Koil: scripted and HTTP model clients | Typed serialization, enums, optional fields, bounded streams |
| `config.rs` | Parse and validate immutable settings | Serde/TOML, `PathBuf`, validation separate from parsing |
| `policy.rs` | Trusted construction of RunAuthority | Private fields, narrow methods, capability passing |
| `tools.rs` | Bounded read/list operations | Open directory handles, byte/UTF-8 limits, blocking-job ownership |
| `verification.rs` | Frozen task contracts, scoped evidence, pure field checker, bounded receipts | Enums, typed deserialization, slices, exhaustive outcomes, small parsers |
| `storage.rs` | Transactional journal and run queries | SQL parameters, transactions, thread ownership, bounded channels, acknowledgements |
| `scheduler.rs` / `dispatch.rs` | Admission, queues, shared effect permits, owner fairness | `VecDeque`, semaphores, `JoinSet`, capacity accounting |
| `replay.rs` | Validate recorded decisions without effects | Versioned inputs, pure validation, exhaustive outcomes |
| `service.rs` / `ingress.rs` | Authorized API and bounded HTTP connections | Extractors, middleware order, resource authorization, owned connections |
| `auth.rs` / `private_state.rs` | Credential verification and native state checks | Fixed-size secrets, OS metadata and permissions |
| `operator.rs` / `signal.rs` | Readiness, supervision and native shutdown | Concurrent owners, signals, failure propagation |

The answer key records durations and counts at their owning runtime boundaries;
`examples/measure.rs` collects repeatable measurements. A separate telemetry
module is unnecessary until that extraction helps. Optional model-process
supervision (`kineserve.rs` in the earlier design) remains unimplemented in the
selected attach profile; it would introduce `Command`, `Child`, readiness and
cleanup ownership.

Keep most helpers private or `pub(crate)`. Integration tests should use the small
public runtime API, not force every helper to become public. Pure unit tests can
exercise private logic.

## Initial data ownership

A run owns its history and counters. It receives an immutable authority snapshot.
A pooled HTTP client can be cloned without cloning the conversation. A model
profile identifies an approved destination; credentials do not appear in domain
messages or serializable configuration snapshots.

The scheduler owns task handles and accounting. The filesystem worker owns its
blocking-work permit. The storage thread owns its connection. These relationships
are more important than the exact filenames.

Keep owned fields until you have a reason for stored borrows. A clear small clone
can be appropriate. Do not add `Arc<Mutex<_>>` simply because two functions need
to see the same data; first decide who owns it and whether an immutable borrow
or message is enough.

## Dependencies introduced along the guide

| Tool/library | Why it is here |
|--------------|----------------|
| `serde`, `serde_json`, `toml` | Established formats at the boundaries |
| Tokio, `tokio-util` | Concurrent I/O, cancellation, bounded coordination |
| reqwest | Pooled async HTTP |
| `cap-std` | Handle-relative filesystem authority |
| `rusqlite` with a verified SQLite build | Transactions, indexed owner-scoped queries, crash consistency |
| `uuid` | Opaque run identifiers |
| `sha2` | Versioned content/submission fingerprints and high-entropy credential verifiers |
| `tracing` / subscriber (optional) | A possible structured diagnostics layer; current measurements use owned timing records and bounded exports |
| Axum, relevant Tower layers | Later service HTTP boundary |
| `getrandom`, `subtle` | Later credential generation and fixed-size verifier comparison |

Inspect selected versions/features, commit `Cargo.lock`, and record the toolchain.
Do not copy stale feature flags or add every row before its step. The
[build guide](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md) tells you when each has a job.

## Boundaries intentionally absent

No `common` dumping-ground crate, embedded WireGuard implementation, generic DAG
engine, plugin loader, arbitrary shell tool, or model-spawned subagent scheduler
is needed for the first complete release. The design keeps those possible through
explicit authority and effect contracts, rather than pretending they are already
implemented by a trait.
