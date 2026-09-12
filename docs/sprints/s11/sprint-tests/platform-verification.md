# Sprint 11 Platform Verification

Date: 2026-09-12. Product source is implementation commit `c7eb088`; local
verification occurred on its exact source before commit. Its ledger successor is
`ee6a26b8c2b662b77511c1f25bfa01a57ea62cd5`.

## Windows

Isolated root: `C:\Users\charl\Kinesin\validation-output\s11-windows`.
Existing Rust/MSVC/native build prerequisites were reused. The PowerShell setup
block was extracted directly from the guide and executed in this root. Fresh
config/state/workspace creation passed; a second execution preserved config and
project sentinel hashes and state ACL SDDL. Task-owned sample content was then
restored for the model walkthrough; the operator's actual config was untouched.

```powershell
cargo run --locked -- --help
cargo install --locked --debug --path . --bin kinesin --root validation-output/s11-windows/install --target-dir target
```

Only kinesin.exe was installed. A process-local PATH addition let Get-Command
resolve it and --help exit 0 from the user's home. From the isolated root, the
installed binary executed the guide's greeting, checked, inspect/export/replay
and bare-session commands. The exact forwarding command was:

```powershell
tailscale ssh charles@nighthawk -N -T -o ExitOnForwardFailure=yes -L 127.0.0.1:8080:127.0.0.1:18080
```

Health returned status=ok. Result JSON and session text are retained in the
isolated root (`greeting.json`, `checked.json`, `inspect.json`, `export.json`,
`replay.json`, `session.txt`, `offline-replay.json`). Run IDs and assertions are
in [e2e-tests.md](e2e-tests.md). The task tunnel PID 8736 was verified by exact
command and creation time 18:00:50Z before termination. Disconnected replay with
the documented relative export path passed in the original execution context.

An initial sandboxed install could not contact crates.io; the network-authorized
retry succeeded. An additional replay attempt from the elevated cleanup context
could not open the private capture; the original-context documented command
passed. Neither failed attempt is counted as a successful test.

## Native Debian on Nighthawk

Debian 13.7, kernel 6.12.94+deb13-amd64; existing GCC/G++ 14.2, Make 4.4.1,
Git, curl and CA certificates sufficed. Free disk was 415 GiB. Rust was absent.
Checksum-verified rustup-init installed Rust/Cargo 1.96.0 with
`-y --no-modify-path --profile minimal --default-toolchain 1.96.0`.
CARGO_HOME, RUSTUP_HOME, build cache and install root remained under
`/home/charles/kinesin-s11-validation`; ordinary Cargo/Rustup homes stayed absent.
Builds used CARGO_BUILD_JOBS=2.

The source archive at `2dcadc431f3c17b2b1052868c684c3e11b15805e` received a
hash-verified overlay of Cargo.toml, src/cli.rs, tests/cli_inspect.rs,
kinesin.example.toml and the guide. The guide later received two prose/provisioning
corrections; the executed Linux workspace setup block and product source were unchanged.

Fresh native per-file hashes matched the committed LF source at
`c7eb088ad104d381853ede5d2288c8d97e663aa2`:

| Product input | SHA-256 |
| --- | --- |
| Cargo.toml | d9d56137fe078a4996fe8e4b2032ff00e3c1d67955d6ac5de937dde16a971b87 |
| src/cli.rs | 62cedaf7fd540e7cabb2fe28a0b2652bd4d90bc9ea62661a313fb7322ba96ae6 |
| tests/cli_inspect.rs | f295f17a3ba7bca41c8d01a3f439376bbaa72d666fcfd3d759b0f9306880344e |
| kinesin.example.toml | 1b6474d1c88c7b35fab1a4975f647ca0d8713575c9f7562756de85048e57d461 |

```bash
cargo run --locked -- --help
cargo install --locked --debug --path . --bin kinesin --root /home/charles/kinesin-s11-validation/install
```

Only kinesin was installed. Installed --help and -h matched from outside source.
The exact guide setup block created mode 700 directories and mode 600 files;
repeat runs preserved existing content and permissions, including deliberately
different sentinel modes. The caller's umask was unchanged.

The copied starter changed only base_url port 8080 to 18080. An absolute config
path selected it when running the installed binary outside the source checkout.
Greeting, checked task, inspect/export/replay and one session prompt followed by
EOF all exited zero. Raw logs are under
`/home/charles/kinesin-s11-validation/logs/`, including
`walkthrough-summary.json`, `setup-validation.log`, install/help logs,
`cleanup-and-offline-replay.log` and `offline-replay.jsonl`.

The retained runtime identifies itself as 6500 (a7a98e0f), Vulkan0 / RTX 2070
Super, alias kinesin-qwen25-coder-7b, context 4096, one slot, loopback port 18080.
PID 46093, executable path and start time 610768791 were checked before TERM.
Afterward the executable and listener were absent, health returned curl exit 7,
and disconnected replay still passed.

| Artifact | SHA-256 |
| --- | --- |
| Source archive | eaaf1fe71fbaff47faca7bf32ea60e5ce241a1033eb4a774f2b6341a376a27e6 |
| Source/guide overlay archive | 3b91634519f14231f2c65e5788f6282e9ec02750987f9d9ad29a094662f8b79c |
| Official rustup-init | dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71 |
| Installed debug Kinesin | c35afff97443a8bc643b076444cf2f325ed1b5fdde27f6e80ca316bb6e69a23b |
| Retained llama.cpp b6500 Linux Vulkan archive | ee15538d98808f15d13a474d2554bc708a475e35bf560e1d0d21b9e96bc5f8ea |
| Retained Qwen GGUF | 509287f78cb4d4cf6b3843734733b914b2c158e43e22a7f4bf5e963800894d3c |

## Boundaries

Not executed: system prerequisite installation, fresh git clone, interactive or
default-home Rust installation, persistent PATH configuration, release-profile
product installation, Windows model download/startup, or new model download on
Nighthawk. The guide distinguishes retained paths from fresh download instructions;
official b6500 archive names/digests were checked against release metadata. No
system packages, GPU drivers, shell profiles or existing operator files changed.
Documentation supports PowerShell 5.1 syntax, while execution used this host's
PowerShell runtime. Full offline platform suites belong to canonical CI, not
these targeted native-host walkthroughs.
