//! Linux command-sandbox enforcement (INT-0012): a Landlock filesystem ruleset
//! (workspace read-write, system prefixes read-execute, deny the rest) plus a
//! seccomp filter denying network syscalls, applied to the child via `pre_exec`.
//!
//! These run where the kernel enforces Landlock+seccomp (WSL kernel 6.6, the
//! Ubuntu CI runner). The sandbox is mandatory: on a kernel that cannot enforce
//! it, `run_command` refuses with `sandbox_unavailable` rather than running
//! unconfined. This suite requires actual enforcement: an unsupported kernel is
//! a test failure, not a passing skip. Forced setup-failure tests separately
//! establish fail-closed behavior in the tools unit suite.
#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::Duration;

use kinesin::tools::{CommandRunner, MAX_COMMAND_OUTPUT_BYTES, ToolResult, ToolStatus};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

fn ensure_fixture_env() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = PathBuf::from(env!("CARGO_BIN_EXE_cmd-fixture"))
            .parent()
            .expect("fixture dir")
            .to_path_buf();
        let mut paths = vec![dir];
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        let joined = std::env::join_paths(paths).expect("join PATH");
        unsafe { std::env::set_var("PATH", joined) };
    });
}

/// A fresh temp base with a `ws/` workspace subdir (canonicalized) and room for
/// an out-of-workspace file beside it.
fn scratch() -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("kinesin-sandbox-{}", uuid::Uuid::new_v4()));
    let ws = base.join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    let ws = ws.canonicalize().unwrap();
    (base, ws)
}

fn runner(root: &Path) -> CommandRunner {
    ensure_fixture_env();
    CommandRunner::new(root, vec!["cmd-fixture".to_string()])
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

fn body(result: &ToolResult) -> Value {
    serde_json::from_str(&result.body).expect("command result body is JSON")
}

async fn run(root: &Path, parts: &[&str]) -> ToolResult {
    runner(root)
        .execute(
            &argv(parts),
            Duration::from_secs(10),
            &CancellationToken::new(),
            MAX_COMMAND_OUTPUT_BYTES,
        )
        .await
}

#[tokio::test]
async fn sandbox_allows_in_workspace_work() {
    let (base, ws) = scratch();
    let inside = ws.join("inside.txt");
    std::fs::write(&inside, b"ok").unwrap();
    let result = run(
        &ws,
        &["cmd-fixture", "--read-file", inside.to_str().unwrap()],
    )
    .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], "READ_OK");
    assert_eq!(body(&result)["exit_code"], 0);
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn sandbox_denies_out_of_workspace_read() {
    let (base, ws) = scratch();
    // A secret beside the workspace (under /tmp, not a granted prefix).
    let secret = base.join("secret.txt");
    std::fs::write(&secret, b"top secret").unwrap();
    let secret = secret.canonicalize().unwrap();
    let result = run(
        &ws,
        &["cmd-fixture", "--read-file", secret.to_str().unwrap()],
    )
    .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], "READ_DENIED");
    assert_eq!(body(&result)["exit_code"], 21);
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn sandbox_denies_network_socket() {
    let (base, ws) = scratch();
    let result = run(&ws, &["cmd-fixture", "--open-socket"]).await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], "SOCKET_DENIED");
    assert_eq!(body(&result)["exit_code"], 22);
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn sandbox_is_actually_available_on_the_test_platform() {
    let (base, ws) = scratch();
    let result = run(&ws, &["cmd-fixture", "--print", "ok"]).await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], "ok");
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn sandbox_denies_outside_truncate_but_allows_workspace_write_and_rename() {
    let (base, ws) = scratch();
    let outside = base.join("private-data");
    std::fs::write(&outside, "preserve").unwrap();
    let result = run(
        &ws,
        &["cmd-fixture", "--truncate-file", outside.to_str().unwrap()],
    )
    .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], "TRUNCATE_DENIED\n");
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "preserve");
    std::fs::create_dir(ws.join("nested")).unwrap();
    let result = run(
        &ws,
        &[
            "cmd-fixture",
            "--write-file",
            "inside",
            "original",
            "--truncate-file",
            "inside",
            "--rename-file",
            "inside",
            "nested/moved",
        ],
    )
    .await;
    assert_eq!(body(&result)["exit_code"], 0);
    assert_eq!(std::fs::read(ws.join("nested/moved")).unwrap(), b"");
    assert!(!ws.join("inside").exists());
    let _ = std::fs::remove_dir_all(base);
}

#[tokio::test]
async fn sandbox_denies_executable_siblings_and_proc() {
    let (base, ws) = scratch();
    let executable_dir = PathBuf::from(env!("CARGO_BIN_EXE_cmd-fixture"))
        .parent()
        .unwrap()
        .to_path_buf();
    let secret = executable_dir.join(format!("private-probe-{}", uuid::Uuid::new_v4()));
    std::fs::write(&secret, "synthetic secret").unwrap();
    for path in [secret.as_path(), Path::new("/proc/version")] {
        let result = run(&ws, &["cmd-fixture", "--read-file", path.to_str().unwrap()]).await;
        assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
        assert_eq!(body(&result)["stdout"], "READ_DENIED");
    }
    std::fs::remove_file(secret).unwrap();
    let _ = std::fs::remove_dir_all(base);
}

#[tokio::test]
async fn sandbox_denies_io_uring_and_process_group_escape() {
    let (base, ws) = scratch();
    for name in [
        "uring-setup",
        "uring-enter",
        "uring-register",
        "setpgid",
        "unshare",
        "setns",
        "clone3",
    ] {
        let result = run(&ws, &["cmd-fixture", "--probe-syscall", name]).await;
        assert_eq!(body(&result)["stdout"], "SYSCALL_DENIED\n", "{name}");
        assert_eq!(body(&result)["exit_code"], 0, "{name}");
    }
    let result = run(&ws, &["cmd-fixture", "--spawn-session-probe"]).await;
    assert_eq!(body(&result)["stdout"], "SYSCALL_DENIED\n");
    assert_eq!(body(&result)["exit_code"], 0);
    let result = run(
        &ws,
        &["cmd-fixture", "--probe-namespace-clone", "--spawn-thread"],
    )
    .await;
    assert_eq!(body(&result)["stdout"], "SYSCALL_DENIED\nTHREAD_OK\n");
    assert_eq!(body(&result)["exit_code"], 0);
    let _ = std::fs::remove_dir_all(base);
}

#[tokio::test]
async fn sandbox_closes_inherited_non_stdio_descriptors() {
    use std::os::fd::AsRawFd;
    let (base, ws) = scratch();
    let secret = base.join("private-descriptor");
    std::fs::write(&secret, "synthetic").unwrap();
    let file = std::fs::File::open(secret).unwrap();
    // SAFETY: file owns this valid descriptor throughout the child execution.
    // Clear CLOEXEC only on this test-owned synthetic file, not an application FD.
    assert_eq!(
        unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, 0) },
        0
    );
    let result = run(
        &ws,
        &["cmd-fixture", "--read-fd", &file.as_raw_fd().to_string()],
    )
    .await;
    assert_eq!(body(&result)["stdout"], "FD_READ_DENIED\n");
    drop(file);
    let _ = std::fs::remove_dir_all(base);
}
