# Threat model & security assurance

- **Version:** 1 (2026-09-11) · **Owner:** maintainers · **Cadence:** reviewed each security-affecting sprint and at each release.
- **Intent:** [INT-0015](intents/INT-0015-threat-model-assurance.md). This is the single standards-mapped view of Kinesin's security posture; [security.md](security.md) holds the boundary contracts, [adversarial-review.md](adversarial-review.md) the attack matrix and out-of-scope claims.

Each row below is one of: a **Mechanism** (an enforced control with a proving test), a **Gap** (desired, owned by a named intent, not yet built), or an **Accepted residual risk** (out of scope by design). No row is "covered" without evidence.

## 1. OWASP LLM Top-10 (2025) & Agentic Top-10 (2026)

| Risk | Status | Kinesin position |
|------|--------|------------------|
| Prompt injection (LLM01) / goal hijacking | Mechanism | All tool/model output and file content is **data, never instructions**: the model receives tool *descriptions*, authority comes only from operator config, and a checked run's verdict is an independent checker over source bytes. Tests: `live_evaluation` hostile-text cards, `redteam_treats_content_as_data`. |
| Excessive agency (functionality/permissions/autonomy) | Mechanism + Gap | Immutable `RunAuthority` (per-run capability scoping), per-workspace tool allow-list, mutating tools barred in checked runs, argv-only commands under an OS sandbox (INT-0012). **Gap:** per-action human-approval gates & just-in-time privilege → **INT-0017**. |
| Sensitive information disclosure | Mechanism | Owner-scoped storage/queries; metadata capture omits prompts/intermediate text; private replay is explicit; credentials/terminal controls never logged. Tests: `service` (owner isolation), `settlement`. |
| Supply-chain | Mechanism | `cargo-deny` + `cargo-audit` CI gate, committed `Cargo.lock`, rustls (no OpenSSL) — **INT-0013**. **Gap (residual):** signed/auditable release artifacts + SBOM → roadmap parking-lot (needs a release pipeline). |
| Improper output / tool misuse | Mechanism | Tools are argv-only capability handlers under cap-std path scoping + (Linux) Landlock/seccomp; no shell string; bounded output/timeout. Tests: `command_tool`, `sandbox_linux`. |
| Unbounded consumption / DoS | Mechanism | Admission control, per-run limits (turns/tool-calls/run-time/output), bounded journal queue, saturation rejection/expiry. Tests: `adversarial_runtime`, `service_load`. |
| System-prompt leakage | Accepted residual risk | Instructions live in operator config, not secrets; the model may reveal its own instructions — treated as non-secret by design. |
| Vector/embedding weaknesses | Not applicable | No embedding/vector retrieval (see decisions.md); checked evidence is literal source bytes. |
| Memory/context poisoning | Mechanism + Gap | Immutable append-only journal; compaction preserves evidence-bearing groups. **Gap:** tamper-evident (hash-chain + signed receipts) → **INT-0014**; persistent cross-run memory not yet built. |
| Misinformation / world-truth | Accepted residual risk | Observed-file agreement is not a truth guarantee (adversarial-review.md); open-ended quality stays unchecked. |

## 2. NIST AI-agent controls

| Control | Status | Kinesin position |
|---------|--------|------------------|
| Policy-based authorization | Mechanism | `RunAuthority` + config-declared workspaces/models/tools/owners; discovery/model output never grants authority. |
| Task scoping / least privilege | Mechanism | Per-run capability set; read vs. write vs. command are distinct capabilities built only where granted. |
| Just-in-time privilege | Gap | Step-scoped elevation → **INT-0017**. |
| Human-approval gates for high-impact actions | Gap | Per-action approve/deny for irreversible effects → **INT-0017**. |
| Dev/test/prod separation | Accepted residual risk (operator) | Deployment concern; the runtime is single-config per process. Documented as operator responsibility. |
| Transport authentication / identity | Mechanism + Gap | The loopback service authenticates a bearer credential (`subtle` constant-time compare, provisioning/rotation, verifier reload); direct backend exposure fails deployment checks. **Gap (residual):** mutual TLS / stronger transport identity is required **before any non-loopback exposure** — folded here as a residual-risk boundary, implemented when INT-0008-style remote service exposure is built. |

## 3. Memory-safety statement (CISA/NSA memory-safe roadmap)

Kinesin is written in **Rust**, a memory-safe language; the pure decision core
(`core.rs`), model adapter, runner, policy, verification, scheduler, storage
logic, and config carry **no `unsafe`**. `unsafe` exists only at unavoidable
OS/FFI boundaries with no safe wrapper, each localized and justified:

| File | `unsafe` use | Justification & containment |
|------|--------------|------------------------------|
| `src/private_state.rs` | Windows security-descriptor / SID / process-token APIs (windows-sys); `libc::geteuid` (unix) | Reads OS ownership/ACLs to verify the private state tree is not world-accessible — no safe Rust API exists. Pointers are validated (`IsValidSid`, length bounds), handles are `OwnedHandle`-wrapped, buffers bounded. |
| `src/storage.rs` | `CreateDirectoryW` with a security descriptor (windows-sys) | Creates the private state directory with an owner-only ACL at first use; descriptor freed with `LocalFree`; scoped to setup. |
| `src/signal.rs` | `SetConsoleCtrlHandler` (windows-sys) | Registers native console cancellation; a single call, no pointer arithmetic. |
| `src/tools.rs` | The Linux command-sandbox `pre_exec` closure (INT-0012) | Applies a parent-built Landlock ruleset + seccomp filter in the forked child before `exec` — apply-only syscalls, no allocation (the fork-safe pattern). |

Posture: the memory-unsafe surface is small, FFI-only, OS-security-boundary
work; the roadmap direction is to keep new `unsafe` out of the core and confined
to reviewed FFI shims. This satisfies the CISA/NSA expectation of a documented
memory-safety approach for a memory-safe-language product.

## 4. Red-team corpus → release-evidence matrix

The corpus is the executed tests below; each [security.md](security.md#evidence-required-before-each-release) release-evidence-matrix row maps to the test(s) proving it. All run in CI.

| Matrix row | Proving tests |
|------------|---------------|
| Tool authority | `command_tool`, `sandbox_linux`, `redteam_denies_unauthorized`, runner tool tests |
| Untrusted content | `live_evaluation` hostile-text cards, `redteam_treats_content_as_data` |
| Task acceptance | `runner_tools`, `settlement`, `replay` (forged-verdict / cross-owner-evidence rejection) |
| Data handling | `settlement`, `cli_inspect` (metadata omits prompts; owner-scoped receipts) |
| Concurrent runs | `adversarial_runtime`, `service_load` (isolation + shared limits) |
| Shared owners | `service` (owner A cannot reach owner B's run) |
| Authentication | `service`, `auth` unit tests (missing/malformed/expired/revoked/rotation) |
| Load and storage | `adversarial_runtime`, `service_load`, `process_recovery` |

Prompt-injection evaluations measure model behavior; authorization tests prove a
forbidden effect cannot pass the implemented gate — both are recorded, neither
substitutes for the other.
