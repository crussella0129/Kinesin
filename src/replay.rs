//! Pure consistency replay of private captures. Imported data cannot create
//! RunAuthority, open a capability, dispatch an effect, or publish a live verdict.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::config::{CaptureMode, Limits, ModelConfig, ToolName, WorkspaceConfig, validate_id};
use crate::core::{self, Effect, ModelReply, RunPhase, ToolCall};
use crate::model::{self, ModelOptions};
use crate::policy::{InputSource, TaskContract};
use crate::storage::{Event, RunRecord};
use crate::tools::{ToolResult, ToolStatus, TypedToolArgs, normalized_path};
use crate::verification::{self, AssessmentContext, EvidenceInventory};

pub const MAX_REPLAY_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_REPLAY_EVENTS: usize = 256;

/// Bump the applicable version whenever its recorded semantics change.
pub fn versions() -> Value {
    json!({"capture":2,"core":1,"adapter":1,"tools":1,"checker":1,
        "source_parser":1,"output_contract":1})
}

/// Recorded input at one actual permission/arbitration check, never a clock
/// sampled again by replay. All times use the accepted run's monotonic origin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlObservation {
    pub elapsed_us: u64,
    pub cancelled: bool,
    pub journal_stopped: bool,
}

impl ControlObservation {
    pub fn stop(&self, max_run_s: u64) -> Option<(RunPhase, &'static str)> {
        if self.cancelled {
            Some((RunPhase::Cancelled, "cancelled"))
        } else if self.elapsed_us / 1_000_000 >= max_run_s {
            Some((RunPhase::Stopped, "run_deadline"))
        } else if self.journal_stopped {
            Some((RunPhase::Stopped, "journal_admission_timeout"))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueueStopKind {
    Timeout,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QueueStop {
    pub started_us: u64,
    pub finished_us: u64,
    pub deadline_us: u64,
    pub kind: QueueStopKind,
}

#[derive(Default)]
struct RecordedControl {
    last_us: u64,
    cancelled: bool,
    journal_stopped: bool,
}

impl RecordedControl {
    fn observe(
        &mut self,
        event: &Event,
        field: &str,
        earliest_ms: u64,
    ) -> Result<ControlObservation, ReplayError> {
        let control: ControlObservation = serde_json::from_value(event.data[field].clone())
            .map_err(|_| error("replay_control_missing", Some(event.seq)))?;
        ensure(
            control.elapsed_us >= earliest_ms.saturating_mul(1000)
                && control.elapsed_us / 1000 <= event.elapsed_ms
                && control.elapsed_us >= self.last_us
                && (!self.cancelled || control.cancelled)
                && (!self.journal_stopped || control.journal_stopped),
            "replay_control_order",
            Some(event.seq),
        )?;
        self.last_us = control.elapsed_us;
        self.cancelled = control.cancelled;
        self.journal_stopped = control.journal_stopped;
        Ok(control)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_notice: Option<String>,
    pub run: RunRecord,
    pub events: Vec<Event>,
}

/// File/HTTP callers must bound their read too. This second bound precedes JSON
/// parsing and the replay function independently bounds already parsed inputs.
pub fn decode_snapshot(bytes: &[u8]) -> Result<Snapshot, ReplayError> {
    ensure(bytes.len() <= MAX_REPLAY_BYTES, "replay_bytes_limit", None)?;
    let snapshot: Snapshot =
        serde_json::from_slice(bytes).map_err(|_| error("replay_invalid_snapshot", None))?;
    ensure(
        snapshot.schema_version == 2,
        "replay_schema_unsupported",
        None,
    )?;
    ensure(
        snapshot.events.len() <= MAX_REPLAY_EVENTS,
        "replay_event_limit",
        None,
    )?;
    Ok(snapshot)
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReplayReport {
    pub owner_id: String,
    pub run_id: String,
    pub consistency: String,
    pub phase: String,
    pub acceptance_status: String,
    pub event_count: usize,
    pub model_requests: usize,
    pub tool_observations: usize,
    pub prepared_sha256: Vec<String>,
    pub verification_replayed: bool,
    pub scope: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReplayError {
    pub code: &'static str,
    pub seq: Option<u64>,
}
impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.seq {
            Some(seq) => write!(f, "{} at event {seq}", self.code),
            None => f.write_str(self.code),
        }
    }
}
impl std::error::Error for ReplayError {}
fn error(code: &'static str, seq: Option<u64>) -> ReplayError {
    ReplayError { code, seq }
}
fn ensure(ok: bool, code: &'static str, seq: Option<u64>) -> Result<(), ReplayError> {
    if ok { Ok(()) } else { Err(error(code, seq)) }
}

/// This private deserializable record is data, never live authority. Ambient
/// path fields are recognized for schema compatibility but are never consulted.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrozenContext {
    owner: String,
    run_id: String,
    #[serde(rename = "config_path")]
    _config_path: Value,
    #[serde(rename = "storage")]
    _storage: Value,
    workspace: WorkspaceConfig,
    model: ModelConfig,
    limits: Limits,
    capture: CaptureMode,
    task: TaskContract,
    task_spec_sha256: String,
    instructions: String,
    prompt: String,
    submission_sha256: String,
    input_sources: Vec<InputSource>,
}
impl AssessmentContext for FrozenContext {
    fn owner(&self) -> &str {
        &self.owner
    }
    fn run_id(&self) -> &str {
        &self.run_id
    }
    fn limits(&self) -> &Limits {
        &self.limits
    }
    fn task(&self) -> &TaskContract {
        &self.task
    }
    fn task_spec_sha256(&self) -> &str {
        &self.task_spec_sha256
    }
}
impl FrozenContext {
    fn options(&self) -> ModelOptions {
        ModelOptions {
            origin: self.model.base_url.clone(),
            served_model: self.model.model_id.clone(),
            temperature: self.model.temperature,
            max_output_tokens: self.limits.max_output_tokens as usize,
            max_request_bytes: self.limits.max_request_bytes,
            max_response_bytes: self.limits.max_response_bytes,
            stream: self.model.stream,
            tools: self
                .workspace
                .tools
                .iter()
                .map(|tool| tool.as_str().into())
                .collect(),
        }
    }
    fn validate(&self, run: &RunRecord, accepted: &Event) -> Result<(), ReplayError> {
        let seq = Some(0);
        ensure(
            self.owner == run.owner_id
                && self.run_id == run.run_id
                && self.workspace.id == run.workspace_id
                && self.model.id == run.model_profile_id
                && self.capture == CaptureMode::Replay
                && self.submission_sha256 == run.submission_sha256
                && run.policy_version == "1"
                && accepted.data["policy_version"] == "1",
            "replay_identity_mismatch",
            seq,
        )?;
        ensure(
            self.limits.validate().is_ok()
                && self.instructions.len() <= 16384
                && self.prompt.len() <= 16384
                && self.model.temperature.is_finite()
                && !self.model.model_id.is_empty()
                && self
                    .workspace
                    .tools
                    .iter()
                    .map(|tool| tool.as_str())
                    .collect::<BTreeSet<_>>()
                    .len()
                    == self.workspace.tools.len(),
            "replay_invalid_frozen_input",
            seq,
        )?;
        ensure(
            accepted.data["owner_id"] == self.owner
                && accepted.data["limits"] == json!(self.limits)
                && accepted.data["input_sources"] == json!(self.input_sources),
            "replay_admission_mismatch",
            seq,
        )?;
        let task = serde_json::to_vec(&(1_u32, &self.task))
            .map_err(|_| error("replay_invalid_frozen_input", seq))?;
        ensure(
            task.len() <= 8192 && model::fingerprint(&task) == self.task_spec_sha256,
            "replay_spec_digest",
            seq,
        )?;
        let (criteria, task_identity) = match &self.task {
            TaskContract::Freeform => {
                ensure(
                    run.task_mode == "freeform"
                        && run.task_profile_id.is_none()
                        && run.task_profile_version.is_none()
                        && run.checker_id.is_none()
                        && run.checker_version.is_none()
                        && run.task_spec_sha256.is_none(),
                    "replay_task_identity",
                    seq,
                )?;
                (Vec::new(), Value::Null)
            }
            TaskContract::FileFieldsV1(profile) => {
                ensure(
                    run.task_mode == "checked"
                        && run.task_profile_id.as_deref() == Some(profile.id.as_str())
                        && run.task_profile_version == Some(profile.version.to_string())
                        && run.checker_id.as_deref() == Some(profile.checker.as_str())
                        && run.checker_version == Some(profile.checker_version.to_string())
                        && run.task_spec_sha256.as_deref() == Some(self.task_spec_sha256.as_str())
                        && profile.workspace == self.workspace.id
                        && profile.checker == "file_fields_v1"
                        && profile.checker_version == 1
                        && (1..=4).contains(&profile.criteria.len())
                        && self.workspace.tools.contains(&ToolName::ReadFile),
                    "replay_task_identity",
                    seq,
                )?;
                let ids = profile
                    .criteria
                    .iter()
                    .map(|criterion| criterion.id.clone())
                    .collect::<Vec<_>>();
                ensure(
                    ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
                        && profile.criteria.iter().all(|criterion| {
                            validate_id(&criterion.id).is_ok()
                                && normalized_path(&criterion.path).is_ok()
                                && !criterion.key.is_empty()
                                && criterion.key.len() <= 32
                        }),
                    "replay_invalid_frozen_input",
                    seq,
                )?;
                let identity = json!({"profile_id":profile.id,"profile_version":profile.version.to_string(),
                    "checker_id":profile.checker,"checker_version":profile.checker_version.to_string(),
                    "spec_sha256":self.task_spec_sha256,"criterion_ids":ids});
                (ids, identity)
            }
        };
        ensure(
            accepted.data["task"] == task_identity
                && accepted.data["required_criterion_ids"] == json!(criteria),
            "replay_task_identity",
            seq,
        )?;
        ensure(self.input_sources.len() == 2, "replay_input_sources", seq)?;
        for (source, id, purpose, origin, text) in [
            (
                &self.input_sources[0],
                "instructions",
                "system instructions",
                "trusted configuration",
                self.instructions.as_str(),
            ),
            (
                &self.input_sources[1],
                "task",
                "task request",
                if self.task.is_checked() {
                    "trusted task profile"
                } else {
                    "owner submission"
                },
                self.prompt.as_str(),
            ),
        ] {
            ensure(
                source.id == id
                    && source.purpose == purpose
                    && source.origin == origin
                    && source.bytes == text.len()
                    && source.sha256 == model::fingerprint(text.as_bytes()),
                "replay_input_sources",
                seq,
            )?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RecordedReply {
    Answer {
        text: String,
    },
    ToolCalls {
        content: Option<String>,
        calls: Vec<RecordedCall>,
    },
    Incomplete {
        reason: String,
    },
    Failure {
        reason: String,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedCall {
    id: String,
    name: String,
    arguments: String,
}
impl From<RecordedReply> for ModelReply {
    fn from(reply: RecordedReply) -> Self {
        match reply {
            RecordedReply::Answer { text } => Self::Answer(text),
            RecordedReply::ToolCalls { content, calls } => Self::ToolCalls {
                content,
                calls: calls
                    .into_iter()
                    .map(|call| ToolCall {
                        id: call.id,
                        name: call.name,
                        arguments: call.arguments,
                    })
                    .collect(),
            },
            RecordedReply::Incomplete { reason } => Self::Incomplete(reason),
            RecordedReply::Failure { reason } => Self::Failure(reason),
        }
    }
}

/// Establishes internal consistency of captured inputs, not their authenticity
/// or present-world correctness. The original database verdict is never changed.
pub fn replay(run: &RunRecord, events: &[Event]) -> Result<ReplayReport, ReplayError> {
    ensure(run.capture == "replay", "replay_unavailable_metadata", None)?;
    ensure(
        matches!(
            run.phase.as_str(),
            "completed" | "failed" | "stopped" | "cancelled"
        ),
        "replay_terminal_unsupported",
        None,
    )?;
    ensure(
        (3..=MAX_REPLAY_EVENTS).contains(&events.len()),
        "replay_event_limit",
        None,
    )?;
    bounded_serialized(&(run, events), MAX_REPLAY_BYTES)?;
    for (index, event) in events.iter().enumerate() {
        ensure(
            event.seq == index as u64
                && (index == 0 || event.elapsed_ms >= events[index - 1].elapsed_ms),
            "replay_event_order",
            Some(event.seq),
        )?;
        bounded_serialized(event, 2 * 1024 * 1024)?;
    }
    let accepted = &events[0];
    ensure(
        accepted.kind == "run_accepted" && accepted.elapsed_ms == 0,
        "replay_admission_missing",
        Some(0),
    )?;
    ensure(
        accepted.data["versions"] == versions(),
        "replay_versions_unsupported",
        Some(0),
    )?;
    let frozen: FrozenContext =
        serde_json::from_value(accepted.data["replay"]["authority"].clone())
            .map_err(|_| error("replay_frozen_input_missing", Some(0)))?;
    // Config structs permit defaults for live configuration. A replay capture
    // must contain every effective field; never silently fill a redacted value.
    ensure(
        json!(frozen) == accepted.data["replay"]["authority"],
        "replay_frozen_input_missing",
        Some(0),
    )?;
    frozen.validate(run, accepted)?;
    ensure(
        events[1].kind == "run_started",
        "replay_start_missing",
        Some(1),
    )?;
    let (mut state, mut effect) = core::initiate_with_tools(
        frozen.instructions.clone(),
        frozen.prompt.clone(),
        !frozen.workspace.tools.is_empty(),
    )
    .map_err(|_| error("replay_initial_transition", Some(0)))?;
    if frozen.task.is_checked() {
        state
            .require_acceptance_check()
            .map_err(|_| error("replay_initial_transition", Some(0)))?;
    }
    let mut index = 2;
    let mut observed_candidate = None;
    let mut evidence = EvidenceInventory::default();
    let mut previous_batch = None;
    let mut repeat_count = 0;
    let mut prepared_hashes = Vec::new();
    let mut observations = 0;
    let mut control_log = RecordedControl::default();
    while index < events.len() - 1 {
        let planned = &events[index];
        let finished = events
            .get(index + 1)
            .ok_or_else(|| error("replay_effect_unfinished", Some(planned.seq)))?;
        match &effect {
            Effect::Model => {
                ensure(
                    planned.kind == "model_planned" && finished.kind == "model_finished",
                    "replay_effect_order",
                    Some(planned.seq),
                )?;
                ensure(
                    state.counters().model_turns < frozen.limits.max_model_turns as usize
                        && !history_exceeds(&state, frozen.limits.max_history_bytes)?,
                    "replay_limit_divergence",
                    Some(planned.seq),
                )?;
                let request = model::prepare(state.messages(), &frozen.options())
                    .map_err(|_| error("replay_request_preparation", Some(planned.seq)))?;
                let id = format!("model-{}", state.counters().model_turns);
                ensure(
                    planned.data["effect_id"] == id
                        && finished.data["effect_id"] == id
                        && planned.data["request_sha256"] == request.sha256()
                        && planned.data["request_bytes"] == request.bytes().len()
                        && planned.data["model_profile"] == frozen.model.id,
                    "replay_request_divergence",
                    Some(planned.seq),
                )?;
                let control =
                    control_log.observe(finished, "control_dispatch", planned.elapsed_ms)?;
                prepared_hashes.push(request.sha256().into());
                if finished.data["dispatch"] == "unsent" {
                    let (phase, reason) = control
                        .stop(frozen.limits.max_run_s)
                        .ok_or_else(|| error("replay_unsent_without_stop", Some(finished.seq)))?;
                    ensure(
                        finished.data["reason"] == reason && finished.data.get("replay").is_none(),
                        "replay_unsent_divergence",
                        Some(finished.seq),
                    )?;
                    effect = state
                        .stop(phase, reason.into())
                        .map_err(|_| error("replay_core_divergence", Some(finished.seq)))?;
                    index += 2;
                    continue;
                }
                ensure(
                    finished.data["dispatch"] == "attempted"
                        && control.stop(frozen.limits.max_run_s).is_none(),
                    "replay_dispatch_after_stop",
                    Some(finished.seq),
                )?;
                let observed: RecordedReply =
                    serde_json::from_value(finished.data["replay"].clone())
                        .map_err(|_| error("replay_model_input_missing", Some(finished.seq)))?;
                ensure(
                    finished.data["classification"] == finished.data["replay"]["kind"],
                    "replay_model_classification",
                    Some(finished.seq),
                )?;
                state
                    .model_started()
                    .map_err(|_| error("replay_core_divergence", Some(planned.seq)))?;
                let observed: ModelReply = observed.into();
                if let ModelReply::Answer(candidate) = &observed {
                    observed_candidate = Some(candidate.clone());
                }
                let mut stop = None;
                if let ModelReply::ToolCalls { calls, .. } = &observed {
                    if calls.len()
                        > (frozen.limits.max_tool_calls as usize)
                            .saturating_sub(state.counters().tool_calls)
                    {
                        stop = Some("tool_call_limit");
                    } else {
                        let batch=calls.iter().map(|call|json!({"name":call.name,"arguments":serde_json::from_str::<Value>(&call.arguments)
                            .unwrap_or_else(|_|Value::String(call.arguments.clone()))})).collect::<Vec<_>>();
                        if previous_batch.as_ref() == Some(&batch) {
                            repeat_count += 1;
                        } else {
                            previous_batch = Some(batch);
                            repeat_count = 1;
                        }
                        if repeat_count >= frozen.limits.repeat_limit {
                            stop = Some("repeat_limit");
                        }
                    }
                }
                effect = if let Some(reason) = stop {
                    stop_state(&mut state, reason)?
                } else {
                    let mut next = state.clone();
                    let proposed = next
                        .observe_model(observed)
                        .map_err(|_| error("replay_core_divergence", Some(finished.seq)))?;
                    if history_exceeds(&next, frozen.limits.max_history_bytes)? {
                        stop_state(&mut state, "history_bytes_limit")?
                    } else {
                        state = next;
                        proposed
                    }
                };
                index += 2;
            }
            Effect::Tools(_) => {
                let call = state
                    .pending_batch()
                    .first()
                    .cloned()
                    .ok_or_else(|| error("replay_core_divergence", Some(planned.seq)))?;
                let id = format!(
                    "tool-{}",
                    state.counters().tool_calls - state.pending_batch().len()
                );
                let model_id = format!("model-{}", state.counters().model_turns - 1);
                ensure(
                    planned.kind == "tool_planned"
                        && finished.kind == "tool_finished"
                        && planned.data["effect_id"] == id
                        && finished.data["effect_id"] == id
                        && planned.data["call_id"] == call.id
                        && finished.data["call_id"] == call.id
                        && planned.data["model_effect_id"] == model_id
                        && planned.data["tool"] == call.name
                        && finished.data["tool"] == call.name
                        && planned.data["arguments_sha256"]
                            == model::fingerprint(call.arguments.as_bytes())
                        && planned.data["replay"]["arguments"] == call.arguments,
                    "replay_tool_correlation",
                    Some(planned.seq),
                )?;
                let result: ToolResult =
                    serde_json::from_value(finished.data["replay"]["observation"].clone())
                        .map_err(|_| error("replay_tool_input_missing", Some(finished.seq)))?;
                let args = TypedToolArgs::parse(&call.arguments);
                let tool = match call.name.as_str() {
                    "read_file" => Some(ToolName::ReadFile),
                    "list_files" => Some(ToolName::ListFiles),
                    "search_files" => Some(ToolName::SearchFiles),
                    _ => None,
                };
                let denial = match tool {
                    None => Some(ToolResult::failure(
                        ToolStatus::Denied,
                        "unknown_tool",
                        "Tool is not available",
                    )),
                    Some(name) if !frozen.workspace.tools.contains(&name) => {
                        Some(ToolResult::failure(
                            ToolStatus::Denied,
                            "tool_denied",
                            "Tool is not allowed",
                        ))
                    }
                    Some(_) if args.is_err() => Some(ToolResult::failure(
                        ToolStatus::Denied,
                        "invalid_arguments",
                        "Tool arguments or resource path are invalid",
                    )),
                    _ => None,
                };
                ensure(
                    planned.data["dispatch"]
                        == if denial.is_some() {
                            "denied"
                        } else {
                            "pending"
                        },
                    "replay_policy_divergence",
                    Some(planned.seq),
                )?;
                let dispatch = finished.data["dispatch"].as_str().unwrap_or("");
                let control =
                    control_log.observe(finished, "control_dispatch", planned.elapsed_ms)?;
                let dispatch_stop = control.stop(frozen.limits.max_run_s);
                ensure(
                    dispatch != "executed" || dispatch_stop.is_none(),
                    "replay_dispatch_after_stop",
                    Some(finished.seq),
                )?;
                if let Some(expected) = denial {
                    ensure(
                        result == expected && dispatch == "denied",
                        "replay_policy_divergence",
                        Some(finished.seq),
                    )?;
                } else {
                    ensure(
                        dispatch == "executed"
                            || (dispatch == "unsent" && result.status == ToolStatus::Error),
                        "replay_tool_dispatch",
                        Some(finished.seq),
                    )?;
                }
                let encoded = result
                    .encoded()
                    .map_err(|_| error("replay_tool_input_missing", Some(finished.seq)))?;
                ensure(
                    encoded.len() <= frozen.limits.max_tool_result_bytes.min(8192)
                        && finished.data["result_bytes"] == encoded.len()
                        && finished.data["classification"] == json!(result.status)
                        && finished.data["complete"] == !result.truncated
                        && finished.data["evidence_id"] == json!(result.evidence_id)
                        && finished.data["resource"]
                            == json!(args.as_ref().ok().map(|args| args.path.as_str()))
                        && finished.data["sha256"] == model::fingerprint(result.body.as_bytes())
                        && (result.status == ToolStatus::Ok) == result.error.is_none(),
                    "replay_tool_observation_divergence",
                    Some(finished.seq),
                )?;
                let mut evidence_error = None;
                if dispatch == "executed" && tool == Some(ToolName::ReadFile) {
                    if result.status == ToolStatus::Ok {
                        ensure(
                            result.evidence_id.as_deref() == Some(evidence.next_id().as_str()),
                            "replay_evidence_identity",
                            Some(finished.seq),
                        )?;
                    }
                    evidence_error = evidence
                        .add_context_observation(
                            &frozen,
                            &id,
                            &args
                                .as_ref()
                                .map_err(|_| error("replay_policy_divergence", Some(finished.seq)))?
                                .path,
                            &result,
                            finished.seq,
                        )
                        .err();
                }
                observations += 1;
                if let Some((phase, reason)) = dispatch_stop {
                    effect = state
                        .stop(phase, reason.into())
                        .map_err(|_| error("replay_core_divergence", Some(finished.seq)))?;
                } else if let Some(reason) = evidence_error {
                    effect = stop_state(&mut state, &reason)?;
                } else {
                    if let Some(next) = state
                        .observe_tool(&call.id, encoded)
                        .map_err(|_| error("replay_core_divergence", Some(finished.seq)))?
                    {
                        effect = next;
                    }
                    if history_exceeds(&state, frozen.limits.max_history_bytes)? {
                        effect = stop_state(&mut state, "history_bytes_limit")?;
                    }
                }
                index += 2;
            }
            _ => return Err(error("replay_effect_after_decision", Some(planned.seq))),
        }
    }
    ensure(
        index == events.len() - 1,
        "replay_effect_unfinished",
        Some(index as u64),
    )?;
    let terminal = &events[index];
    let terminal_control =
        control_log.observe(terminal, "control_terminal", events[index - 1].elapsed_ms)?;
    if effect == Effect::Model {
        let reason = if state.counters().model_turns >= frozen.limits.max_model_turns as usize {
            Some("model_turn_limit".to_owned())
        } else if history_exceeds(&state, frozen.limits.max_history_bytes)? {
            Some("history_bytes_limit".into())
        } else {
            model::prepare(state.messages(), &frozen.options()).err()
        };
        if let Some(reason) = reason {
            effect = stop_state(&mut state, &reason)?;
        }
    }
    let queue_stop: Option<QueueStop> = serde_json::from_value(
        terminal
            .data
            .get("queue_stop")
            .cloned()
            .ok_or_else(|| error("replay_control_missing", Some(terminal.seq)))?,
    )
    .map_err(|_| error("replay_queue_observation", Some(terminal.seq)))?;
    if let Some(queue) = queue_stop {
        let run_deadline = frozen.limits.max_run_s.saturating_mul(1_000_000);
        let expected_deadline = run_deadline.min(
            queue
                .started_us
                .saturating_add(frozen.model.model_queue_timeout_s.saturating_mul(1_000_000)),
        );
        ensure(
            effect == Effect::Model
                && queue.started_us >= events[index - 1].elapsed_ms.saturating_mul(1000)
                && queue.started_us <= queue.finished_us
                && queue.finished_us <= terminal_control.elapsed_us
                && queue.deadline_us == expected_deadline
                && (queue.kind != QueueStopKind::Timeout || queue.finished_us >= queue.deadline_us),
            "replay_queue_observation",
            Some(terminal.seq),
        )?;
        effect = stop_state(&mut state, "model_queue_timeout")?;
    }
    if let Some((phase, reason)) = terminal_control.stop(frozen.limits.max_run_s) {
        effect = state
            .stop(phase, reason.into())
            .map_err(|_| error("replay_core_divergence", Some(terminal.seq)))?;
    }
    let (phase, reason) = match &effect {
        Effect::Candidate(candidate) => {
            ensure(
                observed_candidate.as_ref() == Some(candidate),
                "replay_candidate_divergence",
                Some(terminal.seq),
            )?;
            (RunPhase::Completed, "answer_candidate")
        }
        Effect::Stop { phase, reason } => (*phase, reason.as_str()),
        _ => {
            return Err(error(
                "replay_terminal_decision_unavailable",
                Some(terminal.seq),
            ));
        }
    };
    ensure(
        terminal.kind == "run_finished"
            && terminal.data["phase"] == phase.as_str()
            && run.phase == phase.as_str()
            && run.terminal_reason.as_deref() == Some(reason)
            && terminal.data["reason"] == reason,
        "replay_terminal_divergence",
        Some(terminal.seq),
    )?;
    let mut receipt = if phase == RunPhase::Completed {
        verification::assess_context(
            &frozen,
            observed_candidate.as_deref().unwrap_or(""),
            &evidence,
        )
    } else {
        verification::inconclusive_context(&frozen, observed_candidate.as_deref(), reason)
    };
    let recorded = run
        .receipt
        .as_ref()
        .ok_or_else(|| error("replay_receipt_missing", Some(terminal.seq)))?;
    receipt.duration_ms = recorded["duration_ms"]
        .as_u64()
        .ok_or_else(|| error("replay_receipt_divergence", Some(terminal.seq)))?;
    let recomputed = json!(receipt);
    let candidate_digest = observed_candidate
        .as_deref()
        .map(|candidate| model::fingerprint(candidate.as_bytes()));
    let result = observed_candidate.as_ref().map(|candidate| {
        let mut result = json!({"candidate":candidate});
        if let Some(fields) = receipt.verified_fields() {
            result["verified_fields"] = json!(fields);
        }
        result
    });
    ensure(
        run.result == result
            && run.result_sha256 == candidate_digest
            && terminal.data["candidate_sha256"] == json!(candidate_digest),
        "replay_candidate_divergence",
        Some(terminal.seq),
    )?;
    ensure(
        recorded == &recomputed
            && terminal.data["receipt"] == recomputed
            && run.acceptance_status == receipt.status
            && terminal.data["acceptance_status"] == receipt.status,
        "replay_receipt_divergence",
        Some(terminal.seq),
    )?;
    ensure(
        terminal.data["counters"]
            == json!({"model_turns":state.counters().model_turns,"tool_calls":state.counters().tool_calls}),
        "replay_counter_divergence",
        Some(terminal.seq),
    )?;
    state
        .commit_terminal(phase, receipt.acceptance_status())
        .map_err(|_| error("replay_core_divergence", Some(terminal.seq)))?;
    Ok(ReplayReport {owner_id:run.owner_id.clone(),run_id:run.run_id.clone(),consistency:"consistent".into(),
        phase:run.phase.clone(),acceptance_status:run.acceptance_status.clone(),event_count:events.len(),
        model_requests:state.counters().model_turns,tool_observations:observations,prepared_sha256:prepared_hashes,
        verification_replayed:frozen.task.is_checked() && phase==RunPhase::Completed,
        scope:"Internal consistency of captured inputs, recorded dispatch/cancellation/deadline arbitration and checker decisions only; no origin authentication, live authority, current-world claim, effect execution, or UI timing replay.".into()})
}

fn history_exceeds(state: &core::RunState, limit: usize) -> Result<bool, ReplayError> {
    serde_json::to_vec(&model::conversation_json(state.messages()))
        .map(|bytes| bytes.len() > limit)
        .map_err(|_| error("replay_history_encoding", None))
}
fn stop_state(state: &mut core::RunState, reason: &str) -> Result<Effect, ReplayError> {
    state
        .stop(RunPhase::Stopped, reason.into())
        .map_err(|_| error("replay_core_divergence", None))
}
fn bounded_serialized(value: &impl Serialize, limit: usize) -> Result<(), ReplayError> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit - self.bytes {
                return Err(std::io::Error::other("replay limit"));
            }
            self.bytes += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(&mut Counter { bytes: 0, limit }, value)
        .map_err(|_| error("replay_bytes_limit", None))
}
