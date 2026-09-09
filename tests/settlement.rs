//! Accepted owners survive journal waits without restarting execution.
use kinesin::{
    config::Config,
    core::ModelReply,
    model::{ModelClient, ScriptStep},
    policy::Submission,
    runner::{RunResources, admit, run_admitted},
    storage::{QueueLimits, Storage},
};
use std::time::Duration;
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;

async fn capacity_stop(cancelled: bool, beyond_grace: bool) {
    let root = std::env::temp_dir().join(format!("kinesin-settlement-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("workspace")).unwrap();
    let config = Config::parse(
        r#"
version = 1
instructions = "System"
[storage]
path = "state/kinesin.sqlite"
[limits]
max_run_s = 30
[concurrency]
journal_admission_timeout_s = 1
settlement_grace_s = 1
[[workspaces]]
id = "practice"
root = "workspace"
tools = []
[[models]]
id = "local"
base_url = "http://127.0.0.1:1"
model_id = "scripted"
context_size = 4096
verified_slots = 1
temperature = 0.0
"#,
        &root.join("config.toml"),
    )
    .unwrap();
    let authority = config
        .authorize_local(Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            continues: None,
            prompt: "Question".into(),
            limits: None,
            capture: None,
        })
        .unwrap();
    let path = config.storage().path.clone();
    let storage = tokio::task::spawn_blocking(move || {
        Storage::start(
            path,
            QueueLimits {
                commands: 1,
                bytes: 8 * 1024 * 1024,
                max_waiters: 1,
            },
        )
    })
    .await
    .unwrap()
    .unwrap();
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    let client = ModelClient::scripted([ScriptStep {
        delay: Duration::from_millis(150),
        reply: ModelReply::Answer("candidate".into()),
        usage: None,
    }]);
    let cancel = CancellationToken::new();
    let owner = tokio::spawn(run_admitted(
        authority,
        client.clone(),
        store.clone(),
        RunResources::single(1, config.concurrency().clone()),
        cancel.clone(),
        Instant::now(),
    ));
    timeout(Duration::from_secs(2), async {
        while client.captured_requests().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let held = store
        .reserve(64, Instant::now() + Duration::from_secs(1))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(250)).await;
    if cancelled {
        cancel.cancel();
    }
    let delay = if beyond_grace {
        2200
    } else if cancelled {
        150
    } else {
        1100
    };
    tokio::time::sleep(Duration::from_millis(delay)).await;
    assert!(
        !owner.is_finished(),
        "admitted owner escaped before its terminal write"
    );
    drop(held);
    let record = timeout(Duration::from_secs(3), owner)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        record.phase,
        if cancelled { "cancelled" } else { "stopped" }
    );
    assert_eq!(
        record.terminal_reason.as_deref(),
        Some(if cancelled {
            "cancelled"
        } else {
            "journal_admission_timeout"
        })
    );
    assert!(record.receipt.is_some());
    assert_eq!(client.captured_requests().unwrap().len(), 1);
    assert_eq!(store.outstanding(), (0, 0));
    storage.shutdown().await.unwrap();
    assert!(
        root.is_absolute()
            && root.starts_with(std::env::temp_dir())
            && root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("kinesin-settlement-")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_capacity_wait_finishes_without_another_effect() {
    capacity_stop(true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admission_timeout_stops_a_run_before_its_execution_deadline() {
    capacity_stop(false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expiry_of_the_one_time_grace_retains_the_same_owner() {
    capacity_stop(false, true).await;
}
