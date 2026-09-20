# Sprint 12 local assistant walkthrough

Implementation and native build completed before automated tests were written
or run. Windows Rust 1.96.0 `cargo build --locked --bin kinesin` succeeded.
Live evidence uses the sprint working tree based on e27caf4; final task commits
and the installed artifact identity are recorded in the final test report.

## local_assistant_walkthrough — T-111 / INT-0030

The owned b6500 llama.cpp Vulkan runtime and existing Qwen2.5-Coder-7B Q4_K_M
model started from an explicit local profile with a 4096-token window,
512 output tokens, and the personal profile's six file tools. No download or
settings change was needed. A disposable workspace was used.

1. User supplied `Cobalt Heron` as a project codename. The model answered only
   `Noted`, so retaining only the previous answer could not pass the follow-up.
2. Model called `write_file` to create `revision.txt` containing `draft`.
3. User requested an actual `edit_file` operation replacing `draft` with the
   original codename followed by `ready`. Run
   `da925547-9f5b-4b4e-915f-accf7c430130` executed that tool successfully.
4. A subsequent `read_file` and independent PowerShell filesystem read both
   returned `Cobalt Heron ready` followed by a newline (19 bytes).
5. `/context` showed four retained turns, 983/3584 bytes. `/new` followed by
   `/context` showed zero turns and zero bytes. The process exited 0, and a
   process inventory found neither Kinesin nor its owned llama-server running.
6. Exported the actual edit run; disconnected `kinesin replay` returned
   `consistent`, two model requests and one tool observation. Replay executed
   no effects and did not require the model server.

All model runs are freeform and explicitly unchecked. Disk and journal reads
establish the demonstrated effect; the model's completion text alone does not.
Local raw logs/config/capture are in ignored `target/s12-live/`.

## Preserved model-quality failure

The first variant asked the model to read and then edit `notes.txt` in the same
request. It called `read_file` only, then produced fabricated tool-response text
claiming the new content. Independent inspection found `notes.txt` unchanged
(`draft`). Run `8145ec51-29cb-486c-8810-ad7f4889e77f` retains that outcome.
The second walkthrough explicitly requested the edit tool and produced a real
edit. This demonstrates session continuity and working file tools, not reliable
arbitrary task completion by this small local model. No textual tool markup was
interpreted as an executable call.

## Installed interactive command

The checked executable replaced the existing per-user command. The old binary
is preserved in ignored `target/s12-live/kinesin-before-s12.exe`. Built and
installed SHA-256:
`751b64497a0c7bebc28c0d81c24a1029acc3fca0bb1272b46bf7eef191ce671c`.

Launched bare `kinesin` in a native Windows pseudoterminal from the disposable
workspace with isolated APPDATA/LOCALAPPDATA. Enter accepted the working folder;
the normal model picker discovered the portable GGUF and Enter started its
runtime. No explicit config/model/runtime flags were used. Existing user
settings were not modified.

The terminal session remembered `teal` while replying only `Noted`, created
`installed.txt` containing `draft`, then called `edit_file` to substitute the
color from the first request. Independent disk inspection returned `teal`.
`/context` showed 3 turns/612 bytes, then zero after `/new`. `/exit` returned 0
and no owned model remained. This used the final installed artifact after all
product edits, formatting, Clippy and 271 targeted tests.

## Scope

Native Linux/hosted CI, persistent or branched sessions, real token counting,
cache benchmarking and mixed-owner slot experiments were not run in this pass.
INT-0026 remains open. This sprint adds bounded process-local memory only.
