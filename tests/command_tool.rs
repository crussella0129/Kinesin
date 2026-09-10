//! Integration coverage for the `run_command` execution capability (INT-0003).
//!
//! These drive the real `cmd-fixture` binary (reached through
//! `CARGO_BIN_EXE_cmd-fixture`) so the process spawning, output bounding,
//! timeout tree-kill, and environment scrubbing are exercised end to end on both
//! Windows and Linux — never a shell.

use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

use kinesin::tools::{CommandRunner, MAX_COMMAND_OUTPUT_BYTES, ToolStatus};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

/// The directory holding the fixture binary — put on PATH so the allow-listed
/// bare name `cmd-fixture` resolves. A probe secret is set to prove scrubbing.
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
        // This is an isolated integration-test process; the write happens exactly
        // once, inside Once, before any test reads the environment.
        unsafe {
            std::env::set_var("PATH", joined);
            std::env::set_var("KINESIN_TEST_SECRET", "leak");
        }
    });
}

/// A fresh workspace root, canonicalized so a child's reported cwd compares equal.
fn workspace() -> PathBuf {
    let root = std::env::temp_dir().join(format!("kinesin-cmd-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::canonicalize(&root).unwrap()
}

fn runner(root: &std::path::Path, allow: &[&str]) -> CommandRunner {
    ensure_fixture_env();
    CommandRunner::new(root, allow.iter().map(|s| s.to_string()).collect())
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

fn body(result: &kinesin::tools::ToolResult) -> Value {
    serde_json::from_str(&result.body).expect("command result body is JSON")
}

#[tokio::test]
async fn command_runs_and_captures_bounded_output() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let result = runner
        .execute(
            &argv(&["cmd-fixture", "--print", "hello world"]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert!(!result.truncated);
    let body = body(&result);
    assert_eq!(body["exit_code"], 0);
    assert_eq!(body["success"], true);
    assert_eq!(body["stdout"], "hello world");
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_runs_with_scrubbed_env_and_workspace_cwd() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    // The child's cwd is the workspace root.
    let cwd = runner
        .execute(
            &argv(&["cmd-fixture", "--print-cwd"]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(cwd.status, ToolStatus::Ok, "{:?}", cwd.error);
    let printed = PathBuf::from(body(&cwd)["stdout"].as_str().unwrap());
    assert_eq!(std::fs::canonicalize(&printed).unwrap(), root);
    // The parent's KINESIN_TEST_SECRET does not reach the scrubbed child.
    let env = runner
        .execute(
            &argv(&["cmd-fixture", "--print-env", "KINESIN_TEST_SECRET"]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(body(&env)["stdout"], "");
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_output_truncated_at_cap() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let over = MAX_COMMAND_OUTPUT_BYTES + 100;
    let result = runner
        .execute(
            &argv(&["cmd-fixture", "--emit-stdout", &over.to_string()]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert!(result.truncated, "output beyond the cap is truncated");
    let body = body(&result);
    assert_eq!(body["truncated"], true);
    assert_eq!(
        body["stdout"].as_str().unwrap().len(),
        MAX_COMMAND_OUTPUT_BYTES
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_nonzero_exit_reported() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let result = runner
        .execute(
            &argv(&["cmd-fixture", "--exit", "7"]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    // A command that ran and failed is a completed tool call, not an internal error.
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    let body = body(&result);
    assert_eq!(body["exit_code"], 7);
    assert_eq!(body["success"], false);
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_missing_executable_errors() {
    let root = workspace();
    // Allow-listed and bare, but not present on PATH.
    let runner = runner(&root, &["ghost-binary"]);
    let result = runner
        .execute(
            &argv(&["ghost-binary"]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, ToolStatus::Error);
    assert_eq!(result.error.unwrap().code, "spawn_failed");
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_timeout_kills_process_tree() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let marker = root.join("grandchild-marker");
    // The fixture spawns a grandchild that appends to the marker every 50ms, then
    // sleeps far past the timeout; killing the group must stop the grandchild too.
    let result = runner
        .execute(
            &argv(&[
                "cmd-fixture",
                "--spawn-grandchild",
                marker.to_str().unwrap(),
                "--sleep-ms",
                "60000",
            ]),
            Duration::from_millis(700),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, ToolStatus::Error);
    assert_eq!(result.error.unwrap().code, "timed_out");
    // Give any surviving grandchild time to keep writing, then confirm it stopped.
    let first = std::fs::metadata(&marker).map(|m| m.len()).unwrap_or(0);
    std::thread::sleep(Duration::from_millis(600));
    let second = std::fs::metadata(&marker).map(|m| m.len()).unwrap_or(0);
    assert_eq!(
        first, second,
        "the grandchild kept running after the group kill"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn run_command_passes_argv_without_shell_interpretation() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    // Shell metacharacters must reach the program as one literal argument.
    let literal = "a; rm -rf b && c | d > e";
    let result = runner
        .execute(
            &argv(&["cmd-fixture", "--print", literal]),
            Duration::from_secs(10),
            &CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], literal);
    let _ = std::fs::remove_dir_all(&root);
}
