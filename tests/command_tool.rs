//! Integration coverage for the `run_command` execution capability (INT-0003).
//!
//! These drive the real `cmd-fixture` binary (reached through
//! `CARGO_BIN_EXE_cmd-fixture`) so the process spawning, output bounding,
//! timeout tree-kill, and environment scrubbing are exercised end to end on both
//! Windows and Linux — never a shell.

use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

use kinesin::tools::{CommandRunner, MAX_COMMAND_OUTPUT_BYTES, MAX_TOOL_BYTES, ToolStatus};
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

/// A generous output budget for tests that are not exercising truncation.
const WIDE: usize = MAX_COMMAND_OUTPUT_BYTES;

#[tokio::test]
async fn command_runs_and_captures_bounded_output() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let result = runner
        .execute(
            &argv(&["cmd-fixture", "--print", "hello world"]),
            Duration::from_secs(10),
            &CancellationToken::new(),
            WIDE,
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
            WIDE,
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
            WIDE,
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
            WIDE,
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert!(result.truncated, "output beyond the cap is truncated");
    let body = body(&result);
    assert_eq!(body["truncated"], true);
    assert!(body["stdout"].as_str().unwrap().len() < MAX_TOOL_BYTES);
    assert!(result.encoded().unwrap().len() <= MAX_TOOL_BYTES);
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_output_bounded_by_configured_budget() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    // Both streams emit far more than a small configured budget.
    let result = runner
        .execute(
            &argv(&[
                "cmd-fixture",
                "--emit-stdout",
                "4000",
                "--emit-stderr",
                "4000",
            ]),
            Duration::from_secs(10),
            &CancellationToken::new(),
            512,
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert!(result.truncated);
    let body = body(&result);
    let out = body["stdout"].as_str().unwrap().len();
    let err = body["stderr"].as_str().unwrap().len();
    // The combined capture honors the configured budget, not a per-stream cap.
    assert!(out + err <= 512, "combined {out}+{err} exceeds the budget");
    assert!(result.encoded().unwrap().len() <= 512);
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
            WIDE,
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
            WIDE,
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
            WIDE,
        )
        .await;
    assert_eq!(result.status, ToolStatus::Error);
    assert_eq!(result.error.unwrap().code, "timed_out");
    // Give any surviving grandchild time to keep writing, then confirm it stopped.
    let first = std::fs::metadata(&marker).map(|m| m.len()).unwrap_or(0);
    assert!(first > 0, "the descendant probe must actually have run");
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
            WIDE,
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["stdout"], literal);
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn command_encoded_budget_covers_escaping_and_invalid_utf8() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    for (parts, budget) in [
        (
            argv(&["cmd-fixture", "--print", &"\"\\\n\t\u{0001}é".repeat(1000)]),
            256,
        ),
        (argv(&["cmd-fixture", "--emit-invalid-utf8", "4000"]), 512),
        (
            argv(&[
                "cmd-fixture",
                "--emit-stdout",
                "4000",
                "--emit-stderr",
                "4000",
            ]),
            1024,
        ),
    ] {
        let result = runner
            .execute(
                &parts,
                Duration::from_secs(10),
                &CancellationToken::new(),
                budget,
            )
            .await;
        assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
        assert!(result.encoded().unwrap().len() <= budget);
        assert!(result.truncated);
        assert_eq!(body(&result)["exit_code"], 0);
        assert_eq!(body(&result)["truncated"], true);
    }
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn already_cancelled_and_zero_deadline_commands_never_spawn() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let marker = root.join("must-not-exist");
    let command = argv(&[
        "cmd-fixture",
        "--write-file",
        marker.to_str().unwrap(),
        "spawned",
    ]);
    let cancel = CancellationToken::new();
    cancel.cancel();
    let result = runner
        .execute(&command, Duration::from_secs(10), &cancel, WIDE)
        .await;
    assert_eq!(result.error.unwrap().code, "cancelled");
    assert!(!marker.exists());
    let result = runner
        .execute(&command, Duration::ZERO, &CancellationToken::new(), WIDE)
        .await;
    assert_eq!(result.error.unwrap().code, "timed_out");
    assert!(!marker.exists());
    let _ = std::fs::remove_dir_all(root);
}

async fn assert_marker_stopped(marker: &std::path::Path) {
    let first = std::fs::metadata(marker).unwrap().len();
    assert!(first > 0, "the descendant probe must actually have run");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        first,
        std::fs::metadata(marker).unwrap().len(),
        "descendant survived cleanup"
    );
}

async fn await_marker(marker: &std::path::Path) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !std::fs::metadata(marker).is_ok_and(|m| m.len() > 0) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("descendant starts");
}

#[tokio::test]
async fn leader_exit_still_terminates_its_descendants() {
    let root = workspace();
    let runner = runner(&root, &["cmd-fixture"]);
    let marker = root.join("leader-exit-marker");
    let result = runner
        .execute(
            &argv(&[
                "cmd-fixture",
                "--spawn-grandchild",
                marker.to_str().unwrap(),
                "--wait-file",
                marker.to_str().unwrap(),
                "--exit",
                "7",
            ]),
            Duration::from_secs(10),
            &CancellationToken::new(),
            WIDE,
        )
        .await;
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert_eq!(body(&result)["exit_code"], 7);
    assert_marker_stopped(&marker).await;
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn cancellation_and_dropped_future_terminate_observed_descendants() {
    for drop_future in [false, true] {
        let root = workspace();
        let runner = runner(&root, &["cmd-fixture"]);
        let marker = root.join("cancel-marker");
        let command = argv(&[
            "cmd-fixture",
            "--spawn-grandchild",
            marker.to_str().unwrap(),
            "--sleep-ms",
            "60000",
        ]);
        let cancel = CancellationToken::new();
        let child_cancel = cancel.clone();
        let running = tokio::spawn(async move {
            runner
                .execute(&command, Duration::from_secs(60), &child_cancel, WIDE)
                .await
        });
        await_marker(&marker).await;
        if drop_future {
            running.abort();
            assert!(running.await.unwrap_err().is_cancelled());
        } else {
            cancel.cancel();
            let result = tokio::time::timeout(Duration::from_secs(5), running)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.error.unwrap().code, "cancelled");
        }
        assert_marker_stopped(&marker).await;
        let _ = std::fs::remove_dir_all(root);
    }
}
