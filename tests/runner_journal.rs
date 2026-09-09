//! Offline integration proofs through the real authority, runner and SQLite
//! boundaries. Scripted replies control failure timing; they prove no model
//! quality, remote cancellation or power-loss durability claims.

use kinesin::config::{CaptureMode, Config};
use kinesin::core::{ModelReply, ToolCall};
use kinesin::model::{ModelClient, ScriptStep, fingerprint};
use kinesin::policy::{RunAuthority, Submission};
use kinesin::runner::{RunResources, admit, run_admitted};
use kinesin::storage::{
    Command, Event, QueueLimits, Response, RunRecord, Storage, Store, candidate_digest,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;

const CONFIG: &str = r#"
version = 1
instructions = "private instruction sentinel"
[storage]
path = "state/kinesin.sqlite"
capture = "metadata"
[limits]
max_model_turns = 2
max_run_s = 10
[[workspaces]]
id = "practice"
root = "workspace"
tools = []
[[models]]
id = "local"
base_url = "http://127.0.0.1:8080"
model_id = "scripted-fixture"
context_size = 4096
verified_slots = 1
temperature = 0.0
"#;

struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("kinesin-runner-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        Self { root }
    }
    fn parse(&self, source: &str) -> Result<Config, String> {
        Config::parse(source, &self.root.join("kinesin.toml"))
    }
    fn path(&self) -> PathBuf {
        self.root.join("state/kinesin.sqlite")
    }
    async fn storage(&self) -> Storage {
        let path = self.path();
        tokio::task::spawn_blocking(move || Storage::start(path, QueueLimits::default()))
            .await
            .unwrap()
            .unwrap()
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
                .starts_with("kinesin-runner-")
        );
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn authority(config: &Config, capture: CaptureMode) -> RunAuthority {
    config
        .authorize_local(Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            continues: None,
            prompt: "private prompt sentinel".into(),
            limits: None,
            capture: Some(capture),
        })
        .unwrap()
}
fn events_response(response: Response) -> Vec<Event> {
    match response {
        Response::Events(events) => events,
        _ => panic!("expected event page"),
    }
}
fn run_response(response: Response) -> RunRecord {
    match response {
        Response::Run(Some(record)) => *record,
        _ => panic!("expected retained run"),
    }
}
fn assert_one_model_pair(events: &[Event]) {
    assert_eq!(
        events.iter().map(|e| e.seq).collect::<Vec<_>>(),
        (0..events.len() as u64).collect::<Vec<_>>()
    );
    let planned = events
        .iter()
        .filter(|e| e.kind == "model_planned")
        .collect::<Vec<_>>();
    let finished = events
        .iter()
        .filter(|e| e.kind == "model_finished")
        .collect::<Vec<_>>();
    assert_eq!(planned.len(), 1);
    assert_eq!(finished.len(), 1);
    assert!(planned[0].seq < finished[0].seq);
    assert_eq!(planned[0].data["effect_id"], finished[0].data["effect_id"]);
    assert_eq!(events.last().unwrap().kind, "run_finished");
}

struct Case {
    fixture: Fixture,
    record: RunRecord,
    events: Vec<Event>,
    requests: Vec<Vec<u8>>,
}

async fn run_case(reply: ModelReply, capture: CaptureMode) -> Case {
    let fixture = Fixture::new();
    let config = fixture.parse(CONFIG).unwrap();
    let authority = authority(&config, capture);
    let owner = authority.owner().to_owned();
    let run = authority.run_id().to_owned();
    let storage = fixture.storage().await;
    let store = storage.client();
    let client = ModelClient::scripted([
        reply.into(),
        ModelReply::Answer("forbidden retry".into()).into(),
    ]);
    let resources = RunResources::single(1, config.concurrency().clone());
    let acceptance = admit(&authority, &store, None).await;
    let outcome = match acceptance {
        Ok(_) => {
            run_admitted(
                authority,
                client.clone(),
                store.clone(),
                resources.clone(),
                CancellationToken::new(),
                Instant::now(),
            )
            .await
        }
        Err(problem) => Err(problem),
    };
    let events = store
        .execute(
            Command::Events {
                owner_id: owner,
                run_id: run,
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await;
    let shutdown = storage.shutdown().await;
    shutdown.unwrap();
    assert_eq!(
        resources.models.available_permits(),
        1,
        "model permit must return after observed work and journal settlement"
    );
    Case {
        fixture,
        record: outcome.expect("run settles a durable terminal record"),
        events: events_response(events.unwrap()),
        requests: client.captured_requests().unwrap(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn completed_freeform_retains_exact_candidate_receipt_and_prepared_fingerprint() {
    let candidate = "  bounded final answer\n";
    let case = run_case(ModelReply::Answer(candidate.into()), CaptureMode::Metadata).await;
    assert_eq!(case.record.phase, "completed");
    assert_eq!(case.record.acceptance_status, "unchecked");
    assert!(!case.record.task_accepted());
    assert_eq!(case.record.result.as_ref().unwrap()["candidate"], candidate);
    assert_eq!(
        case.record.result_sha256.as_deref(),
        Some(candidate_digest(candidate).as_str())
    );
    assert_eq!(case.record.receipt.as_ref().unwrap()["status"], "unchecked");
    assert_eq!(
        case.record.receipt.as_ref().unwrap()["candidate_sha256"],
        json!(case.record.result_sha256)
    );
    assert_one_model_pair(&case.events);
    assert_eq!(case.requests.len(), 1);
    let planned = case
        .events
        .iter()
        .find(|event| event.kind == "model_planned")
        .unwrap();
    assert_eq!(
        planned.data["request_sha256"],
        fingerprint(&case.requests[0])
    );
    let request: Value = serde_json::from_slice(&case.requests[0]).unwrap();
    assert_eq!(request["messages"][0]["role"], "system");
    assert_eq!(request["messages"][1]["content"], "private prompt sentinel");
    assert_eq!(
        case.events.last().unwrap().data["receipt"],
        json!(case.record.receipt)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn model_failures_empty_answers_and_length_stop_with_one_recorded_attempt() {
    for (reply, phase, reason) in [
        (
            ModelReply::Failure("model_connection_failed".into()),
            "failed",
            "model_connection_failed",
        ),
        (
            ModelReply::Failure("protocol_finish_reason".into()),
            "failed",
            "protocol_finish_reason",
        ),
        (ModelReply::Answer(" \n".into()), "failed", "empty_response"),
        (
            ModelReply::Incomplete("generation_length".into()),
            "stopped",
            "generation_length",
        ),
        (
            ModelReply::ToolCalls {
                content: None,
                calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "read_file".into(),
                    arguments: "{\"path\":\"project.txt\"}".into(),
                }],
            },
            "failed",
            "unexpected_tool_call",
        ),
    ] {
        let case = run_case(reply, CaptureMode::Metadata).await;
        assert_eq!(case.record.phase, phase);
        assert_eq!(case.record.terminal_reason.as_deref(), Some(reason));
        assert_eq!(case.record.acceptance_status, "unchecked");
        assert!(!case.record.task_accepted());
        assert_one_model_pair(&case.events);
        assert_eq!(
            case.requests.len(),
            1,
            "failure must not consume the second scripted reply"
        );
        assert!(case.events.iter().all(|e| !e.kind.starts_with("tool_")));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metadata_omits_private_inputs_and_replay_retains_each_observation_once() {
    for capture in [CaptureMode::Metadata, CaptureMode::Replay] {
        let case = run_case(
            ModelReply::Answer("observed answer sentinel".into()),
            capture,
        )
        .await;
        let serialized = serde_json::to_string(&case.events).unwrap();
        let expected = usize::from(capture == CaptureMode::Replay);
        assert_eq!(
            serialized.matches("private instruction sentinel").count(),
            expected
        );
        assert_eq!(
            serialized.matches("private prompt sentinel").count(),
            expected
        );
        assert_eq!(
            serialized.matches("observed answer sentinel").count(),
            expected
        );
        assert_eq!(
            case.record.result.unwrap()["candidate"],
            "observed answer sentinel"
        );
        let observation = case
            .events
            .iter()
            .find(|e| e.kind == "model_finished")
            .unwrap();
        if capture == CaptureMode::Replay {
            assert_eq!(observation.data["replay"]["kind"], "answer");
            assert_eq!(
                observation.data["replay"]["text"],
                "observed answer sentinel"
            );
            assert!(
                case.events
                    .iter()
                    .filter(|e| e.kind == "model_planned")
                    .all(|e| e.data.get("replay").is_none())
            );
        } else {
            assert!(case.events.iter().all(|e| e.data.get("replay").is_none()));
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_admission_means_no_model_effect_or_accidental_run_creation() {
    let fixture = Fixture::new();
    let config = fixture.parse(CONFIG).unwrap();
    let authority = authority(&config, CaptureMode::Metadata);
    let owner = authority.owner().to_owned();
    let run = authority.run_id().to_owned();
    let storage = fixture.storage().await;
    let store = storage.client();
    let client = ModelClient::scripted([ModelReply::Answer("must not run".into()).into()]);
    let outcome = run_admitted(
        authority,
        client.clone(),
        store.clone(),
        RunResources::single(1, config.concurrency().clone()),
        CancellationToken::new(),
        Instant::now(),
    )
    .await;
    let retained = store
        .execute(
            Command::Get {
                owner_id: owner,
                run_id: run,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await;
    storage.shutdown().await.unwrap();
    assert!(outcome.is_err());
    assert!(client.captured_requests().unwrap().is_empty());
    assert!(matches!(retained.unwrap(), Response::Run(None)));
}

#[test]
fn malformed_configuration_and_unauthorized_aliases_cannot_construct_run_authority() {
    let fixture = Fixture::new();
    assert!(
        fixture
            .parse(&CONFIG.replace("max_run_s = 10", "max_run_s = 0"))
            .is_err()
    );
    assert!(
        fixture
            .parse(&format!("{CONFIG}\nunknown_setting = true\n"))
            .is_err()
    );
    let config = fixture.parse(CONFIG).unwrap();
    assert!(
        config
            .authorize_local(Submission::Freeform {
                workspace: "unknown".into(),
                model: "local".into(),
                continues: None,
                prompt: "private prompt".into(),
                limits: None,
                capture: None
            })
            .is_err()
    );
    assert!(
        !fixture.path().exists(),
        "configuration/authority failures must precede opening the run journal"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn intent_commit_failure_prevents_the_model_request() {
    let fixture = Fixture::new();
    let config = fixture.parse(CONFIG).unwrap();
    let authority = authority(&config, CaptureMode::Metadata);
    let owner = authority.owner().to_owned();
    let run = authority.run_id().to_owned();
    let storage = fixture.storage().await;
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    let path = fixture.path();
    tokio::task::spawn_blocking(move || {
        let connection=rusqlite::Connection::open(path).unwrap();
        connection.execute_batch("CREATE TRIGGER reject_intent BEFORE INSERT ON events WHEN NEW.kind='model_planned' BEGIN SELECT RAISE(ABORT,'synthetic writer failure'); END;").unwrap();
    }).await.unwrap();
    let client = ModelClient::scripted([ModelReply::Answer("must not run".into()).into()]);
    let resources = RunResources::single(1, config.concurrency().clone());
    let outcome = run_admitted(
        authority,
        client.clone(),
        store.clone(),
        resources.clone(),
        CancellationToken::new(),
        Instant::now(),
    )
    .await;
    storage.shutdown().await.unwrap();
    assert!(outcome.is_err());
    assert!(!store.is_accepting());
    assert!(client.captured_requests().unwrap().is_empty());
    assert_eq!(resources.models.available_permits(), 1);
    let path = fixture.path();
    let (record, events) = tokio::task::spawn_blocking(move || {
        let mut reopened = Store::open(&path).unwrap();
        let record = run_response(
            reopened
                .execute(Command::Get {
                    owner_id: owner.clone(),
                    run_id: run.clone(),
                })
                .unwrap(),
        );
        let events = events_response(
            reopened
                .execute(Command::Events {
                    owner_id: owner,
                    run_id: run,
                    after: None,
                    limit: 100,
                })
                .unwrap(),
        );
        (record, events)
    })
    .await
    .unwrap();
    assert_eq!(record.phase, "interrupted");
    assert!(
        events
            .iter()
            .all(|e| e.kind != "model_planned" && e.kind != "model_finished")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancellation_during_a_model_wait_records_one_terminal_and_returns_its_permit() {
    let fixture = Fixture::new();
    let config = fixture.parse(CONFIG).unwrap();
    let authority = authority(&config, CaptureMode::Replay);
    let owner = authority.owner().to_owned();
    let run = authority.run_id().to_owned();
    let storage = fixture.storage().await;
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    let client = ModelClient::scripted([
        ScriptStep {
            delay: Duration::from_secs(2),
            reply: ModelReply::Answer("late answer".into()),
            usage: None,
        },
        ModelReply::Answer("forbidden retry".into()).into(),
    ]);
    let resources = RunResources::single(1, config.concurrency().clone());
    let cancel = CancellationToken::new();
    let owned = tokio::spawn(run_admitted(
        authority,
        client.clone(),
        store.clone(),
        resources.clone(),
        cancel.clone(),
        Instant::now(),
    ));
    let observed = timeout(Duration::from_secs(3), async {
        while client.captured_requests().unwrap().is_empty() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await;
    let held_during_exchange = resources.models.available_permits();
    let before = store
        .execute(
            Command::Events {
                owner_id: owner.clone(),
                run_id: run.clone(),
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await;
    cancel.cancel();
    let outcome = owned.await;
    let events = store
        .execute(
            Command::Events {
                owner_id: owner,
                run_id: run,
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await;
    storage.shutdown().await.unwrap();
    observed.expect("scripted model began within bounded startup time");
    assert_eq!(held_during_exchange, 0);
    let before = events_response(before.unwrap());
    assert_eq!(
        before.last().unwrap().kind,
        "model_planned",
        "intent must be committed before the model begins waiting"
    );
    let record = outcome
        .unwrap()
        .expect("cancellation must settle durably rather than leave an unfinished row");
    assert_eq!(record.phase, "cancelled");
    assert_eq!(record.acceptance_status, "unchecked");
    assert!(!record.task_accepted());
    assert!(record.result.is_none());
    let events = events_response(events.unwrap());
    assert_one_model_pair(&events);
    assert_eq!(
        events.iter().filter(|e| e.kind == "run_finished").count(),
        1
    );
    let finished = events.iter().find(|e| e.kind == "model_finished").unwrap();
    assert_eq!(
        finished.data["replay"]["reason"],
        "cancelled_remote_outcome_unknown"
    );
    assert_eq!(client.captured_requests().unwrap().len(), 1);
    assert_eq!(resources.models.available_permits(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execution_deadline_allows_terminal_bookkeeping_but_no_extra_generation() {
    let fixture = Fixture::new();
    let config = fixture
        .parse(&CONFIG.replace("max_run_s = 10", "max_run_s = 1"))
        .unwrap();
    let authority = authority(&config, CaptureMode::Metadata);
    let owner = authority.owner().to_owned();
    let run = authority.run_id().to_owned();
    let storage = fixture.storage().await;
    let store = storage.client();
    admit(&authority, &store, None).await.unwrap();
    let client = ModelClient::scripted([
        ScriptStep {
            delay: Duration::from_secs(2),
            reply: ModelReply::Answer("too late".into()),
            usage: None,
        },
        ModelReply::Answer("forbidden retry".into()).into(),
    ]);
    let resources = RunResources::single(1, config.concurrency().clone());
    let outcome = run_admitted(
        authority,
        client.clone(),
        store.clone(),
        resources.clone(),
        CancellationToken::new(),
        Instant::now(),
    )
    .await;
    let events = store
        .execute(
            Command::Events {
                owner_id: owner,
                run_id: run,
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await;
    storage.shutdown().await.unwrap();
    let record =
        outcome.expect("expired execution budget still permits bounded terminal bookkeeping");
    assert_eq!(record.phase, "stopped");
    assert_eq!(record.terminal_reason.as_deref(), Some("run_deadline"));
    assert_eq!(record.acceptance_status, "unchecked");
    let events = events_response(events.unwrap());
    assert_one_model_pair(&events);
    assert!(events.last().unwrap().elapsed_ms >= 1000);
    assert_eq!(client.captured_requests().unwrap().len(), 1);
    assert_eq!(resources.models.available_permits(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_preserves_the_entire_committed_terminal_record() {
    let case = run_case(
        ModelReply::Answer("retained answer".into()),
        CaptureMode::Replay,
    )
    .await;
    let path = case.fixture.path();
    let owner = case.record.owner_id.clone();
    let run = case.record.run_id.clone();
    let (record, events) = tokio::task::spawn_blocking(move || {
        let mut reopened = Store::open(&path).unwrap();
        let record = run_response(
            reopened
                .execute(Command::Get {
                    owner_id: owner.clone(),
                    run_id: run.clone(),
                })
                .unwrap(),
        );
        let events = events_response(
            reopened
                .execute(Command::Events {
                    owner_id: owner,
                    run_id: run,
                    after: None,
                    limit: 100,
                })
                .unwrap(),
        );
        (record, events)
    })
    .await
    .unwrap();
    assert_eq!(record, case.record);
    assert_eq!(events, case.events);
    assert!(events.iter().all(|e| e.kind != "recovery_interrupted"));
}
