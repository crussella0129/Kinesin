//! Full CLI ownership of a fixture-backed managed server, including idle input.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

struct Fixture {
    root: PathBuf,
    model: PathBuf,
}
impl Fixture {
    fn new(mode: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kinesin-managed-cli-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        std::fs::create_dir(root.join("models")).unwrap();
        let model = root.join("models/fixture.gguf");
        let mut bytes = b"GGUF\x03\0\0\0".to_vec();
        bytes.extend(serde_json::to_vec(&json!({"mode":mode,"descendant":true})).unwrap());
        std::fs::write(&model, bytes).unwrap();
        let source = json!({
            "version":1,"instructions":"Answer briefly.",
            "storage":{"path":"state/kinesin.sqlite"},
            "workspaces":[{"id":"working","root":"workspace","tools":[]}],
            "models":[{"id":"local","base_url":"http://127.0.0.1:8080","model_id":"selected-file","context_size":4096,"verified_slots":1,"temperature":0.0}],
            "managed_model":{"model":"local","executable":env!("CARGO_BIN_EXE_cmd-fixture"),"model_path":model,"startup_timeout_s":10,"gpu_layers":0,"threads":2},
        });
        std::fs::write(
            root.join("kinesin.toml"),
            toml::to_string_pretty(&source).unwrap(),
        )
        .unwrap();
        Self { root, model }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_kinesin"));
        command
            .current_dir(&self.root)
            .args(["--config", "kinesin.toml"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn wait_marker(&self, extension: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.model.with_extension(extension).exists() {
            assert!(
                Instant::now() < deadline,
                "fixture did not produce {extension}"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn assert_descendant_stopped(&self) {
        let marker = self.model.with_extension("heartbeat");
        assert!(marker.exists());
        let length = std::fs::metadata(&marker).unwrap().len();
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(std::fs::metadata(marker).unwrap().len(), length);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(self.root.is_absolute() && self.root.starts_with(std::env::temp_dir()));
        assert!(
            self.root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("kinesin-managed-cli-")
        );
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.0.stdin.take();
        let deadline = Instant::now() + Duration::from_secs(2);
        while self.0.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn finish(child: &mut ChildGuard) -> (i32, String, String) {
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "managed CLI did not exit");
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    child
        .0
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    (status.code().unwrap_or(-1), stdout, stderr)
}

#[test]
fn normal_session_eof_settles_run_and_stops_managed_tree() {
    let fixture = Fixture::new("ready");
    let mut child = ChildGuard(fixture.command().spawn().unwrap());
    child.0.stdin.take().unwrap().write_all(b"Hello\n").unwrap();
    let (code, stdout, stderr) = finish(&mut child);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("managed fixture answer"), "{stdout}");
    fixture.assert_descendant_stopped();
    let store = kinesin::storage::Store::open(&fixture.root.join("state/kinesin.sqlite")).unwrap();
    drop(store);
    let database = rusqlite::Connection::open(fixture.root.join("state/kinesin.sqlite")).unwrap();
    let (phase, result): (String, String) = database
        .query_row("SELECT phase,result_json FROM runs", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert_eq!(phase, "completed");
    assert_eq!(
        serde_json::from_str::<Value>(&result).unwrap()["candidate"],
        "managed fixture answer"
    );
}

#[test]
fn managed_backend_death_exits_an_idle_session_without_waiting_for_stdin() {
    let fixture = Fixture::new("die-after-ready");
    let mut child = ChildGuard(fixture.command().spawn().unwrap());
    // Stdin remains open and no request is entered; backend death must wake the
    // session's input wait and still release storage and process-tree ownership.
    let (code, stdout, stderr) = finish(&mut child);
    assert_eq!(code, 1, "{stdout}\n{stderr}");
    assert!(
        stdout.contains("> "),
        "session must reach input before death: {stdout}"
    );
    assert!(stderr.contains("local model server exited"), "{stderr}");
    fixture.assert_descendant_stopped();
    let store = kinesin::storage::Store::open(&fixture.root.join("state/kinesin.sqlite")).unwrap();
    drop(store);
}

#[test]
fn managed_cli_ctrl_c_during_startup_stops_tree_before_any_admission() {
    #[cfg(windows)]
    if std::env::var_os("KINESIN_MANAGED_SIGNAL_CHILD").is_none() {
        hidden_console_signal_test();
        return;
    }
    #[cfg(windows)]
    {
        // This wrapper alone ignores the event. The product must restore its
        // listener; its managed child has a separate hidden process/job.
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        // SAFETY: this test runs in a separately created hidden console.
        assert_ne!(unsafe { SetConsoleCtrlHandler(None, 1) }, 0);
    }
    let fixture = Fixture::new("never-ready");
    let mut child = ChildGuard(fixture.command().spawn().unwrap());
    fixture.wait_marker("heartbeat");
    #[cfg(unix)]
    {
        // SAFETY: this PID is our directly owned, unreaped CLI child.
        assert_eq!(
            unsafe { libc::kill(child.0.id() as libc::pid_t, libc::SIGINT) },
            0
        );
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::GenerateConsoleCtrlEvent;
        // SAFETY: group zero reaches only this isolated test console.
        assert_ne!(unsafe { GenerateConsoleCtrlEvent(0, 0) }, 0);
    }
    let (code, stdout, stderr) = finish(&mut child);
    assert_eq!(code, 130, "{stdout}\n{stderr}");
    fixture.assert_descendant_stopped();
    assert!(
        !fixture.root.join("state/kinesin.sqlite").exists(),
        "no journal/run starts before managed readiness"
    );
}

#[cfg(windows)]
fn hidden_console_signal_test() {
    let logs =
        std::env::temp_dir().join(format!("kinesin-managed-signal-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&logs).unwrap();
    let output = Command::new("powershell.exe").args([
        "-NoProfile", "-NonInteractive", "-Command",
        r#"$taskSignal = Start-Process -FilePath $env:KINESIN_MANAGED_SIGNAL_EXE -ArgumentList @('--exact','managed_cli_ctrl_c_during_startup_stops_tree_before_any_admission','--nocapture') -WindowStyle Hidden -RedirectStandardOutput (Join-Path $env:KINESIN_MANAGED_SIGNAL_DIR 'stdout.log') -RedirectStandardError (Join-Path $env:KINESIN_MANAGED_SIGNAL_DIR 'stderr.log') -PassThru; if (!$taskSignal.WaitForExit(20000)) { $taskSignal.Kill(); throw 'managed signal child timed out' }; exit $taskSignal.ExitCode"#,
    ]).env("KINESIN_MANAGED_SIGNAL_CHILD", "1")
        .env("KINESIN_MANAGED_SIGNAL_EXE", std::env::current_exe().unwrap())
        .env("KINESIN_MANAGED_SIGNAL_DIR", &logs).output().unwrap();
    let stdout = std::fs::read_to_string(logs.join("stdout.log")).unwrap_or_default();
    let stderr = std::fs::read_to_string(logs.join("stderr.log")).unwrap_or_default();
    assert!(logs.is_absolute() && logs.starts_with(std::env::temp_dir()));
    assert!(
        logs.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kinesin-managed-signal-")
    );
    std::fs::remove_dir_all(logs).unwrap();
    assert!(
        output.status.success(),
        "native managed signal failed: {stdout}\n{stderr}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
