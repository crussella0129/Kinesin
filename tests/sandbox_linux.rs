//! Linux command-sandbox enforcement (INT-0012): a Landlock filesystem ruleset
//! (workspace read-write, system prefixes read-execute, deny the rest) plus a
//! seccomp filter denying network syscalls, applied to the child via `pre_exec`.
//!
//! These run where the kernel enforces Landlock+seccomp (WSL kernel 6.6, the
//! Ubuntu CI runner). The sandbox is mandatory: on a kernel that cannot enforce
//! it, `run_command` refuses with `sandbox_unavailable` rather than running
//! unconfined — so each enforcement test skips (with a note) when the kernel is
//! not capable, and `sandbox_is_mandatory` asserts the enforce-or-refuse
//! dichotomy holds on any kernel.
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

/// True when the kernel refused the command because the sandbox could not be
/// established (the mandatory refuse path).
fn refused(result: &ToolResult) -> bool {
    result.status == ToolStatus::Error
        && result
            .error
            .as_ref()
            .is_some_and(|e| e.code == "sandbox_unavailable")
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
    if refused(&result) {
        eprintln!("skip: kernel does not enforce the sandbox (mandatory refuse)");
    } else {
        assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
        assert_eq!(body(&result)["stdout"], "READ_OK");
        assert_eq!(body(&result)["exit_code"], 0);
    }
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
    if refused(&result) {
        eprintln!("skip: kernel does not enforce the sandbox (mandatory refuse)");
    } else {
        // The command runs but Landlock denies the read.
        assert_eq!(body(&result)["stdout"], "READ_DENIED");
        assert_eq!(body(&result)["exit_code"], 21);
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn sandbox_denies_network_socket() {
    let (base, ws) = scratch();
    let result = run(&ws, &["cmd-fixture", "--open-socket"]).await;
    if refused(&result) {
        eprintln!("skip: kernel does not enforce the sandbox (mandatory refuse)");
    } else {
        // seccomp denies the `socket` syscall.
        assert_eq!(body(&result)["stdout"], "SOCKET_DENIED");
        assert_eq!(body(&result)["exit_code"], 22);
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn sandbox_is_mandatory() {
    // On any kernel, a command either runs under the enforced sandbox (Ok) or is
    // refused (sandbox_unavailable) — it is never run unconfined. This covers the
    // refuse clause on kernels lacking Landlock, and the enforced path elsewhere.
    let (base, ws) = scratch();
    let result = run(&ws, &["cmd-fixture", "--print", "ok"]).await;
    if refused(&result) {
        // mandatory refusal — nothing ran.
        assert!(result.body.is_empty() || body(&result).get("stdout").is_none());
    } else {
        assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
        assert_eq!(body(&result)["stdout"], "ok");
    }
    let _ = std::fs::remove_dir_all(&base);
}
