# Interactive entry repair

## T-108: Interactive entry and workspace operations

Intent: [INT-0029](intents/INT-0029-interactive-entry-and-workspaces.md).
This is the user's direct follow-up to completed sprint 11. The earlier requested
documentation was included in PR #11, which merged on 2026-09-12 at 18:45:03Z.
The subsequent usability repair requires a follow-up PR; it does not reopen or
relabel that sprint's historical test results.

### Observed failure

The Windows run `01b088a9-837b-4132-a6b0-b00ccc1c975b` produced a folder refusal
without a tool call. Its configuration granted only read/list tools. Run
`1bb5d113-9cf7-4879-80b4-837d302caef7` did execute list_files successfully at `.`
inside the configured example workspace. The malformed final backtick was in
the model candidate itself, not a truncated transport response. Both ordinary
freeform runs were correctly recorded as unchecked, but exposing full JSON made
that metadata look like a failure and concealed the working directory.

The user also reported that Ctrl+C required closing PowerShell. Session input
used Tokio's blocking pool; shutdown waited indefinitely for that console read
to complete even after cancellation had settled the controller.

### Repair and verification plan

1. Install one callable product binary and introduce a per-user first-run profile
   plus a visible folder choice for terminal sessions. Preserve explicit config
   and automation behavior. Test path resolution, persistence and no replacement.
2. Add a native create_directory tool through existing grant, writer, journal and
   replay routes. Test spaces, existing destinations, no-follow parent traversal,
   outside paths, missing grants and checked-run mutation refusal.
3. Display human answers and bounded tool activity in sessions; retain JSON for
   explicit automation and operator commands. Verify prompt ordering, errors,
   streaming presentation and escaping of terminal control sequences. Exercise
   native Ctrl+C while idle and during a request, including durable cancellation
   and resource cleanup before exit.
4. Exercise installed entry in actual terminal sessions, choose a task-owned
   directory, ask to create a spaced-name folder and list it, then independently
   verify filesystem results and persisted events. Reuse the authorized Nighthawk
   model. Do not describe a greeting or piped-only process test as this proof.
5. Update the README and Windows/Linux guide around normal interactive use,
   retaining explicit examples for checked/automated runs. Run affected tests,
   format/clippy, dependency checks and final CI; record review and limitations.

### Execution status

Implementation, native platform verification and independent review completed.
Required CI for the submitted commit is reported on the follow-up PR. The earlier
sprint 11 test report does not establish these new acceptance criteria.

### Implemented behavior

- Product-only installation exposes `kinesin` on PATH. Bare terminal startup
  introduces the program, chooses an existing folder and creates a private
  personal model/file-tool profile on first use. Subsequent selection changes
  only the in-memory workspace. Explicit/JSON/piped setup rules remain distinct.
- The new native directory tool uses frozen grants and the existing writer,
  durable effect and replay routes. Parent capabilities open without following
  links; existing destinations are never replaced. It grants no shell authority.
- Human sessions show capabilities, bounded tool activity, readable answers and
  slash commands. JSON remains explicit. Terminal controls are escaped, input is
  bounded, and final answers remain readable if activity interrupts streaming.
- A dedicated read-only input thread replaces Tokio's blocking stdin task.
  Cancellation settles the controller and storage without waiting for another
  keyboard entry. Setup restores Windows' default Ctrl+C delivery before any
  controller or journal exists.

### Windows terminal evidence

Executed the installed command from `C:\Users\charl`, outside the checkout, in
Windows PowerShell `5.1.26100.9444` with `Console.IsInputRedirected = false`.
First use chose the task-owned `validation-output/interactive-first-use/project-a`
folder and accepted the running model's port-8080 tunnel and served alias.

| Request | Run | Durable operation and independent result |
| --- | --- | --- |
| `make folder called test 1` | `c24c5f00-d3e5-4640-b8e3-4b24e2c741ec` | `create_directory test 1`, executed/ok; the directory exists in project-a. |
| `list the files in this folder` | `18511e33-d51c-4187-bda7-35e532e322b8` | The model interpreted the reference as the new `test 1` folder; `list_files test 1` correctly reported it empty. |
| `list the files and folders at the workspace root` | `30f61122-08bb-494f-8639-7afca7499d56` | `list_files .`, executed/ok; the answer names the actual `test 1` directory. |
| Same explicit root listing after a second launch selecting project-b | `6d880ffb-4bfd-46cf-bc93-6117a37052db` | `list_files .`, executed/ok; answer and independent directory enumeration both confirm project-b is empty. |

All four records are completed/unchecked, as expected for freeform work. Read-only
SQLite inspection confirmed the executed tool resources and successful results.
`/status` and `/permissions` displayed correctly, and `/exit` returned 0. The
personal settings hash stayed
`6bf4841f9eaae6a6956b66ec8b63aaff87ef66fa487b4fb112bf68f13248854b`
across folder selections. New private Windows directories have protected DACLs
for charl, SYSTEM and Administrators; the TOML inherits only those grants.

Actual native isolated-console tests send CTRL_C_EVENT while idle and while a
model request is stalled, require exit 130 within five seconds, reopen storage
and verify the cancelled request's durable record. A separate setup test requires
Windows' native `0xC000013A` status. A manual Ctrl+C through the tool's PowerShell
PTY also exited promptly, but interrupted the enclosing test script with status
1; it is not used as proof of the product's precise 130 status.

After the final installation, repeated the exact creation request in project-b:
run `3d98f1b2-d744-4b36-bbaf-dd2e11e065a2` created its `test 1` directory, confirmed
on disk; run `c0c23a4f-5b51-46d4-b2cf-f9a33e19e7b5` listed that directory at the
workspace root. Both completed, showed successful tool activity and readable
answers, and `/exit` returned 0. The earlier empty project-b observation therefore
precedes this deliberate final verification write.

### Linux first-use defect discovered during verification

The initial native Debian build passed 342 tests, format and all-target Clippy.
An actual PTY first-use session created and listed `test 1`, but relaunch exposed
an untested default-umask defect: SQLite's database and controller lock were
created as 0644 under umask 022. Personal setup then correctly rejected its own
state directory as permissive. This attempt is recorded as failed; it does not
count as a successful Linux first-use/relaunch proof. The repair creates new
Unix lock/database files with mode 0600 before SQLite opens them; live WAL/SHM
files then inherit that private mode. New exports/backups also use 0600, so a
supported export does not make later personal startup reject its state directory.
Existing files, contents and modes are never silently repaired or replaced.

### Final native Linux proof

On Debian Nighthawk, installed only the product under the task-owned
`kinesin-s11-validation/interactive-repair/install` prefix and invoked `kinesin`
by name through PATH in actual PTYs outside the source checkout. XDG settings
and state were isolated under `cases-final`; the normal user PATH/profile was
not modified. The existing user model on loopback 18080 remained running.

| Operation | Run | Result |
| --- | --- | --- |
| Create `test 1` | `2a508043-9c97-4577-874e-19789c41b8f8` | Executed/ok and independently present in workspace-one. |
| Natural follow-up listing | `0379bd5f-6743-46b7-95b3-e947fcd77bb6` | Model selected the new subfolder; actual listing returned empty. |
| Explicit root `.` listing | `3b35f591-f7f7-41fd-ab77-0d750ba11ae9` | Listed actual `project.txt` and `test 1`. |
| Second launch/second workspace | `a2e50fcf-e8e0-4a71-b78d-b493d71abef0` | Listed only `second-folder.txt` from the newly selected workspace. |
| Ctrl+C during a stalled request | `81015168-1e4d-444c-8bc5-90be9b15ce5d` | Exited 130 within five seconds; durable cancelled record and receipt. |

Both ordinary sessions exited 0; idle Ctrl+C also exited 130 within five seconds.
The live database, lock, WAL and SHM files were each 0600. The personal settings
hash remained `4553e25fe8753942cfb6fe6dc76d0ef162826b254c87b3c8bac447f18c09556e`
across folder choices. An isolated-process regression repeats setup, live storage,
shutdown and reopen under umask 022, then verifies existing 0640 files stay
unchanged and are rejected by the private-tree check.

### Verification summary and provenance

- Windows: full `cargo test --locked` passed 331 tests; after the final Unix
  export/backup permission change the affected CLI suite passed all 15 again.
  Final format and all-target/all-feature Clippy with warnings denied passed.
- Native Debian: final full suite passed 343 tests, plus format and the same
  all-target/all-feature Clippy command. Both suites report nine ignored entries;
  isolated workers are invoked by parent tests, while explicit live/stress cases
  require separate invocation. The real PTY/model checks above are additional.
- Directory tests cover spaces, existing paths/content, missing parents, denied
  grants, checked mutations, traversal and actual symlink parents. The runner
  records the real effect; replay remains consistent without recreating it.
- `cargo deny check` passed advisories, licenses, bans and sources. `cargo audit`
  scanned 250 locked dependencies against 1,243 advisories with no vulnerability.
- Final runtime sources, tests and Cargo files matched the Nighthawk source
  manifest; transferred archive SHA256:
  `c3668d2cf5f1f890c4262c5633ccca2f81260eec7f468b7ddc3dfb280c1bf80e`.
  Later Book/documentation reconciliation does not change those runtime sources.
- Final installed debug binaries: Windows
  `a202cdb15018794215460a7ad7684cb017082b41c0fc5181b303bcda8a985e1b`;
  Linux `8e3f9c3338275bec61ee903cad4c30c19d7824e1b38d0dc442645236996f574d`.
  Windows installation is in the actual user's Cargo bin directory. The final
  installed Windows binary re-opened the folder chooser and saved profile in
  PowerShell 5.1 and completed `/permissions` and `/exit` with status 0.

Raw logs, manifests and read-only journal summaries are retained under ignored
`target/interactive-repair/` and `target/interactive-windows-*`. Original failed
Linux cases remain intact. The configured Nighthawk model and Windows tunnel
remain available; these tests did not replace user workspaces or their files.

### Independent review

Separate reviews covered directory authority/replay, onboarding/private files,
CLI lifecycle and documentation. Fixed findings include raw path control
characters, unbounded console input, inherited setup Ctrl+C suppression,
interleaved provisional display, Nighthawk's personal endpoint instructions,
session exit-code wording and the setup/native cancellation distinction.

The requested entry and workspace behavior is narrower than complete Codex CLI
parity. It uses the existing Qwen2.5-Coder 7B model, which can resolve ambiguous
references differently from the operator and can emit an unclosed Markdown
backtick even after a successful tool operation. Model prose is preserved rather
than silently rewritten into a stronger result. Follow-ups carry the preceding answer,
not full conversation history. Model serving remains separate. Installed proof
uses debug builds; it does not claim a new release packaging/signing pipeline.
