//! Native regression in an isolated hidden console, never the test runner's console.
#![cfg(windows)]

use std::future::Future;
use std::process::Command;
use std::task::Poll;
use std::time::Duration;

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
