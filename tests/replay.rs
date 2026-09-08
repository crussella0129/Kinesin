//! Captures are produced by the actual runner and journal. Replay receives only
//! owned data after all live resources have shut down and the workspace is gone.

use std::path::PathBuf;
use std::time::Duration;

use kinesin::config::{CaptureMode, Config};
use kinesin::core::{ModelReply, ToolCall};
use kinesin::model::{ModelClient, ScriptStep};
use kinesin::policy::Submission;
use kinesin::replay::{MAX_REPLAY_BYTES, MAX_REPLAY_EVENTS, Snapshot, decode_snapshot, replay};
use kinesin::runner::{RunResources, admit, run_admitted};
use kinesin::storage::{Command, QueueLimits, Response, Storage};
use serde_json::{Value, json};
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;

const CONFIG: &str = r#"
version = 1
instructions = "Use tools when requested. Return exact observed fields."
[storage]
path = "state/kinesin.sqlite"
capture = "metadata"
[limits]
max_model_turns = 3
max_run_s = 10
[[workspaces]]
id = "practice"
root = "workspace"
tools = ["read_file", "list_files"]
[[models]]
id = "local"
base_url = "http://127.0.0.1:1"
model_id = "scripted-fixture"
context_size = 4096
verified_slots = 1
temperature = 0.0
[[tasks]]
id = "practice-fields"
version = 1
checker = "file_fields_v1"
checker_version = 1
workspace = "practice"
[[tasks.criteria]]
id = "language"
path = "project.txt"
key = "language"
"#;

struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("kinesin-replay-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        std::fs::write(root.join("workspace/project.txt"), "language=Rust\n").unwrap();
        Self { root }
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
                .starts_with("kinesin-replay-")
        );
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn read_file() -> ModelReply {
    ModelReply::ToolCalls {
        content: None,
        calls: vec![ToolCall {
            id: "call-1".into(),
            name: "read_file".into(),
            arguments: "{\"path\":\"project.txt\"}".into(),
        }],
    }
}
fn answer(value: &str, evidence: &str) -> ModelReply {
    ModelReply::Answer(
        json!({"facts":[{"id":"language","value":value,"evidence_id":evidence}]}).to_string(),
    )
}
async fn capture(checked: bool, mode: CaptureMode, replies: Vec<ModelReply>) -> Snapshot {
    let fixture = Fixture::new();
    let config = Config::parse(CONFIG, &fixture.root.join("kinesin.toml")).unwrap();
    let submission = if checked {
        Submission::Checked {
            task: "practice-fields".into(),
            model: "local".into(),
            limits: None,
            capture: Some(mode),
        }
    } else {
        Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: "Please inspect the file".into(),
            limits: None,
            capture: Some(mode),
        }
    };
    let authority = config.authorize_local(submission).unwrap();
    let owner = authority.owner().to_owned();
    let run_id = authority.run_id().to_owned();
    let path = fixture.root.join("state/kinesin.sqlite");
    let storage = tokio::task::spawn_blocking(move || Storage::start(path, QueueLimits::default()))
        .await
        .unwrap()
        .unwrap();
    let store = storage.client();
    let resources = RunResources::single(1, config.concurrency().clone())
        .with_workspace("practice", &fixture.root.join("workspace"))
        .unwrap();
    let model = ModelClient::scripted(replies.into_iter().map(Into::into));
    admit(&authority, &store, None).await.unwrap();
    let result = run_admitted(
        authority,
        model.clone(),
        store.clone(),
        resources,
        CancellationToken::new(),
        Instant::now(),
    )
    .await;
    let page = store
        .execute(
            Command::Events {
                owner_id: owner,
                run_id,
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(2),
        )
        .await;
    storage.shutdown().await.unwrap();
    let run = result.unwrap();
    let Response::Events(events) = page.unwrap() else {
        panic!("event page");
    };
    // Fixture drop deletes the database, source file and configured workspace.
    // ModelClient is also dropped; no execution object reaches replay().
    Snapshot {
        schema_version: 2,
        capture_notice: None,
        run,
        events,
    }
}
fn event_index(snapshot: &Snapshot, kind: &str) -> usize {
    snapshot
        .events
        .iter()
        .position(|event| event.kind == kind)
        .unwrap()
}
fn replay_code(snapshot: &Snapshot) -> &'static str {
    replay(&snapshot.run, &snapshot.events).unwrap_err().code
}

#[derive(Clone, Copy)]
enum StopScenario {
    BeforeStart,
    ModelCancellation,
    RunDeadline,
    ModelQueueTimeout,
    ModelQueueUnavailable,
    IntentCancellation,
    ToolCancellation,
    JournalTimeout,
}

async fn capture_stop(scenario: StopScenario) -> (Snapshot, usize) {
    let fixture = Fixture::new();
    let mut source = CONFIG.replace(
        "temperature = 0.0",
        "temperature = 0.0\nmodel_queue_timeout_s = 1",
    );
    source.push_str("\n[concurrency]\njournal_admission_timeout_s = 1\n");
    if matches!(scenario, StopScenario::RunDeadline) {
        source = source.replace("max_run_s = 10", "max_run_s = 1");
    }
    let config = Config::parse(&source, &fixture.root.join("kinesin.toml")).unwrap();
    let authority = config
        .authorize_local(Submission::Checked {
            task: "practice-fields".into(),
            model: "local".into(),
            limits: None,
            capture: Some(CaptureMode::Replay),
        })
        .unwrap();
    let path = fixture.root.join("state/kinesin.sqlite");
    let storage = tokio::task::spawn_blocking(move || {
        Storage::start(
            path,
            QueueLimits {
                commands: 1,
                ..QueueLimits::default()
            },
        )
    })
    .await
    .unwrap()
    .unwrap();
    let store = storage.client();
    let resources = RunResources::single(1, config.concurrency().clone())
        .with_workspace("practice", &fixture.root.join("workspace"))
        .unwrap();
    let mut model_gate = if matches!(
        scenario,
        StopScenario::ModelQueueTimeout | StopScenario::IntentCancellation
    ) {
        Some(resources.models.clone().acquire_owned().await.unwrap())
    } else {
        None
    };
    if matches!(scenario, StopScenario::ModelQueueUnavailable) {
        resources.models.close();
    }
    let mut tool_gate = if matches!(scenario, StopScenario::ToolCancellation) {
        Some(
            resources
                .tools
                .clone()
                .acquire_many_owned(config.concurrency().max_blocking_tools as u32)
                .await
                .unwrap(),
        )
    } else {
        None
    };
    let delay = match scenario {
        StopScenario::ModelCancellation | StopScenario::RunDeadline => Duration::from_secs(2),
        StopScenario::JournalTimeout => Duration::from_millis(200),
        _ => Duration::ZERO,
    };
    let reply = if matches!(scenario, StopScenario::ToolCancellation) {
        ModelReply::ToolCalls {
            content: None,
            calls: vec![
                ToolCall {
                    id: "first".into(),
                    name: "read_file".into(),
                    arguments: "{\"path\":\"project.txt\"}".into(),
                },
                ToolCall {
                    id: "second".into(),
                    name: "read_file".into(),
                    arguments: "{\"path\":\"project.txt\"}".into(),
                },
            ],
        }
    } else {
        ModelReply::Answer("candidate retained through stop".into())
    };
    let model = ModelClient::scripted([ScriptStep { delay, reply }]);
    let cancel = CancellationToken::new();
    if matches!(scenario, StopScenario::BeforeStart) {
        cancel.cancel();
    }
    admit(&authority, &store, None).await.unwrap();
    let mut running = Box::pin(run_admitted(
        authority.clone(),
        model.clone(),
        store.clone(),
        resources.clone(),
        cancel.clone(),
        Instant::now(),
    ));
    if matches!(
        scenario,
        StopScenario::IntentCancellation | StopScenario::ToolCancellation
    ) {
        // Hold model capacity until run_started has actually committed. Then
        // reserve the only journal slot before allowing model intent to queue.
        let waiting_kind = if matches!(scenario, StopScenario::IntentCancellation) {
            "run_started"
        } else {
            "tool_planned"
        };
        let started = async {
            loop {
                let page = store
                    .execute(
                        Command::Events {
                            owner_id: authority.owner().into(),
                            run_id: authority.run_id().into(),
                            after: None,
                            limit: 100,
                        },
                        Instant::now() + Duration::from_secs(1),
                    )
                    .await
                    .unwrap();
                if matches!(page, Response::Events(events) if events.iter().any(|e|e.kind==waiting_kind))
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        };
        tokio::select! {
            outcome=&mut running=>panic!("run unexpectedly finished: {outcome:?}"),
            ready=timeout(Duration::from_secs(1),started)=>ready.unwrap(),
        }
        // Poll the owned future through run_started's acknowledgement until it
        // waits for the held model permit, without spawning a competing owner.
        advance_pending(running.as_mut()).await;
        let journal_gate = store
            .reserve(64, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        drop(model_gate.take());
        drop(tool_gate.take());
        // Advance synchronously through the now-ready model grant until the
        // journal reservation blocks. Cancellation therefore follows intent
        // preparation rather than racing the model-permit wakeup.
        advance_pending(running.as_mut()).await;
        cancel.cancel();
        drop(journal_gate);
    } else if matches!(
        scenario,
        StopScenario::ModelCancellation | StopScenario::JournalTimeout
    ) {
        let dispatched = async {
            while model.captured_requests().unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
        };
        tokio::select! {
            outcome=&mut running=>panic!("run unexpectedly finished: {outcome:?}"),
            ready=timeout(Duration::from_secs(1),dispatched)=>ready.unwrap(),
        }
        if matches!(scenario, StopScenario::ModelCancellation) {
            cancel.cancel();
        } else {
            let held = store
                .reserve(64, Instant::now() + Duration::from_secs(1))
                .await
                .unwrap();
            tokio::select! {
                outcome=&mut running=>panic!("run unexpectedly finished: {outcome:?}"),
                _=tokio::time::sleep(Duration::from_millis(1500))=>{},
            }
            drop(held);
        }
    }
    let outcome = running.await;
    drop(model_gate);
    drop(tool_gate);
    let page = store
        .execute(
            Command::Events {
                owner_id: authority.owner().into(),
                run_id: authority.run_id().into(),
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(2),
        )
        .await;
    storage.shutdown().await.unwrap();
    let run = outcome.unwrap();
    let Response::Events(events) = page.unwrap() else {
        panic!("events missing")
    };
    let attempts = model.captured_requests().unwrap().len();
    (
        Snapshot {
            schema_version: 2,
            capture_notice: None,
            run,
            events,
        },
        attempts,
    )
}

async fn advance_pending<F: std::future::Future>(mut future: std::pin::Pin<&mut F>) {
    std::future::poll_fn(|context| {
        assert!(
            future.as_mut().poll(context).is_pending(),
            "run must remain owned and pending"
        );
        std::task::Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn recorded_cancellation_deadline_and_queue_stops_replay_without_effects() {
    for (scenario, reason, attempts) in [
        (StopScenario::BeforeStart, "cancelled", 0),
        (StopScenario::ModelCancellation, "cancelled", 1),
        (StopScenario::RunDeadline, "run_deadline", 1),
        (StopScenario::ModelQueueTimeout, "model_queue_timeout", 0),
        (
            StopScenario::ModelQueueUnavailable,
            "model_queue_timeout",
            0,
        ),
        (StopScenario::JournalTimeout, "journal_admission_timeout", 1),
    ] {
        let (captured, actual_attempts) = capture_stop(scenario).await;
        assert_eq!(actual_attempts, attempts);
        assert_eq!(captured.run.terminal_reason.as_deref(), Some(reason));
        assert_eq!(captured.run.acceptance_status, "inconclusive");
        let report = replay(&captured.run, &captured.events).unwrap();
        assert_eq!(report.phase, captured.run.phase);
        assert_eq!(report.model_requests, attempts);
        assert!(!report.verification_replayed);
    }
}

#[tokio::test]
async fn a_committed_intent_cancelled_before_dispatch_replays_as_known_unsent() {
    let (captured, attempts) = capture_stop(StopScenario::IntentCancellation).await;
    assert_eq!(attempts, 0);
    let finished = &captured.events[event_index(&captured, "model_finished")];
    assert_eq!(finished.data["dispatch"], "unsent");
    let report = replay(&captured.run, &captured.events).unwrap();
    assert_eq!(report.model_requests, 0);
    assert_eq!(report.prepared_sha256.len(), 1);
    let mut changed = captured.clone();
    let index = event_index(&changed, "model_finished");
    changed.events[index].data["control_dispatch"]["cancelled"] = json!(false);
    assert_eq!(replay_code(&changed), "replay_unsent_without_stop");
}

#[tokio::test]
async fn cancellation_during_a_tool_batch_replays_only_the_started_observation() {
    let (captured, attempts) = capture_stop(StopScenario::ToolCancellation).await;
    assert_eq!(attempts, 1);
    assert_eq!(captured.run.phase, "cancelled");
    assert_eq!(
        captured
            .events
            .iter()
            .filter(|event| event.kind == "tool_planned")
            .count(),
        1
    );
    let finished = &captured.events[event_index(&captured, "tool_finished")];
    assert_eq!(finished.data["dispatch"], "executed");
    let report = replay(&captured.run, &captured.events).unwrap();
    assert_eq!(report.tool_observations, 1);
    assert_eq!(report.acceptance_status, "inconclusive");
    let mut changed = captured;
    let index = event_index(&changed, "tool_finished");
    changed.events[index].data["control_dispatch"]["cancelled"] = json!(true);
    assert_eq!(replay_code(&changed), "replay_dispatch_after_stop");
}

#[tokio::test]
async fn control_capture_rejects_shifted_timing_and_old_or_missing_arbitration() {
    let captured = capture(
        false,
        CaptureMode::Replay,
        vec![ModelReply::Answer("answer".into())],
    )
    .await;
    let mut changed = captured.clone();
    for event in changed.events.iter_mut().skip(1) {
        event.elapsed_ms += 20_000;
    }
    assert_eq!(replay_code(&changed), "replay_control_order");
    let mut changed = captured.clone();
    for event in changed.events.iter_mut().skip(1) {
        event.elapsed_ms += 20_000;
        for field in ["control_dispatch", "control_terminal"] {
            if let Some(control) = event.data.get_mut(field) {
                control["elapsed_us"] = json!(control["elapsed_us"].as_u64().unwrap() + 20_000_000);
            }
        }
    }
    assert_eq!(replay_code(&changed), "replay_dispatch_after_stop");
    let mut changed = captured.clone();
    changed
        .events
        .last_mut()
        .unwrap()
        .data
        .as_object_mut()
        .unwrap()
        .remove("control_terminal");
    assert_eq!(replay_code(&changed), "replay_control_missing");
    let mut changed = captured.clone();
    changed.events[0].data["versions"]["capture"] = json!(1);
    assert_eq!(replay_code(&changed), "replay_versions_unsupported");
    // The recorded pre-inbox check remains authoritative when settlement ends
    // late; moving only the terminal event time cannot retroactively cancel it.
    let mut late = captured;
    late.events.last_mut().unwrap().elapsed_ms = 20_000;
    assert_eq!(replay(&late.run, &late.events).unwrap().phase, "completed");
}

#[tokio::test]
async fn checked_capture_replays_after_workspace_and_controller_are_removed() {
    let captured = capture(
        true,
        CaptureMode::Replay,
        vec![read_file(), answer("Rust", "e0")],
    )
    .await;
    assert_eq!(captured.run.acceptance_status, "passed");
    let bytes = serde_json::to_vec(&captured).unwrap();
    let imported = decode_snapshot(&bytes).unwrap();
    let report = replay(&imported.run, &imported.events).unwrap();
    assert_eq!(report.consistency, "consistent");
    assert_eq!(report.phase, "completed");
    assert_eq!(report.acceptance_status, "passed");
    assert_eq!(report.model_requests, 2);
    assert_eq!(report.tool_observations, 1);
    assert!(report.verification_replayed);
    assert!(report.scope.contains("no origin authentication"));
    let recorded = imported
        .events
        .iter()
        .filter(|event| event.kind == "model_planned")
        .map(|event| event.data["request_sha256"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(report.prepared_sha256, recorded);
    assert_eq!(
        serde_json::to_vec(&captured).unwrap(),
        bytes,
        "replay is read-only"
    );
}

#[tokio::test]
async fn completed_wrong_and_unsupported_answers_keep_their_original_verdicts() {
    for (replies, status) in [
        (vec![read_file(), answer("Python", "e0")], "failed"),
        (vec![answer("Rust", "invented")], "failed"),
        (
            vec![ModelReply::Answer("ordinary freeform text".into())],
            "failed",
        ),
    ] {
        let captured = capture(true, CaptureMode::Replay, replies).await;
        let report = replay(&captured.run, &captured.events).unwrap();
        assert_eq!(report.phase, "completed");
        assert_eq!(report.acceptance_status, status);
        assert!(report.verification_replayed);
    }
    let captured = capture(
        false,
        CaptureMode::Replay,
        vec![read_file(), ModelReply::Answer("unchecked opinion".into())],
    )
    .await;
    let report = replay(&captured.run, &captured.events).unwrap();
    assert_eq!(report.acceptance_status, "unchecked");
    assert!(!report.verification_replayed);
}

#[tokio::test]
async fn deterministic_failure_length_and_policy_denial_replay_without_retry() {
    for (reply, phase) in [
        (
            ModelReply::Failure("model_connection_failed".into()),
            "failed",
        ),
        (
            ModelReply::Incomplete("generation_length".into()),
            "stopped",
        ),
        (ModelReply::Answer(String::new()), "failed"),
    ] {
        let captured = capture(false, CaptureMode::Replay, vec![reply]).await;
        let report = replay(&captured.run, &captured.events).unwrap();
        assert_eq!(report.phase, phase);
        assert_eq!(report.model_requests, 1);
        assert_eq!(report.acceptance_status, "unchecked");
    }
    let denial = ModelReply::ToolCalls {
        content: None,
        calls: vec![ToolCall {
            id: "denied-call".into(),
            name: "shell".into(),
            arguments: "{}".into(),
        }],
    };
    let captured = capture(
        false,
        CaptureMode::Replay,
        vec![denial, ModelReply::Answer("tool unavailable".into())],
    )
    .await;
    assert_eq!(
        replay(&captured.run, &captured.events)
            .unwrap()
            .tool_observations,
        1
    );
    let repeated = capture(
        false,
        CaptureMode::Replay,
        vec![read_file(), read_file(), read_file()],
    )
    .await;
    let report = replay(&repeated.run, &repeated.events).unwrap();
    assert_eq!(report.phase, "stopped");
    assert_eq!(
        repeated.run.terminal_reason.as_deref(),
        Some("repeat_limit")
    );
    assert_eq!(report.model_requests, 3);
    assert_eq!(report.tool_observations, 2);
}

#[tokio::test]
async fn missing_capture_versions_bodies_and_sequence_are_explicit_refusals() {
    let metadata = capture(
        true,
        CaptureMode::Metadata,
        vec![read_file(), answer("Rust", "e0")],
    )
    .await;
    assert_eq!(replay_code(&metadata), "replay_unavailable_metadata");
    let original = capture(
        true,
        CaptureMode::Replay,
        vec![read_file(), answer("Rust", "e0")],
    )
    .await;
    for (pointer, code) in [
        ("/versions", "replay_versions_unsupported"),
        ("/replay/authority", "replay_frozen_input_missing"),
    ] {
        let mut changed = original.clone();
        *changed.events[0].data.pointer_mut(pointer).unwrap() = Value::Null;
        assert_eq!(replay_code(&changed), code);
    }
    let mut changed = original.clone();
    changed.events[0].data["versions"]["adapter"] = json!(99);
    assert_eq!(replay_code(&changed), "replay_versions_unsupported");
    let mut changed = original.clone();
    changed.events[0].data["replay"]["authority"]["limits"]
        .as_object_mut()
        .unwrap()
        .remove("max_response_bytes");
    assert_eq!(replay_code(&changed), "replay_frozen_input_missing");
    let mut changed = original.clone();
    let index = event_index(&changed, "tool_finished");
    changed.events[index].data["replay"]["observation"]
        .as_object_mut()
        .unwrap()
        .remove("body");
    assert_eq!(replay_code(&changed), "replay_tool_input_missing");
    let mut changed = original.clone();
    let index = event_index(&changed, "model_finished");
    changed.events[index].data["replay"] = Value::Null;
    assert_eq!(replay_code(&changed), "replay_model_input_missing");
    let mut changed = original.clone();
    changed.events.remove(3);
    assert_eq!(replay_code(&changed), "replay_event_order");
    let mut changed = original.clone();
    changed.events[2].elapsed_ms = u64::MAX;
    assert_eq!(replay_code(&changed), "replay_event_order");
    let mut changed = original;
    changed.run.phase = "interrupted".into();
    assert_eq!(replay_code(&changed), "replay_terminal_unsupported");
}

#[tokio::test]
async fn correlation_fingerprint_candidate_evidence_and_receipt_tampering_diverge() {
    let original = capture(
        true,
        CaptureMode::Replay,
        vec![read_file(), answer("Rust", "e0")],
    )
    .await;
    for (kind, field, value, code) in [
        (
            "model_planned",
            "request_sha256",
            json!("0".repeat(64)),
            "replay_request_divergence",
        ),
        (
            "tool_planned",
            "call_id",
            json!("different"),
            "replay_tool_correlation",
        ),
        (
            "tool_planned",
            "model_effect_id",
            json!("model-9"),
            "replay_tool_correlation",
        ),
        (
            "tool_finished",
            "call_id",
            json!("different"),
            "replay_tool_correlation",
        ),
        (
            "tool_finished",
            "sha256",
            json!("0".repeat(64)),
            "replay_tool_observation_divergence",
        ),
        (
            "tool_finished",
            "resource",
            json!("different.txt"),
            "replay_tool_observation_divergence",
        ),
    ] {
        let mut changed = original.clone();
        let index = event_index(&changed, kind);
        changed.events[index].data[field] = value;
        assert_eq!(replay_code(&changed), code);
    }
    let mut changed = original.clone();
    changed.events[0].data["replay"]["authority"]["prompt"] = json!("changed prompt");
    assert_eq!(replay_code(&changed), "replay_input_sources");
    let mut changed = original.clone();
    changed.events[0].data["replay"]["authority"]["task"]["profile"]["criteria"][0]["key"] =
        json!("changed");
    assert_eq!(replay_code(&changed), "replay_spec_digest");
    let mut changed = original.clone();
    changed.run.owner_id = "different-owner".into();
    assert_eq!(replay_code(&changed), "replay_identity_mismatch");
    let mut changed = original.clone();
    changed.run.result.as_mut().unwrap()["candidate"] = json!("changed candidate");
    assert_eq!(replay_code(&changed), "replay_candidate_divergence");
    let mut changed = original;
    changed.run.receipt.as_mut().unwrap()["criteria"][0]["status"] = json!("failed");
    assert_eq!(replay_code(&changed), "replay_receipt_divergence");
}

#[tokio::test]
async fn checker_duration_is_compared_as_recorded_metadata_not_remeasured_time() {
    let mut captured = capture(
        true,
        CaptureMode::Replay,
        vec![read_file(), answer("Rust", "e0")],
    )
    .await;
    captured.run.receipt.as_mut().unwrap()["duration_ms"] = json!(123);
    captured.events.last_mut().unwrap().data["receipt"]["duration_ms"] = json!(123);
    assert!(
        replay(&captured.run, &captured.events)
            .unwrap()
            .verification_replayed
    );
}

#[tokio::test]
async fn imported_snapshots_are_versioned_and_bounded_before_replay() {
    assert_eq!(
        decode_snapshot(&vec![b' '; MAX_REPLAY_BYTES + 1])
            .unwrap_err()
            .code,
        "replay_bytes_limit"
    );
    assert_eq!(
        decode_snapshot(b"{\"schema_version\":2,\"schema_version\":2}")
            .unwrap_err()
            .code,
        "replay_invalid_snapshot"
    );
    let mut captured = capture(
        false,
        CaptureMode::Replay,
        vec![ModelReply::Answer("ok".into())],
    )
    .await;
    captured.schema_version = 1;
    assert_eq!(
        decode_snapshot(&serde_json::to_vec(&captured).unwrap())
            .unwrap_err()
            .code,
        "replay_schema_unsupported"
    );
    captured.events = vec![captured.events[0].clone(); MAX_REPLAY_EVENTS + 1];
    assert_eq!(replay_code(&captured), "replay_event_limit");
}
