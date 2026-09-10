# Sprint 1 Research Report

## Intents Reviewed
- [INT-0003](../../../intents/INT-0003-shell-execution.md) — selected; relevance: this sprint's sole goal is the bounded command-execution tool; current state: `proposed` (transitions to `planned` at plan finalization). No revision to the intent was needed — research confirmed its boundaries hold.

## 1. Sprint Goal
Add one bounded command-execution tool — `run_command` — as the largest and last
member of Kinesin's write surface. A freeform run in a workspace the operator has
granted may run an **allow-listed executable** as an **argv vector** (never a
shell string), with bounded captured stdout/stderr, an enforced timeout, no
auto-retry, a scrubbed environment, and the workspace root as its working
directory. The effect is journalled like every other tool. Because it spawns a
process it is a distinct trust class from a file write; it is `is_mutating()` and
therefore barred from checked runs by the same authorization rule that bars file
writes. Non-goals (from the intent): an interactive shell, arbitrary shell
interpolation, and unbounded output.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| [src/tools.rs](src/tools.rs) | high | `WorkspaceReader`/`WorkspaceWriter`, `TypedToolArgs`, `ToolResult`/`ToolStatus`, byte bounds (`MAX_WRITE_BYTES`, `MAX_TOOL_BYTES`). The `WorkspaceWriter` (l.659) is the pattern to mirror: a capability built only when the grant exists, so an ungranted run has nothing to reach. |
| [src/config.rs](src/config.rs) | high | `ToolName` enum (l.88), `is_mutating()` (l.114) — the checked-run bar hinges on it — `mints_evidence()` (l.124), and `WorkspaceConfig { id, root, tools }` (l.131). The allow-list must live here. |
| [src/runner.rs](src/runner.rs) | high | Tool dispatch (l.872–1041): name→`ToolName`, `allows_tool` gate, `tool_planned`/`tool_finished` journalling, and the binary reader-vs-writer route at l.952. A process needs a *third* route and a kill-on-deadline model unlike the filesystem worker's detach-and-wait (l.979). |
| [src/policy.rs](src/policy.rs) | high | `allows_tool` (l.232) = workspace grants the tool; the checked-run mutation bar (l.355) refuses a checked workspace that grants any mutating tool. Making `run_command` mutating gives the checked-run bar for free. |
| [src/core.rs](src/core.rs) | medium | Pure ReAct loop that emits tool-call effects the runner dispatches; confirms the model never controls execution directly and no auto-retry exists to remove. |
| [tests/runner_tools.rs](tests/runner_tools.rs) | medium | The integration harness (scripted `ModelClient`, `Fixture`, `run_admitted`) the failure-mode tests will reuse. |
| [tests/cli_inspect.rs](tests/cli_inspect.rs) | medium | Real-binary E2E pattern (`CARGO_BIN_EXE_kinesin`) for asserting a command effect surfaces through `inspect`. |
| [Cargo.toml](Cargo.toml) | medium | `tokio` features (l.25) omit `process`; `cap-std = 4.0.3` (l.13). Spawning a killable child needs either the `process` feature or `std::process` in the existing `spawn_blocking` worker. |

## 3. External Sources
- [std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html) — argv-vector spawning (`.arg`/`.args`, `.env_clear`, `.current_dir`), `Child::kill`/`wait`; the no-shell primitive the tool is built on.
- [tokio::process](https://docs.rs/tokio/latest/tokio/process/index.html) — async `Child` with `.kill().await` and `Command::process_group`, which fits the runner's `tokio::select!` deadline/cancel model if the `process` feature is added.
- [command-group crate](https://docs.rs/command-group/latest/command_group/) — portable process-group / Windows Job Object kill of a whole tree; reference for the process-tree-cleanup risk rather than a mandated dependency.
- [Kinesin paper review — CodeAct](https://github.com/crussella0129/building-an-agent-harness/blob/main/paper-review.md) — the sandboxed code-interpreter alternative considered and not chosen; records why an allow-listed argv tool beats an interpreter on trust surface.

## 4. Risks, Unknowns, Dependencies
- **Risk — process-tree cleanup.** `Child::kill` on Unix kills only the direct child; an allow-listed `cargo`/`make` spawns grandchildren that can survive. Mitigate with a new process group (Unix `process_group(0)` + kill the group) and, on Windows, a Job Object; a first cut may kill the direct child and document grandchild leakage as a bounded known-limitation. This is the intent's named hardest surface.
- **Risk — execution model mismatch.** The filesystem worker detaches and waits through completion under a settlement grace (runner.rs:979) because a syscall cannot be cancelled. A process *can and must* be killed on timeout/cancel, so `run_command` needs its own dispatch branch rather than reusing the writer's detach-and-wait.
- **Risk — cap-std vs real cwd.** Process spawning needs a real `PathBuf` cwd and cannot go through the cap-std `Dir` sandbox. The command capability must hold the workspace root path, `env_clear()` the environment, set `current_dir(root)`, and refuse any `argv[0]` that is a path (only bare allow-listed basenames) so nothing escapes the workspace.
- **Unknown — allow-list shape.** Options: a per-workspace `commands = ["cargo", "git"]` list (allowed `argv[0]` basenames) vs. structured entries with fixed leading args. Recommend the simple basename list first (matches the CLI-surface readability preference); revisit if per-command argument constraints are later needed.
- **Unknown — timeout source and non-zero-exit classification.** Whether the per-command timeout is a new `Limits` field or a workspace setting, and whether a non-zero exit is `Ok` (ran, reported failure, exit code in body) vs. `Error`. Recommend: non-zero exit → `Ok` with `exit_code`; missing executable → `Error`; timeout and oversized output → defined `Error`/`Denied` with a truncation flag. Settled in the plan.
- **Dependency — `tokio` `process` feature** (or a `std::process` worker). Additive Cargo change; no external service.
- **Dependency — none on other intents.** INT-0003 is self-contained; it reuses the token-accounting-era journalling untouched.

## 5. Recommended Approach
**Primary.** Add `ToolName::RunCommand` (`is_mutating()==true`, `mints_evidence()==false`,
`as_str()=="run_command"`). Extend `WorkspaceConfig` with an executable allow-list
and validate that `run_command` is granted only alongside a non-empty allow-list.
Introduce a distinct `CommandRunner` capability (parallel to `WorkspaceWriter`, not
folded into it) that holds the workspace root `PathBuf`; it spawns the child with
`env_clear()`, `current_dir(root)`, a new process group, argv from a `command:
Vec<String>` field on `TypedToolArgs`, bounded stdout+stderr capture, an enforced
timeout that kills the process (group), and a `ToolResult` carrying exit code /
truncation / outcome. Add a third route in the runner dispatch (`name ==
RunCommand → command_runner.execute(...)`), killable via the existing
`tokio::select!` on deadline/cancel. The checked-run bar and per-workspace grant
come for free from `is_mutating()` + `allows_tool`. Journalling reuses
`tool_planned`/`tool_finished` unchanged; `inspect` surfaces the effect as it now
surfaces every tool.

**Alternative considered.** A sandboxed code interpreter / CodeAct. Rejected per the
intent and paper review: far larger dependency and security surface than an
allow-listed argv tool, for capability this sprint does not need.

**Rationale.** Mirroring the `WorkspaceWriter` design keeps the new trust class
legible and testable, isolates the one genuinely new mechanism (killable process
spawning) behind a single capability, and reuses the entire acceptance/journal/
inspect path proven in sprint 0.

## Artifacts
- No code snippets or trace files were saved; the survey references live source at the paths and line numbers above.
