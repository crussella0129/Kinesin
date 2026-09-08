# Native Windows console shutdown validation

Date: 2026-09-08. Profile: `service-validation/shutdown.toml`, loopback service
8093 and deterministic delayed HTTP model 8083. No bearer tokens are recorded
here. The local Qwen generation proof is a separate test.

The original apparent hang was a Windows launch setting: the process inherited
the Ctrl+C ignore attribute. A tiny Tokio-only child reproduced it. Registering
Tokio's listener and then calling `SetConsoleCtrlHandler(NULL,FALSE)` made native
Ctrl+C resume the future and exit. The earlier claim that the listener had closed
was an observation error: `Get-NetTCPConnection` returned no rows, while an actual
HTTP request still received 401.

The helper attaches only to each test's newly created hidden console, ignores
Ctrl+C in itself, emits `GenerateConsoleCtrlEvent(CTRL_C_EVENT,0)`, and detaches.
It never sends a console event to a shared user console.

After the fix:

- Idle service PID22920: `/health` returned 200 and `{"alive":true}`; native
  Ctrl+C closed admission, joined ingress, readiness monitor, signal task,
  controller, and journal, dropped the runtime, and exited in under one second.
  See `service-validation/shutdown-fixed.stderr.log`.
- Active service PID29128: fake model recorded `generation_started` before
  Ctrl+C; service completed every join and exited. Run
  `d4fa4229-617f-472d-96dc-a1a079b83660` was inspected read-only in SQLite and was
  `cancelled`, `unchecked`, with terminal reason `cancelled` and `run_finished`
  sequence 4. The fake observed the generation connection close. See
  `service-validation/shutdown-active.stderr.log` and accepted JSON.
- Active CLI: after observing a new actual generation request, the launcher
  delivered native Ctrl+C and waited for the child. Its **OS process exit code
  was 130**, and the emitted run JSON was cancelled/unaccepted. Run
  `b2357271-0a22-4172-af22-69019b231a81` was confirmed durably cancelled with
  `run_finished` sequence 4. See `service-validation/shutdown-cli.exit` and
  `shutdown-cli-exit.stdout.log`.
- `cargo test --locked --test windows_signal`: passed. This committed regression
  creates a separate hidden console, explicitly sets Ctrl+C ignored, registers
  the production listener, generates a native event, and requires child exit.
- `cargo test --locked --test service`: 12 passed, including public liveness when
  models are unready or credentials disabled, with readiness and run routes
  retaining authentication.
- `cargo test --locked --lib operator::tests`: 3 passed, covering monitor fault
  supervision and owned shutdown of network waits and blocking credential I/O.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.

All helper, model fixture, and service processes owned by this shutdown diagnosis
have exited or were stopped after verifying their exact executable paths. The
first CLI exit-code measurement attempts outlasted or exhausted the finite model
fixture and returned ordinary model errors; the final measurement synchronizes
the signal directly to the observed request and records the actual exit code.
