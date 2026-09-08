//! Independent runtime regressions discovered after the happy-path guide build.

use std::path::PathBuf;
use std::time::Duration;

use kinesin::config::Config;
use kinesin::core::ModelReply;
use kinesin::model::{ModelClient, ScriptStep};
use kinesin::policy::Submission;
use kinesin::runner::{RunResources, admit, run_admitted};
use kinesin::storage::{Command, QueueLimits, Response, Storage};
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("kinesin-adversarial-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(path.join("workspace")).unwrap();
        Self(path)
    }
    fn config(&self, seconds: u64) -> Config {
        self.config_with_limits(seconds, "")
    }
    fn config_with_limits(&self, seconds: u64, more_limits: &str) -> Config {
        Config::parse(
            &format!(
                r#"
version = 1
instructions = "System"
[storage]
path = "state/kinesin.sqlite"
[limits]
max_run_s = {seconds}
max_model_turns = 2
{more_limits}
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
"#
            ),
            &self.0.join("kinesin.toml"),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(self.0.is_absolute() && self.0.starts_with(std::env::temp_dir()));
        assert!(
            self.0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("kinesin-adversarial-")
        );
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn inbox_capacity_expiry_settles_the_run_within_its_one_time_grace() {
    let fixture = Fixture::new();
    let config = fixture.config(1);
    let authority = config
        .authorize_local(Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: "Question".into(),
            limits: None,
            capture: None,
        })
        .unwrap();
    let owner = authority.owner().to_owned();
    let run_id = authority.run_id().to_owned();
    let path = config.storage().path.clone();
    let storage = tokio::task::spawn_blocking(move || {
        Storage::start(
            path,
            QueueLimits {
                commands: 1,
                bytes: 8 * 1024 * 1024,
                max_waiters: 4,
            },
        )
    })
    .await
    .unwrap()
    .unwrap();
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    let model = ModelClient::scripted([ScriptStep {
        delay: Duration::from_millis(200),
        reply: ModelReply::Answer("candidate".into()),
    }]);
    let running = tokio::spawn(run_admitted(
        authority,
        model.clone(),
        store.clone(),
        RunResources::single(1, config.concurrency().clone()),
        CancellationToken::new(),
        Instant::now(),
    ));
    timeout(Duration::from_secs(1), async {
        while model.captured_requests().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    // A different bounded owner reserves the only inbox slot while the model
    // is executing. Capacity returns after execution expiry, within 5s grace.
    let held = store
        .reserve(64, Instant::now() + Duration::from_secs(1))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1200)).await;
    drop(held);
    let outcome = timeout(Duration::from_secs(3), running)
        .await
        .unwrap()
        .unwrap();
    let projection = store
        .execute(
            Command::Get {
                owner_id: owner,
                run_id,
            },
            Instant::now() + Duration::from_secs(2),
        )
        .await
        .unwrap();
    let still_accepting = store.is_accepting();
    storage.shutdown().await.unwrap();
    let Response::Run(Some(record)) = projection else {
        panic!("admitted run missing");
    };
    assert_eq!(model.captured_requests().unwrap().len(), 1);
    assert_eq!(
        record.phase, "stopped",
        "outcome={outcome:?}; storage_accepting={still_accepting}"
    );
    assert_eq!(record.terminal_reason.as_deref(), Some("run_deadline"));
    assert!(record.receipt.is_some());
    assert!(outcome.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_accepted_response_size_can_be_durably_settled() {
    let fixture = Fixture::new();
    let unsafe_source = toml::to_string(&fixture.config(10)).unwrap().replace(
        "max_response_bytes = 1048576",
        "max_response_bytes = 2097152",
    );
    let rejected = Config::parse(&unsafe_source, &fixture.0.join("kinesin.toml"));
    assert!(rejected.unwrap_err().contains("max_response_bytes"));
    let config = fixture.config_with_limits(
        10,
        "max_response_bytes = 1048576\nmax_history_bytes = 2097152\n[concurrency]\njournal_queue_bytes = 2162688",
    );
    let authority = config
        .authorize_local(Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: "Question".into(),
            limits: None,
            capture: None,
        })
        .unwrap();
    let owner = authority.owner().to_owned();
    let run_id = authority.run_id().to_owned();
    let path = config.storage().path.clone();
    let bytes = config.concurrency().journal_queue_bytes;
    let storage = tokio::task::spawn_blocking(move || {
        Storage::start(
            path,
            QueueLimits {
                bytes,
                ..QueueLimits::default()
            },
        )
    })
    .await
    .unwrap()
    .unwrap();
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    // The fake counts its JSON string quotes: this fills the maximum accepted
    // response exactly and must settle using the minimum accepted journal inbox.
    let model = ModelClient::scripted([ModelReply::Answer("x".repeat(1024 * 1024 - 2)).into()]);
    let outcome = timeout(
        Duration::from_secs(5),
        run_admitted(
            authority,
            model.clone(),
            store.clone(),
            RunResources::single(1, config.concurrency().clone()),
            CancellationToken::new(),
            Instant::now(),
        ),
    )
    .await
    .unwrap();
    let projection = store
        .execute(
            Command::Get {
                owner_id: owner,
                run_id,
            },
            Instant::now() + Duration::from_secs(2),
        )
        .await
        .unwrap();
    let still_accepting = store.is_accepting();
    storage.shutdown().await.unwrap();
    let Response::Run(Some(record)) = projection else {
        panic!("admitted run missing");
    };
    assert_eq!(model.captured_requests().unwrap().len(), 1);
    assert!(
        record.receipt.is_some(),
        "accepted response has no terminal receipt: phase={}, outcome={outcome:?}, storage_accepting={still_accepting}",
        record.phase,
    );
    assert!(outcome.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_accepted_journal_byte_capacity_can_settle_a_tiny_run() {
    let fixture = Fixture::new();
    let unsafe_source = toml::to_string(&fixture.config(10)).unwrap().replace(
        "journal_queue_bytes = 8388608",
        "journal_queue_bytes = 16384",
    );
    let rejected = Config::parse(&unsafe_source, &fixture.0.join("kinesin.toml"));
    assert!(rejected.unwrap_err().contains("journal_queue_bytes"));
    let config = fixture.config_with_limits(10, "[concurrency]\njournal_queue_bytes = 2162688");
    let authority = config
        .authorize_local(Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: "Question".into(),
            limits: None,
            capture: None,
        })
        .unwrap();
    let owner = authority.owner().to_owned();
    let run_id = authority.run_id().to_owned();
    let path = config.storage().path.clone();
    let bytes = config.concurrency().journal_queue_bytes;
    let storage = tokio::task::spawn_blocking(move || {
        Storage::start(
            path,
            QueueLimits {
                bytes,
                ..QueueLimits::default()
            },
        )
    })
    .await
    .unwrap()
    .unwrap();
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    let model = ModelClient::scripted([ModelReply::Answer("tiny answer".into()).into()]);
    let outcome = run_admitted(
        authority,
        model.clone(),
        store.clone(),
        RunResources::single(1, config.concurrency().clone()),
        CancellationToken::new(),
        Instant::now(),
    )
    .await;
    let projection = store
        .execute(
            Command::Get {
                owner_id: owner,
                run_id,
            },
            Instant::now() + Duration::from_secs(2),
        )
        .await
        .unwrap();
    storage.shutdown().await.unwrap();
    let Response::Run(Some(record)) = projection else {
        panic!("admitted run missing");
    };
    assert_eq!(model.captured_requests().unwrap().len(), 1);
    assert_eq!(record.phase, "completed", "outcome={outcome:?}");
    assert!(record.receipt.is_some());
    assert!(outcome.is_ok());
}
