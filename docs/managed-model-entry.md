# Local model entry follow-up

This direct follow-up T-109 extends PR12 before merge, following the user's
report that interactive entry still exposed a personal remote deployment and
required URL/model-ID knowledge. It does not reopen the completed sprint11.

## Implementation plan

1. Keep mandatory working-folder selection. Discover bounded regular GGUF files
   in per-user model storage and launch-folder `model`/`models` directories;
   offer an arrow/Enter selector, saved selection and custom file choice.
2. Resolve a trusted local llama-server executable outside the working folder.
   Treat GGUF and runtime parent directories as protected runtime resources.
   Persist model changes atomically with a private backup and preserve unrelated
   operator settings. Path overrides apply to human session setup only.
3. Start one owned loopback backend with typed arguments and a unique served
   identity. Gate resource preparation/admission on health, identity, context
   and slot readiness. Bound startup and diagnostic retention; stop admission
   and settle work on child death. Reap the owned tree on all exit paths.
4. Preserve explicitly selected external HTTP(S) serving. SSH authentication
   remains in a foreground SSH terminal and is never collected by the model.
5. Remove personal machine/account/model-alias defaults from active usage
   examples. Normalize sensitive environment labels in prior public evidence
   without changing historical IDs, hashes, timestamps or outcomes.

## Validation plan

- Configuration tests: missing/invalid files, bounds, disjoint model/runtime
  parents, unsupported service scope and profile mismatch.
- Selector/onboarding tests: bounded discovery, canonical deduplication,
  trusted runtime resolution, cancellation, preserved settings and backups.
- Native process tests: delayed readiness, never-ready and identity/context
  mismatch, startup cancellation, child death, bounded logs and descendant
  cleanup, including CLI ownership rather than only isolated primitives.
- Windows and native Debian terminal walkthroughs: arrow/Enter selection,
  startup, actual model tool use, exit and Ctrl+C cleanup. Record actual source
  and artifact identities; distinguish real model evidence from fixture tests.
- Format, all-target/all-feature Clippy, relevant/full tests, dependency gates,
  independent review, Book/link checks and final PR12 CI. Do not merge.

## Results

The implementation is complete in source commit
`c099fd49f011d929c38c00ad06324b92f65fa364`. This follow-up remains on PR12;
it has not been merged. The final hosted CI checkpoint is recorded in the PR.

### Automated checks

Windows Rust 1.96.0: `cargo test --locked` passed **355 tests**, with nine
explicit live/experiment tests ignored. Native Debian Rust 1.96.0: `cargo test
--locked --all-targets` passed **368 tests**, with nine ignored. Windows
`cargo clippy --locked --all-targets --all-features -- -D warnings`, native
Debian all-target Clippy, and formatting passed. `cargo deny check` passed
advisories/licenses/bans/sources; `cargo audit` checked 264 package entries
against 1,243 advisories without vulnerabilities.

The new managed-runtime and full-CLI tests pass on both platforms: delayed
readiness; wrong identity/context; never-ready timeout; startup cancellation;
bounded escaped logs; leader death during a model request and during idle
terminal input; descendant cleanup; EOF with durable completion; and no
interference with an unrelated listener. Fifteen Windows onboarding tests and
seven selector tests cover local choice, custom paths, preserved settings,
private backups, bounds and trusted runtime resolution.

Independent review found and fixed three integration problems: redirected
stdout could cause nested stdin locking; Windows canonical path prefixes could
defeat the launch-directory runtime exclusion; and model startup could race
asynchronous Ctrl+C registration. Human setup now requires terminal input and
output, runtime comparisons canonicalize the launch folder, and cancellation is
registered synchronously before spawning the backend. Native terminal testing
also changed the picker to display filenames first, with the selected path
below, so a long installation path cannot obscure the model name.

### Windows terminal and filesystem proof

The installed command was exercised from outside the checkout in PowerShell
5.1.26100.9444 using isolated settings/state. It discovered the GGUF in the
installed executable's sibling `models/` directory and llama-server in
`runtime/`, accepted Down/Up/Enter, loaded the model, and opened the human
session without URL or served-ID questions. Run
`9cb3522e-107b-47a5-b0a1-8d53215e90b2` created `final proof` and listed root `.`.
Independent filesystem and SQLite reads verified the directory and two
successful tool events; the run is `completed` with an unchecked freeform
answer. Ctrl+C stopped the CLI and owned model; a subsequent process inventory
found neither. The ConPTY wrapper itself reported exit 1 when interrupted,
so its exit value is not presented as Kinesin's exit code. The separate native
Windows signal/process tests verify Kinesin's exit 130 and settlement.

An earlier live run `2c15fe57-bf7f-4b5e-a0db-22fb07ff3eaa` created `local proof`
and listed root before a normal `/exit`; its owned backend was also reaped.
That run preceded the final picker/registration review fixes. The final
installed Windows binary SHA-256 is
`5338d68c8adcd14429c2feaa0521ea3310678474452a075975a0d4c1fc7e57f2`.

The runtime is the existing verified b6500 Vulkan build; Windows executable
SHA-256 `62e6a5fec588ff3ea6d26a507db4373e56c850db19b39239bd30f5e7d87751cd`.
The Qwen2.5-Coder-7B Q4_K_M GGUF SHA-256 is
`509287f78cb4d4cf6b3843734733b914b2c158e43e22a7f4bf5e963800894d3c`.
The task initially provisioned application-data locations, then discovered
MSIX redirection in the testing host. Its copied model and runtime were placed
beside the installed executable so normal PowerShell also discovers them;
the original model and existing separate runtime were preserved.

### Native Debian terminal and filesystem proof

The native installed binary SHA-256 is
`3511f9b70cdde3fbbb6384eee60aa71ea25073da6b386e7cec68f3dbb84f0d8c`.
The transferred source archive SHA-256 is
`1820a145f815cad242a48783f507cb504558c67042801f7291f4e46ee5381199`;
it includes the final Rust sources and lockfile. Later edits only finalize
documentation and replace the explicit starter's example served alias.

Actual PTY checks verified Down/Up/Enter; selector Ctrl+C returned 130 and
restored canonical input/echo without settings or state; startup Ctrl+C
returned 130, reaped the owned server, and created no journal. A ready session
created `test 1`, listed root, and `/exit` returned 0 with its server reaped
and port closed. A second session chose another workspace, listed `second.txt`,
and idle Ctrl+C returned 130 with cleanup. Saved settings were byte-identical
across both completed sessions; settings, database and controller lock modes
were 0600. Independent disk/journal checks verified three successful effects:

| Run | Verified effect |
| --- | --- |
| `e9020940-e8fa-4bad-b4c6-587be85b41ff` | Created `test 1` |
| `6fc5b2ce-df0c-47cc-abec-09d6d8d6c9f7` | Listed the first workspace root |
| `910f9f4f-b1cf-4781-a0a9-06f1bdff4231` | Listed the second workspace root |

All three are completed freeform runs with unchecked answers. The existing
separately managed server stayed healthy throughout. Because it already used
5,033 MiB of the 8-GiB GPU, the live managed session used 12 GPU layers;
combined usage reached 7,500 MiB and returned to 5,034 MiB after cleanup.
This proves local lifecycle and effects alongside another server, not default
full-offload capacity on an already occupied GPU.

### Scope and retained evidence

Managed mode supports one local CLI backend and one verified slot. Service
mode still requires an external backend. Model/runtime installation is a
one-time operator step; model downloads, remote process supervision, automatic
SSH login and backend restart are outside this implementation. External SSH
uses its normal foreground authentication prompts. Existing private workspace
restrictions continue to apply to selected model/runtime parents.

Local raw evidence is retained under ignored `target/managed-entry/`, including
platform test logs, source manifest, PTY logs and independent JSON summaries.
Public evidence uses host roles and generic paths rather than personal account,
machine or application-container names. Historical run IDs, hashes and outcomes
remain unchanged.
