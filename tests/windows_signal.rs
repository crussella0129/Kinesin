//! Native regression in an isolated hidden console, never the test runner's console.
#![cfg(windows)]

use std::future::Future;
use std::process::Command;
use std::task::Poll;
use std::time::Duration;

#[test]
fn windows_setup_ctrl_c_cancels_before_controller_startup() {
    if std::env::var_os("KINESIN_SETUP_SIGNAL_CHILD").is_some() {
        use windows_sys::Win32::System::Console::{
            GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
        };
        // SAFETY: this process owns an isolated hidden console, and simulates
        // the ignore attribute inherited from some Windows launchers.
        assert_ne!(unsafe { SetConsoleCtrlHandler(None, 1) }, 0);
        kinesin::signal::enable_setup_ctrl_c().unwrap();
        // SAFETY: group zero reaches only this test's isolated console.
        assert_ne!(unsafe { GenerateConsoleCtrlEvent(0, 0) }, 0);
        std::thread::sleep(Duration::from_secs(5));
        panic!("default setup cancellation was not restored");
    }
    let root = std::env::temp_dir().join(format!("kinesin-setup-signal-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let result = Command::new("powershell.exe").args([
        "-NoProfile", "-NonInteractive", "-Command",
        r#"$taskSignal = Start-Process -FilePath $env:KINESIN_SIGNAL_EXE -ArgumentList @('--exact','windows_setup_ctrl_c_cancels_before_controller_startup','--nocapture') -WindowStyle Hidden -RedirectStandardOutput (Join-Path $env:KINESIN_SIGNAL_DIR 'stdout.log') -RedirectStandardError (Join-Path $env:KINESIN_SIGNAL_DIR 'stderr.log') -PassThru -Wait; if ($taskSignal.ExitCode -ne -1073741510) { throw ('unexpected setup cancellation status: ' + $taskSignal.ExitCode) }"#,
    ])
        .env("KINESIN_SETUP_SIGNAL_CHILD", "1")
        .env("KINESIN_SIGNAL_EXE", std::env::current_exe().unwrap())
        .env("KINESIN_SIGNAL_DIR", &root)
        .output().unwrap();
    let stdout = std::fs::read_to_string(root.join("stdout.log")).unwrap_or_default();
    let stderr = std::fs::read_to_string(root.join("stderr.log")).unwrap_or_default();
    assert!(root.is_absolute() && root.starts_with(std::env::temp_dir()));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kinesin-setup-signal-")
    );
    std::fs::remove_dir_all(&root).unwrap();
    assert!(
        result.status.success(),
        "setup cancellation failed: {stdout}\n{stderr}\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn windows_ctrl_c_restores_inherited_delivery() {
    if std::env::var_os("KINESIN_SIGNAL_CHILD").is_some() {
        native_child();
        return;
    }
    let root = std::env::temp_dir().join(format!("kinesin-signal-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    // Start-Process creates a separate Windows console. Hiding that console
    // keeps the test noninteractive; CTRL_C_EVENT then reaches only this child.
    // Use the Windows inbox shell rather than requiring PowerShell 7 separately.
    let result = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"$taskSignal = Start-Process -FilePath $env:KINESIN_SIGNAL_EXE -ArgumentList @('--exact','windows_ctrl_c_restores_inherited_delivery','--nocapture') -WindowStyle Hidden -RedirectStandardOutput (Join-Path $env:KINESIN_SIGNAL_DIR 'stdout.log') -RedirectStandardError (Join-Path $env:KINESIN_SIGNAL_DIR 'stderr.log') -PassThru; if (!$taskSignal.WaitForExit(15000)) { $taskSignal.Kill(); throw 'signal child timed out' }; exit $taskSignal.ExitCode"#,
        ])
        .env("KINESIN_SIGNAL_CHILD", "1")
        .env("KINESIN_SIGNAL_EXE", std::env::current_exe().unwrap())
        .env("KINESIN_SIGNAL_DIR", &root)
        .output()
        .unwrap();
    let stdout = std::fs::read_to_string(root.join("stdout.log")).unwrap_or_default();
    let stderr = std::fs::read_to_string(root.join("stderr.log")).unwrap_or_default();
    assert!(root.is_absolute() && root.starts_with(std::env::temp_dir()));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kinesin-signal-")
    );
    std::fs::remove_dir_all(&root).unwrap();
    assert!(
        result.status.success(),
        "isolated signal failed: {stdout}\n{stderr}"
    );
}

fn native_child() {
    use windows_sys::Win32::System::Console::{GenerateConsoleCtrlEvent, SetConsoleCtrlHandler};
    // SAFETY: simulate the inherited ignore attribute in this isolated child.
    assert_ne!(unsafe { SetConsoleCtrlHandler(None, 1) }, 0);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut signal = Box::pin(kinesin::signal::ctrl_c());
        futures_util::future::poll_fn(|cx| match signal.as_mut().poll(cx) {
            Poll::Pending => Poll::Ready(()),
            Poll::Ready(result) => panic!("listener unexpectedly completed: {result:?}"),
        })
        .await;
        // SAFETY: this test process owns a separate hidden console; group zero
        // delivers only within that console, never to the parent test runner.
        assert_ne!(unsafe { GenerateConsoleCtrlEvent(0, 0) }, 0);
        tokio::time::timeout(Duration::from_secs(5), signal)
            .await
            .unwrap()
            .unwrap();
    });
}

#[test]
fn windows_session_ctrl_c_exits_after_settling() {
    if std::env::var_os("KINESIN_SESSION_SIGNAL_CHILD").is_some() {
        native_session_child();
        return;
    }
    let root =
        std::env::temp_dir().join(format!("kinesin-session-signal-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let result = Command::new("powershell.exe").args([
        "-NoProfile", "-NonInteractive", "-Command",
        r#"$taskSignal = Start-Process -FilePath $env:KINESIN_SIGNAL_EXE -ArgumentList @('--exact','windows_session_ctrl_c_exits_after_settling','--nocapture') -WindowStyle Hidden -RedirectStandardOutput (Join-Path $env:KINESIN_SIGNAL_DIR 'stdout.log') -RedirectStandardError (Join-Path $env:KINESIN_SIGNAL_DIR 'stderr.log') -PassThru; if (!$taskSignal.WaitForExit(30000)) { $taskSignal.Kill(); throw 'session signal child timed out' }; exit $taskSignal.ExitCode"#,
    ])
        .env("KINESIN_SESSION_SIGNAL_CHILD", "1")
        .env("KINESIN_SIGNAL_EXE", std::env::current_exe().unwrap())
        .env("KINESIN_SIGNAL_DIR", &root)
        .output().unwrap();
    let stdout = std::fs::read_to_string(root.join("stdout.log")).unwrap_or_default();
    let stderr = std::fs::read_to_string(root.join("stderr.log")).unwrap_or_default();
    assert!(root.is_absolute() && root.starts_with(std::env::temp_dir()));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kinesin-session-signal-")
    );
    std::fs::remove_dir_all(&root).unwrap();
    assert!(
        result.status.success(),
        "isolated session signal failed: {stdout}\n{stderr}\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn native_session_child() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::process::Stdio;
    use std::time::Instant;
    use windows_sys::Win32::System::Console::{GenerateConsoleCtrlEvent, SetConsoleCtrlHandler};

    // This wrapper ignores Ctrl+C, and the product inherits that attribute.
    // The product must restore delivery through its own listener. Both run in
    // this test's isolated console, so group zero cannot signal the test host.
    assert_ne!(unsafe { SetConsoleCtrlHandler(None, 1) }, 0);
    let root = std::path::PathBuf::from(std::env::var_os("KINESIN_SIGNAL_DIR").unwrap());
    for running in [false, true] {
        let case = root.join(if running { "running" } else { "idle" });
        std::fs::create_dir_all(case.join("workspace")).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let config = format!(
            r#"version = 1
instructions = "Answer briefly."
[storage]
path = "state/kinesin.sqlite"
[[workspaces]]
id = "working"
root = "workspace"
tools = []
[[models]]
id = "local"
base_url = "http://{}"
model_id = "fixture"
context_size = 4096
verified_slots = 1
temperature = 0.0
"#,
            listener.local_addr().unwrap()
        );
        std::fs::write(case.join("kinesin.toml"), config).unwrap();
        struct ChildGuard(std::process::Child);
        impl Drop for ChildGuard {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let mut child = ChildGuard(
            Command::new(env!("CARGO_BIN_EXE_kinesin"))
                .current_dir(&case)
                .args(["--config", "kinesin.toml"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        // Keep stdin open through cancellation; EOF must not mask a blocked
        // reader. The previous spawn_blocking implementation hangs this case.
        let mut input = child.0.stdin.take().unwrap();
        let mut stdout = child.0.stdout.take().unwrap();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let reader = std::thread::spawn(move || {
            let mut collected = Vec::new();
            let mut ready = false;
            loop {
                let mut bytes = [0; 1024];
                let count = stdout.read(&mut bytes).unwrap();
                if count == 0 {
                    break;
                }
                collected.extend_from_slice(&bytes[..count]);
                assert!(collected.len() < 65536);
                if !ready && collected.ends_with(b"> ") {
                    ready_tx.send(()).unwrap();
                    ready = true;
                }
            }
            String::from_utf8(collected).unwrap()
        });
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let mut model_connection = None;
        if running {
            input.write_all(b"Hello\n").unwrap();
            input.flush().unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            let (mut connection, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "model request did not start");
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("model accept failed: {error}"),
                }
            };
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut header = Vec::new();
            while !header.ends_with(b"\r\n\r\n") {
                assert!(header.len() < 16384);
                let mut byte = [0];
                connection.read_exact(&mut byte).unwrap();
                header.push(byte[0]);
            }
            let header = String::from_utf8(header).unwrap();
            let length = header
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().unwrap())
                })
                .unwrap();
            assert!(length < 131072);
            let mut body = vec![0; length];
            connection.read_exact(&mut body).unwrap();
            // Leave the model request pending while the product is cancelled.
            model_connection = Some(connection);
        }
        assert_ne!(unsafe { GenerateConsoleCtrlEvent(0, 0) }, 0);
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "Ctrl+C did not exit the session; running={running}"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(
            status.code(),
            Some(130),
            "Ctrl+C exit status; running={running}"
        );
        drop(input);
        drop(model_connection);
        let stdout = reader.join().unwrap();
        let mut stderr = String::new();
        child
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        assert!(stderr.is_empty(), "{stderr}");
        // Opening the store immediately proves shutdown released ownership;
        // a running request must have a terminal cancellation and receipt.
        let store = kinesin::storage::Store::open(&case.join("state/kinesin.sqlite")).unwrap();
        drop(store);
        let database = rusqlite::Connection::open(case.join("state/kinesin.sqlite")).unwrap();
        let rows: i64 = database
            .query_row("SELECT count(*) FROM runs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, i64::from(running));
        if running {
            let (phase, receipt): (String, String) = database
                .query_row("SELECT phase,acceptance_json FROM runs", [], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })
                .unwrap();
            assert_eq!(phase, "cancelled");
            assert!(
                serde_json::from_str::<serde_json::Value>(&receipt)
                    .unwrap()
                    .is_object()
            );
            assert!(stdout.contains("Run cancelled:"), "{stdout}");
        }
    }
}
