//! Real child-process termination at committed journal boundaries. Replies are
//! scripted; the filesystem read, checker, writer, WAL and recovery are real.
//! This proves process-crash recovery, not physical power-loss durability.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::time::Duration;

use kinesin::config::{Config, ToolName};
use kinesin::core::{self, Effect, ModelReply, ToolCall};
use kinesin::model::{self, ModelClient};
use kinesin::policy::{RunAuthority, Submission};
use kinesin::runner::{RunResources, admit, options, run_admitted};
use kinesin::storage::{
    Command, Event, QueueLimits, Response, RunRecord, Storage, StorageClient, Store,
};
use kinesin::tools::{ToolStatus, WorkspaceReader};
use kinesin::verification::{EvidenceInventory, assess};
use serde_json::{Value, json};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const CONFIG: &str = r#"
version = 1
instructions = "Read the source and return exact observed fields."
[storage]
path = "state/kinesin.sqlite"
capture = "metadata"
[limits]
max_model_turns = 3
max_run_s = 10
[[workspaces]]
id = "practice"
root = "workspace"
tools = ["read_file"]
[[models]]
id = "local"
base_url = "http://127.0.0.1:1"
model_id = "scripted-fixture"
context_size = 4096
verified_slots = 1
temperature = 0.0
[[tasks]]
id = "fields"
version = 1
checker = "file_fields_v1"
checker_version = 1
workspace = "practice"
[[tasks.criteria]]
id = "language"
path = "project.txt"
key = "language"
"#;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("kinesin-process-recovery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        std::fs::write(root.join("workspace/project.txt"), "language=Rust\n").unwrap();
        std::fs::write(root.join("kinesin.toml"), CONFIG).unwrap();
        Self(root)
    }
}
fn checked_fixture_path(path: &Path) {
    assert!(path.is_absolute() && path.starts_with(std::env::temp_dir()));
    assert!(
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kinesin-process-recovery-")
    );
}
impl Drop for Fixture {
    fn drop(&mut self) {
        checked_fixture_path(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A timeout or assertion also terminates and reaps only this owned child.
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

fn authorize(config: &Config) -> RunAuthority {
    config
        .authorize_local(Submission::Checked {
            task: "fields".into(),
            model: "local".into(),
            limits: None,
            capture: None,
        })
        .unwrap()
}
fn scripted() -> ModelClient {
    ModelClient::scripted([
        ModelReply::ToolCalls {
            content: None,
            calls: vec![ToolCall {
                id: "call-1".into(),
                name: "read_file".into(),
                arguments: "{\"path\":\"project.txt\"}".into(),
            }],
        }
        .into(),
        ModelReply::Answer(
            json!({"facts":[{"id":"language","value":"Rust","evidence_id":"e0"}]}).to_string(),
        )
        .into(),
    ])
}
fn run(response: Response) -> RunRecord {
    let Response::Run(Some(record)) = response else {
        panic!("expected retained run");
    };
    *record
}
fn events(response: Response) -> Vec<Event> {
    let Response::Events(events) = response else {
        panic!("expected event page");
    };
    events
}
fn get(run_id: &str) -> Command {
    Command::Get {
        owner_id: "local".into(),
        run_id: run_id.into(),
    }
}
fn page(run_id: &str) -> Command {
    Command::Events {
        owner_id: "local".into(),
        run_id: run_id.into(),
        after: None,
        limit: 100,
    }
}

fn committed_baseline(root: &Path) -> (RunRecord, Vec<Event>) {
    let config = Config::parse(CONFIG, &root.join("kinesin.toml")).unwrap();
    let authority = authorize(&config);
    let storage =
        Storage::start(root.join("state/kinesin.sqlite"), QueueLimits::default()).unwrap();
    let client = storage.client();
    let resources = RunResources::single(1, config.concurrency().clone())
        .with_workspace("practice", &root.join("workspace"))
        .unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        admit(&authority, &client, None).await.unwrap();
        let result = run_admitted(
            authority,
            scripted(),
            client.clone(),
            resources,
            CancellationToken::new(),
            Instant::now(),
        )
        .await;
        let record = result.as_ref().unwrap();
        let history = client
            .execute(
                page(&record.run_id),
                Instant::now() + Duration::from_secs(2),
            )
            .await;
        storage.shutdown().await.unwrap();
        let record = result.unwrap();
        assert!(record.task_accepted());
        (record, events(history.unwrap()))
    })
}

fn kill_at_checkpoint(stage: &str, expected_events: usize, expected_attempts: usize) {
    let fixture = Fixture::new();
    let (baseline, baseline_events) = committed_baseline(&fixture.0);
    let mut child = OwnedChild(
        ProcessCommand::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "process_checkpoint_child",
                "--nocapture",
            ])
            .env("KINESIN_RECOVERY_CHILD_ROOT", &fixture.0)
            .env("KINESIN_RECOVERY_CHECKPOINT", stage)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let marker_path = fixture.0.join("checkpoint.json");
    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker_path.exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "checkpoint child exited early"
        );
        assert!(Instant::now() < deadline, "checkpoint child timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    let bytes = std::fs::read(&marker_path).unwrap();
    assert!(bytes.len() < 8_192);
    let marker: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(marker["stage"], stage);
    assert_eq!(marker["event_count"], expected_events);
    assert_eq!(marker["model_attempts"], expected_attempts);
    assert_eq!(marker["assessed_passed"], stage == "assessed");
    assert!(child.0.try_wait().unwrap().is_none());
    // TerminateProcess/SIGKILL, not a graceful journal shutdown or injected error.
    child.0.kill().unwrap();
    assert!(!child.0.wait().unwrap().success());

    // Recovery has no current source or configuration from which to reassess.
    std::fs::remove_file(fixture.0.join("kinesin.toml")).unwrap();
    std::fs::remove_file(fixture.0.join("workspace/project.txt")).unwrap();
    std::fs::remove_dir(fixture.0.join("workspace")).unwrap();
    let id = marker["run_id"].as_str().unwrap();
    let path = fixture.0.join("state/kinesin.sqlite");
    let mut store = Store::open(&path).unwrap();
    assert_eq!(run(store.execute(get(&baseline.run_id)).unwrap()), baseline);
    assert_eq!(
        events(store.execute(page(&baseline.run_id)).unwrap()),
        baseline_events
    );
    let recovered = run(store.execute(get(id)).unwrap());
    assert_eq!(recovered.phase, "interrupted");
    assert_eq!(recovered.acceptance_status, "inconclusive");
    assert_eq!(
        recovered.terminal_reason.as_deref(),
        Some("controller_interrupted")
    );
    assert!(!recovered.task_accepted());
    assert!(recovered.result.is_none() && recovered.result_sha256.is_none());
    let receipt = recovered.receipt.as_ref().unwrap();
    assert!(receipt["candidate_sha256"].is_null());
    assert_eq!(
        receipt["criteria"],
        json!([{
            "id":"language", "status":"inconclusive", "code":"controller_interrupted"
        }])
    );
    assert_eq!(
        receipt["contract"]["spec_sha256"],
        baseline.receipt.as_ref().unwrap()["contract"]["spec_sha256"]
    );
    let history = events(store.execute(page(id)).unwrap());
    assert_eq!(history.len(), expected_events + 1);
    assert_eq!(
        history.iter().map(|event| event.seq).collect::<Vec<_>>(),
        (0..history.len() as u64).collect::<Vec<_>>()
    );
    assert_eq!(history.last().unwrap().kind, "recovery_interrupted");
    assert_eq!(
        history.last().unwrap().data["pending_effect_outcome"],
        "unknown"
    );
    assert!(!history.iter().any(|event| event.kind == "run_finished"));
    assert_eq!(
        history
            .iter()
            .filter(|event| event.kind == "model_finished")
            .count(),
        expected_attempts
    );
    if stage == "intent" {
        assert_eq!(history[expected_events - 1].kind, "model_planned");
    }
    if stage == "assessed" {
        assert_eq!(marker["candidate_sha256"].as_str().unwrap().len(), 64);
        assert_eq!(
            history
                .iter()
                .filter(|event| event.kind == "tool_finished")
                .count(),
            1
        );
    }
    drop(store);
    let mut reopened = Store::open(&path).unwrap();
    assert_eq!(run(reopened.execute(get(id)).unwrap()), recovered);
    assert_eq!(events(reopened.execute(page(id)).unwrap()), history);
    assert_eq!(
        run(reopened.execute(get(&baseline.run_id)).unwrap()),
        baseline
    );
    assert_eq!(
        events(reopened.execute(page(&baseline.run_id)).unwrap()),
        baseline_events
    );
}

#[test]
fn process_killed_after_admission_recovers_without_execution() {
    kill_at_checkpoint("accepted", 1, 0);
}
#[test]
fn process_killed_after_effect_intent_recovers_unknown_outcome() {
    kill_at_checkpoint("intent", 3, 0);
}
#[test]
fn process_killed_after_checker_pass_does_not_publish_uncommitted_acceptance() {
    kill_at_checkpoint("assessed", 8, 2);
}

struct ChildJournal<'a> {
    client: &'a StorageClient,
    authority: &'a RunAuthority,
    started: Instant,
    next: u64,
}
impl ChildJournal<'_> {
    async fn append(&mut self, kind: &str, data: Value) {
        self.client
            .execute(
                Command::Append {
                    owner_id: self.authority.owner().into(),
                    run_id: self.authority.run_id().into(),
                    event: Event {
                        seq: self.next,
                        kind: kind.into(),
                        elapsed_ms: self.started.elapsed().as_millis() as u64,
                        data,
                    },
                    phase: (kind == "run_started").then(|| "running".into()),
                },
                Instant::now() + Duration::from_secs(2),
            )
            .await
            .unwrap();
        self.next += 1;
    }
}

fn publish_marker(root: &Path, marker: &Value) {
    let bytes = serde_json::to_vec(marker).unwrap();
    assert!(bytes.len() < 8_192);
    let temporary = root.join("checkpoint.tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    drop(file);
    std::fs::rename(temporary, root.join("checkpoint.json")).unwrap();
}

#[test]
#[ignore = "worker invoked only by the three parent process-crash tests"]
fn process_checkpoint_child() {
    let root =
        PathBuf::from(std::env::var_os("KINESIN_RECOVERY_CHILD_ROOT").expect("parent fixture"));
    checked_fixture_path(&root);
    let stage = std::env::var("KINESIN_RECOVERY_CHECKPOINT").expect("parent checkpoint");
    assert!(matches!(stage.as_str(), "accepted" | "intent" | "assessed"));
    let config = Config::parse(
        &std::fs::read_to_string(root.join("kinesin.toml")).unwrap(),
        &root.join("kinesin.toml"),
    )
    .unwrap();
    let authority = authorize(&config);
    let storage =
        Storage::start(root.join("state/kinesin.sqlite"), QueueLimits::default()).unwrap();
    let client = storage.client();
    let model = scripted();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        admit(&authority, &client, None).await.unwrap();
        let mut journal = ChildJournal { client: &client, authority: &authority, started: Instant::now(), next: 1 };
        let mut candidate_hash = None;
        if stage != "accepted" {
            journal.append("run_started", json!({})).await;
            let (mut state, effect) = core::initiate_with_tools(authority.instructions().into(), authority.prompt().into(), true).unwrap();
            assert_eq!(effect, Effect::Model);
            state.require_acceptance_check().unwrap();
            let prepared = model::prepare(state.messages(), &options(&authority)).unwrap();
            journal.append("model_planned", json!({"effect_id":"model-0","request_sha256":prepared.sha256(),"request_bytes":prepared.bytes().len(),"model_profile":"local"})).await;
            if stage == "assessed" {
                state.model_started().unwrap();
                let observation = model.send(&prepared).await.unwrap();
                journal.append("model_finished", json!({"effect_id":"model-0","dispatch":"attempted","classification":"tool_calls"})).await;
                let Effect::Tools(calls) = state.observe_model(observation).unwrap() else { panic!("expected read effect") };
                assert_eq!(calls.len(), 1);
                journal.append("tool_planned", json!({"effect_id":"tool-0","call_id":calls[0].id,"tool":"read_file","resource":"project.txt"})).await;
                let result = WorkspaceReader::open(&root.join("workspace")).unwrap().execute(ToolName::ReadFile, "project.txt", authority.limits().max_tool_result_bytes, Some("e0"));
                assert_eq!(result.status, ToolStatus::Ok);
                let observation_seq = journal.next;
                journal.append("tool_finished", json!({"effect_id":"tool-0","call_id":calls[0].id,"dispatch":"executed","classification":result.status,"tool":"read_file","resource":"project.txt","complete":!result.truncated,"result_bytes":result.encoded().unwrap().len(),"evidence_id":result.evidence_id,"sha256":model::fingerprint(result.body.as_bytes())})).await;
                let mut evidence = EvidenceInventory::default();
                evidence.add_observation(&authority, "tool-0", "project.txt", &result, observation_seq).unwrap();
                assert_eq!(state.observe_tool(&calls[0].id, result.encoded().unwrap()).unwrap(), Some(Effect::Model));
                let prepared = model::prepare(state.messages(), &options(&authority)).unwrap();
                journal.append("model_planned", json!({"effect_id":"model-1","request_sha256":prepared.sha256(),"request_bytes":prepared.bytes().len(),"model_profile":"local"})).await;
                state.model_started().unwrap();
                let observation = model.send(&prepared).await.unwrap();
                journal.append("model_finished", json!({"effect_id":"model-1","dispatch":"attempted","classification":"answer"})).await;
                let Effect::Candidate(candidate) = state.observe_model(observation).unwrap() else { panic!("expected candidate") };
                let receipt = assess(&authority, &candidate, &evidence);
                assert_eq!(receipt.status, "passed");
                assert_eq!(receipt.verified_fields().unwrap().len(), 1);
                candidate_hash = receipt.candidate_sha256;
                // No Finish command is submitted: this real checker pass remains private.
            }
        }
        let persisted = run(client.execute(get(authority.run_id()), Instant::now() + Duration::from_secs(2)).await.unwrap());
        assert_eq!(persisted.acceptance_status, "pending");
        assert!(persisted.result.is_none() && persisted.receipt.is_none());
        publish_marker(&root, &json!({"stage":stage,"run_id":authority.run_id(),"event_count":journal.next,"model_attempts":model.captured_requests().unwrap().len(),"assessed_passed":candidate_hash.is_some(),"candidate_sha256":candidate_hash}));
    });
    // Keep the actual connection-owning writer alive until the parent kills us.
    loop {
        std::hint::black_box((&storage, &client, &runtime));
        std::thread::park();
    }
}
