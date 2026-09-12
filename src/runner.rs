//! One owner performs and records a run's effects in order.

use crate::config::{CaptureMode, ConcurrencyConfig, Config, ToolName, ToolRef};
use crate::core::{self, AcceptanceStatus, Effect, ModelReply, RunPhase};
use crate::dispatch::{ModelDispatcher, ModelPermit};
use crate::model::{self, ModelClient, ModelOptions, TextObserver};
use crate::policy::{RunAuthority, TaskContract};
use crate::replay::{ControlObservation, QueueStop, QueueStopKind};
use crate::storage::{
    self, Admission, Command, Event, PendingAck, Response, RunRecord, StorageClient, TaskIdentity,
    Terminal,
};
use crate::tools::{
    CommandRunner, ToolResult, ToolStatus, TypedToolArgs, WorkspaceReader, WorkspaceWriter,
};
use crate::verification::{self, AcceptanceReceipt, EvidenceInventory};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tokio::time::{Instant, timeout_at};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct RunResources {
    pub models: Arc<Semaphore>,
    pub tools: Arc<Semaphore>,
    workspaces: Arc<BTreeMap<String, Arc<WorkspaceReader>>>,
    /// Present only for workspaces whose operator enabled a write tool, so a run
    /// without that grant has no writer to reach.
    writers: Arc<BTreeMap<String, Arc<WorkspaceWriter>>>,
    /// Present only for workspaces whose operator granted `run_command` with an
    /// allow-list, so a run without that grant has no command runner to reach.
    command_runners: Arc<BTreeMap<String, Arc<CommandRunner>>>,
    /// Present only for a run that allow-lists MCP tools: the live stdio sessions
    /// discovered and opened at run start, held for the run. Per-run, not shared
    /// across the config's models like the other backends.
    mcp: Option<Arc<crate::mcp::McpClientPool>>,
    dispatcher: Option<Arc<ModelDispatcher>>,
    pub concurrency: ConcurrencyConfig,
}

impl RunResources {
    pub fn single(profile_slots: usize, concurrency: ConcurrencyConfig) -> Self {
        Self {
            models: Arc::new(Semaphore::new(
                profile_slots.min(concurrency.max_inflight_model_requests),
            )),
            tools: Arc::new(Semaphore::new(concurrency.max_blocking_tools)),
            workspaces: Arc::new(BTreeMap::new()),
            writers: Arc::new(BTreeMap::new()),
            command_runners: Arc::new(BTreeMap::new()),
            mcp: None,
            dispatcher: None,
            concurrency,
        }
    }

    /// Attach a per-run MCP client pool (trusted startup / run preparation only).
    /// The pool is discovered and opened before admission and lives for the run.
    pub fn with_mcp(mut self, pool: Arc<crate::mcp::McpClientPool>) -> Self {
        self.mcp = Some(pool);
        self
    }

    /// Trusted startup only. Run inputs can select aliases, never open roots.
    pub fn with_workspace(mut self, alias: &str, root: &Path) -> Result<Self, String> {
        Arc::make_mut(&mut self.workspaces)
            .insert(alias.into(), Arc::new(WorkspaceReader::open(root)?));
        Ok(self)
    }

    /// Trusted startup only. Grants writes as well as reads for one workspace,
    /// mirroring an operator who listed a write tool.
    pub fn with_write_workspace(mut self, alias: &str, root: &Path) -> Result<Self, String> {
        self = self.with_workspace(alias, root)?;
        Arc::make_mut(&mut self.writers)
            .insert(alias.into(), Arc::new(WorkspaceWriter::open(root)?));
        Ok(self)
    }

    /// Trusted startup only. Grants the command tool for one workspace with its
    /// allow-list, mirroring an operator who listed `run_command`.
    pub fn with_command_workspace(
        mut self,
        alias: &str,
        root: &Path,
        allowed: Vec<String>,
    ) -> Result<Self, String> {
        self = self.with_workspace(alias, root)?;
        Arc::make_mut(&mut self.command_runners)
            .insert(alias.into(), Arc::new(CommandRunner::new(root, allowed)));
        Ok(self)
    }

    pub fn from_config(config: &Config) -> Result<BTreeMap<String, Self>, String> {
        let concurrency = config.concurrency().clone();
        let tools = Arc::new(Semaphore::new(concurrency.max_blocking_tools));
        let workspaces = Arc::new(
            config
                .workspaces()
                .iter()
                .filter(|workspace| !workspace.tools.is_empty())
                .map(|workspace| {
                    Ok((
                        workspace.id.clone(),
                        Arc::new(WorkspaceReader::open(&workspace.root)?),
                    ))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()?,
        );
        let writers = Arc::new(
            config
                .workspaces()
                .iter()
                .filter(|workspace| workspace.tools.iter().any(|tool| tool.is_mutating()))
                .map(|workspace| {
                    Ok((
                        workspace.id.clone(),
                        Arc::new(WorkspaceWriter::open(&workspace.root)?),
                    ))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()?,
        );
        // A command runner exists only for a workspace the operator granted
        // run_command; config validation guarantees such a workspace has a
        // non-empty allow-list.
        let command_runners = Arc::new(
            config
                .workspaces()
                .iter()
                .filter(|workspace| {
                    workspace
                        .tools
                        .contains(&ToolRef::Compiled(ToolName::RunCommand))
                })
                .map(|workspace| {
                    (
                        workspace.id.clone(),
                        Arc::new(CommandRunner::new(
                            &workspace.root,
                            workspace.commands.clone(),
                        )),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        );
        // Aliases sharing an origin cannot multiply the backend's actual slots.
        let mut capacities = BTreeMap::<String, usize>::new();
        for model in config.models() {
            capacities
                .entry(model.base_url.clone())
                .and_modify(|slots| *slots = (*slots).min(model.verified_slots))
                .or_insert(model.verified_slots);
        }
        let backends: BTreeMap<_, _> = capacities
            .into_iter()
            .map(|(origin, slots)| (origin, Arc::new(Semaphore::new(slots))))
            .collect();
        let dispatcher = ModelDispatcher::new(
            concurrency.max_inflight_model_requests,
            backends.clone(),
            concurrency.max_active_runs,
        )?;
        Ok(config
            .models()
            .iter()
            .map(|model| {
                (
                    model.id.clone(),
                    Self {
                        models: backends[&model.base_url].clone(),
                        tools: tools.clone(),
                        workspaces: workspaces.clone(),
                        writers: writers.clone(),
                        command_runners: command_runners.clone(),
                        mcp: None,
                        dispatcher: Some(dispatcher.clone()),
                        concurrency: concurrency.clone(),
                    },
                )
            })
            .collect())
    }

    async fn model_capacity(&self, authority: &RunAuthority) -> Result<ModelCapacity, String> {
        match &self.dispatcher {
            Some(dispatcher) => dispatcher
                .acquire(authority.owner(), &authority.model().base_url)
                .await
                .map(ModelCapacity::Shared),
            None => self
                .models
                .clone()
                .acquire_owned()
                .await
                .map(ModelCapacity::Single)
                .map_err(|_| "model_capacity_closed".into()),
        }
    }
}

enum ModelCapacity {
    Single(tokio::sync::OwnedSemaphorePermit),
    Shared(ModelPermit),
}

impl Drop for ModelCapacity {
    fn drop(&mut self) {
        // Explicitly visit both RAII owners; the fields release on enum drop.
        match self {
            Self::Single(permit) => {
                let _ = permit;
            }
            Self::Shared(permit) => {
                let _ = permit;
            }
        }
    }
}

/// A constrained turn withdraws tools: the server installs its own grammar for
/// tool calls from the chat template, so the two cannot share one request.
pub fn finalize_options(authority: &RunAuthority) -> ModelOptions {
    let mut options = options(authority);
    if let TaskContract::FileFieldsV1(profile) = authority.task() {
        options.constraint = Some(verification::candidate_schema(profile));
        options.tools.clear();
    }
    options
}

pub fn options(authority: &RunAuthority) -> ModelOptions {
    ModelOptions {
        origin: authority.model().base_url.clone(),
        served_model: authority.model().model_id.clone(),
        temperature: authority.model().temperature,
        max_output_tokens: authority.limits().max_output_tokens as usize,
        max_request_bytes: authority.limits().max_request_bytes,
        max_response_bytes: authority.limits().max_response_bytes,
        stream: authority.model().stream,
        cache_prompt: authority.model().cache_prompt,
        tools: model::tool_defs(&authority.workspace().tools, authority.mcp_tools()),
        constraint: None,
    }
}

pub async fn admit(
    authority: &RunAuthority,
    store: &StorageClient,
    key: Option<String>,
) -> Result<RunRecord, String> {
    admit_once(authority, store, key)
        .await
        .map(|(record, _)| record)
}

pub async fn admit_once(
    authority: &RunAuthority,
    store: &StorageClient,
    key: Option<String>,
) -> Result<(RunRecord, bool), String> {
    let mut data = json!({"input_sources":authority.input_sources(),"limits":authority.limits(),"policy_version":"1"});
    data["versions"] = crate::replay::versions();
    if authority.capture() == CaptureMode::Replay {
        data["replay"] = json!({"authority":authority});
    }
    let admission = Admission {
        run_id: authority.run_id().into(),
        owner_id: authority.owner().into(),
        workspace_id: authority.workspace().id.clone(),
        model_profile_id: authority.model().id.clone(),
        task: match authority.task() {
            TaskContract::Freeform => None,
            TaskContract::FileFieldsV1(profile) => Some(TaskIdentity {
                profile_id: profile.id.clone(),
                profile_version: profile.version.to_string(),
                spec_sha256: authority.task_spec_sha256().into(),
                checker_id: profile.checker.clone(),
                checker_version: profile.checker_version.to_string(),
                criterion_ids: profile.criteria.iter().map(|c| c.id.clone()).collect(),
            }),
        },
        capture: match authority.capture() {
            CaptureMode::Metadata => "metadata",
            CaptureMode::Replay => "replay",
        }
        .into(),
        created_unix_ms: i64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "invalid system clock")?
                .as_millis(),
        )
        .map_err(|_| "system clock overflow")?,
        policy_version: "1".into(),
        submission_sha256: authority.submission_sha256().into(),
        idempotency_key: key,
        data,
    };
    match store
        .execute(
            Command::Admit(admission),
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .map_err(|e| e.to_string())?
    {
        Response::Admitted { record, created } => Ok((*record, created)),
        _ => Err("unexpected admission acknowledgement".into()),
    }
}

struct RunJournal {
    store: StorageClient,
    owner: String,
    run: String,
    accepted: Instant,
    deadline: Instant,
    grace: Option<Instant>,
    grace_duration: Duration,
    admission_timeout: Duration,
    seq: u64,
    late_bookkeeping: bool,
    cancel: CancellationToken,
    model_turns: usize,
    tool_calls: usize,
    // Exact totals become unknown permanently if any dispatched call omits a
    // component or its sum overflows. Each component has independent coverage.
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    compactions: u64,
    journal_wait_ms: u64,
    capacity_stopped: bool,
    queue_stop: Option<QueueStop>,
}

impl RunJournal {
    /// Per-run counters. A token total requires complete dispatched-call
    /// coverage and an exact sum. No dispatch is unknown rather than zero.
    fn counters(&self) -> Value {
        let mut counters = json!({"model_turns": self.model_turns, "tool_calls": self.tool_calls});
        if self.model_turns > 0 {
            if let Some(tokens) = self.prompt_tokens {
                counters["prompt_tokens"] = json!(tokens);
            }
            if let Some(tokens) = self.completion_tokens {
                counters["completion_tokens"] = json!(tokens);
            }
        }
        // Present only when the run actually compacted, so an ordinary run's
        // counters are unchanged and old replay captures never carry it.
        if self.compactions > 0 {
            counters["compactions"] = json!(self.compactions);
        }
        counters
    }

    fn stop(&mut self) {
        if self.grace.is_none() {
            self.grace = Some(Instant::now() + self.grace_duration);
        }
    }
    fn admission_deadline(&self) -> Instant {
        if self.late_bookkeeping {
            return Instant::now() + self.admission_timeout;
        }
        (Instant::now() + self.admission_timeout).min(self.grace.unwrap_or(self.deadline))
    }
    fn requested_stop(&self) -> Option<(RunPhase, &'static str)> {
        stop_requested(&self.cancel, self.deadline).or_else(|| {
            self.capacity_stopped
                .then_some((RunPhase::Stopped, "journal_admission_timeout"))
        })
    }
    fn control(&self) -> ControlObservation {
        ControlObservation {
            elapsed_us: self
                .accepted
                .elapsed()
                .as_micros()
                .min(u128::from(u64::MAX)) as u64,
            cancelled: self.cancel.is_cancelled(),
            journal_stopped: self.capacity_stopped,
        }
    }
    async fn reserve(&mut self, bytes: usize) -> Result<storage::Reservation, String> {
        let store = self.store.clone();
        let reservation = store.reserve_owned(bytes);
        tokio::pin!(reservation);
        if self.grace.is_none() {
            tokio::select! {
                biased;
                result = &mut reservation => return result.map_err(|e| e.to_string()),
                _ = self.cancel.cancelled() => self.stop(),
                _ = tokio::time::sleep_until(self.admission_deadline()) => {
                    self.capacity_stopped = true;
                    self.stop();
                },
            }
        }
        match timeout_at(
            self.grace.ok_or("missing settlement grace")?,
            &mut reservation,
        )
        .await
        {
            Ok(result) => result.map_err(|e| e.to_string()),
            Err(_) => {
                if !self.late_bookkeeping {
                    eprintln!(
                        "Run {} has unresolved journal capacity; retaining ownership",
                        self.run
                    );
                }
                // No command has transferred yet. Keep this same bounded owner
                // and its reservation future; do not retry or resume execution.
                self.late_bookkeeping = true;
                reservation.await.map_err(|e| e.to_string())
            }
        }
    }
    fn event(&self, kind: &str, data: Value) -> Event {
        Event {
            seq: self.seq,
            kind: kind.into(),
            elapsed_ms: self
                .accepted
                .elapsed()
                .as_millis()
                .min(u128::from(u64::MAX)) as u64,
            data,
        }
    }
    async fn record(&mut self, kind: &str, data: Value, phase: Option<&str>) -> Result<(), String> {
        let started = Instant::now();
        let command = Command::Append {
            owner_id: self.owner.clone(),
            run_id: self.run.clone(),
            event: self.event(kind, data),
            phase: phase.map(str::to_owned),
        };
        let bytes = command.encoded_len().map_err(|e| e.to_string())?;
        let reservation = self.reserve(bytes).await?;
        let pending = reservation.submit(command).map_err(|e| e.to_string())?;
        self.settle(pending).await?;
        self.journal_wait_ms = self
            .journal_wait_ms
            .saturating_add(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
        self.seq += 1;
        Ok(())
    }

    async fn settle(&mut self, pending: PendingAck) -> Result<Response, String> {
        let acknowledgement = pending.wait();
        tokio::pin!(acknowledgement);
        if self.grace.is_none() {
            tokio::select! {
                biased;
                result = &mut acknowledgement => return result.map_err(|e|e.to_string()),
                _ = self.cancel.cancelled() => self.stop(),
                _ = tokio::time::sleep_until(self.deadline) => self.stop(),
            }
        }
        match timeout_at(
            self.grace.ok_or("missing settlement grace")?,
            &mut acknowledgement,
        )
        .await
        {
            Ok(result) => result.map_err(|e| e.to_string()),
            Err(_) => {
                eprintln!(
                    "Run {} has unresolved journal settlement; retaining ownership",
                    self.run
                );
                // The command may still commit. Do not drop its owner or send
                // a replacement, and do not release active/model capacity early.
                let result = acknowledgement.await.map_err(|e| e.to_string());
                self.late_bookkeeping = true;
                result
            }
        }
    }
    fn terminal_command(
        &self,
        outcome: &RunOutcome,
        receipt: &AcceptanceReceipt,
        control: &ControlObservation,
    ) -> Command {
        let digest = outcome.candidate.as_deref().map(storage::candidate_digest);
        let result = outcome.candidate.as_ref().map(|candidate| {
            let mut value = json!({"candidate":candidate});
            if let Some(fields) = receipt.verified_fields() {
                value["verified_fields"] = json!(fields);
            }
            value
        });
        Command::Finish {owner_id:self.owner.clone(),run_id:self.run.clone(),
            event:self.event("run_finished",json!({"phase":outcome.phase.as_str(),"acceptance_status":outcome.acceptance.as_str(),"reason":outcome.reason,"candidate_sha256":digest,
                "counters":self.counters(),"journal_wait_before_terminal_ms":self.journal_wait_ms,
                "control_terminal":control,"queue_stop":self.queue_stop})),
            terminal:Terminal {phase:outcome.phase.as_str().into(),reason:outcome.reason.clone(),
                result,
                result_sha256:digest,acceptance_status:outcome.acceptance.as_str().into(),receipt:json!(receipt)},
        }
    }
    async fn finalize(
        &mut self,
        mut outcome: RunOutcome,
        state: &mut core::RunState,
        authority: &RunAuthority,
        evidence: &EvidenceInventory,
    ) -> Result<RunRecord, String> {
        self.model_turns = state.counters().model_turns;
        self.tool_calls = state.counters().tool_calls;
        if let Some((phase, reason)) = self.requested_stop() {
            self.stop();
            outcome.phase = phase;
            outcome.reason = reason.into();
            state
                .stop(phase, reason.into())
                .map_err(|e| e.to_string())?;
        }
        let check_start = Instant::now();
        let mut receipt = if outcome.phase == RunPhase::Completed {
            verification::assess(
                authority,
                outcome.candidate.as_deref().unwrap_or(""),
                evidence,
            )
        } else {
            verification::inconclusive(authority, outcome.candidate.as_deref(), &outcome.reason)
        };
        receipt.duration_ms = check_start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        outcome.acceptance = receipt.acceptance_status();
        let command = self.terminal_command(&outcome, &receipt, &self.control());
        // Reserve room for any bounded receipt/reason selected after this wait.
        let reservation_bytes = command
            .encoded_len()
            .map_err(|e| e.to_string())?
            .checked_add(32768)
            .ok_or("terminal size overflow")?;
        let reservation = self.reserve(reservation_bytes).await?;
        let control = self.control();
        if let Some((phase, reason)) = control.stop(authority.limits().max_run_s) {
            self.stop();
            outcome.phase = phase;
            outcome.reason = reason.into();
            state
                .stop(phase, reason.into())
                .map_err(|e| e.to_string())?;
            receipt = verification::inconclusive(authority, outcome.candidate.as_deref(), reason);
            outcome.acceptance = receipt.acceptance_status();
        }
        // No await from the final arbitration through transfer into the inbox.
        let acknowledgement = reservation
            .submit(self.terminal_command(&outcome, &receipt, &control))
            .map_err(|e| e.to_string())?;
        self.settle(acknowledgement).await?;
        state
            .commit_terminal(outcome.phase, outcome.acceptance)
            .map_err(|e| e.to_string())?;
        match self
            .store
            .execute(
                Command::Get {
                    owner_id: self.owner.clone(),
                    run_id: self.run.clone(),
                },
                Instant::now() + self.admission_timeout,
            )
            .await
            .map_err(|e| e.to_string())?
        {
            Response::Run(Some(record)) => Ok(*record),
            _ => Err("committed run could not be retrieved".into()),
        }
    }
}

fn stop_requested(
    cancel: &CancellationToken,
    deadline: Instant,
) -> Option<(RunPhase, &'static str)> {
    if cancel.is_cancelled() {
        Some((RunPhase::Cancelled, "cancelled"))
    } else if Instant::now() >= deadline {
        Some((RunPhase::Stopped, "run_deadline"))
    } else {
        None
    }
}

fn observed_json(reply: &ModelReply) -> Value {
    match reply {
        ModelReply::Answer(text) => json!({"kind":"answer","text":text}),
        ModelReply::Incomplete(reason) => json!({"kind":"incomplete","reason":reason}),
        ModelReply::Failure(reason) => json!({"kind":"failure","reason":reason}),
        ModelReply::ToolCalls { content, calls } => {
            json!({"kind":"tool_calls","content":content,"calls":calls.iter().map(|c|json!({"id":c.id,"name":c.name,"arguments":c.arguments})).collect::<Vec<_>>()})
        }
    }
}

/// The controller retains this future through terminal acknowledgement. Callers
/// must not detach/abort it to implement a request timeout or disconnect.
pub async fn run_admitted(
    authority: RunAuthority,
    client: ModelClient,
    store: StorageClient,
    resources: RunResources,
    cancel: CancellationToken,
    accepted: Instant,
) -> Result<RunRecord, String> {
    run_admitted_with_text(authority, client, store, resources, cancel, accepted, None).await
}

pub async fn run_admitted_with_text(
    authority: RunAuthority,
    client: ModelClient,
    store: StorageClient,
    resources: RunResources,
    cancel: CancellationToken,
    accepted: Instant,
    mut observer: Option<TextObserver>,
) -> Result<RunRecord, String> {
    let deadline = accepted + Duration::from_secs(authority.limits().max_run_s);
    let mut journal = RunJournal {
        store,
        owner: authority.owner().into(),
        run: authority.run_id().into(),
        accepted,
        deadline,
        grace: None,
        grace_duration: Duration::from_secs(resources.concurrency.settlement_grace_s),
        admission_timeout: Duration::from_secs(resources.concurrency.journal_admission_timeout_s),
        seq: 1,
        late_bookkeeping: false,
        cancel: cancel.clone(),
        model_turns: 0,
        tool_calls: 0,
        prompt_tokens: Some(0),
        completion_tokens: Some(0),
        compactions: 0,
        journal_wait_ms: 0,
        capacity_stopped: false,
        queue_stop: None,
    };
    let (mut state, mut effect) = core::initiate_continued(
        authority.instructions().into(),
        authority.prior().map(|prior| prior.answer.clone()),
        authority.prompt().into(),
        !authority.workspace().tools.is_empty(),
    )
    .map_err(|e| e.to_string())?;
    if authority.task().is_checked() {
        state
            .require_acceptance_check()
            .map_err(|e| e.to_string())?;
    }
    if journal.requested_stop().is_some() {
        journal.stop();
    }
    journal
        .record(
            "run_started",
            json!({"queue_ms":accepted.elapsed().as_millis().min(u128::from(u64::MAX)) as u64}),
            Some("running"),
        )
        .await?;
    let settings = options(&authority);
    // Built once: the constrained turn is a different request shape, not a
    // mutation of the gather turn's settings.
    let finalize_settings = finalize_options(&authority);
    let mut observed_candidate = None;
    let mut evidence = EvidenceInventory::default();
    let mut previous_batch = None;
    let mut repeat_count = 0;
    if client
        .origin()
        .is_some_and(|origin| origin != authority.model().base_url)
    {
        journal.stop();
        effect = state
            .stop(RunPhase::Failed, "model_authority_mismatch".into())
            .map_err(|e| e.to_string())?;
    }
    'run: loop {
        if let Some((phase, reason)) = journal.requested_stop() {
            journal.stop();
            effect = state
                .stop(phase, reason.into())
                .map_err(|e| e.to_string())?;
        }
        match std::mem::replace(
            &mut effect,
            Effect::Stop {
                phase: RunPhase::Failed,
                reason: "missing_next_effect".into(),
            },
        ) {
            Effect::Model => {
                if state.counters().model_turns >= authority.limits().max_model_turns as usize {
                    journal.stop();
                    effect = state
                        .stop(RunPhase::Stopped, "model_turn_limit".into())
                        .map_err(|e| e.to_string())?;
                    continue;
                }
                if !compact_state(&mut state, &mut journal, authority.limits())? {
                    journal.stop();
                    effect = state
                        .stop(RunPhase::Stopped, "history_bytes_limit".into())
                        .map_err(|e| e.to_string())?;
                    continue;
                }
                let turn_settings = if state.finalizing() {
                    &finalize_settings
                } else {
                    &settings
                };
                let prepared = match model::prepare(state.messages(), turn_settings) {
                    Ok(request) => request,
                    Err(reason) => {
                        journal.stop();
                        effect = state
                            .stop(RunPhase::Stopped, reason)
                            .map_err(|e| e.to_string())?;
                        continue;
                    }
                };
                let queue_start = Instant::now();
                let queue_deadline = deadline.min(
                    queue_start + Duration::from_secs(authority.model().model_queue_timeout_s),
                );
                let permit = tokio::select! {
                    biased;
                    _=cancel.cancelled()=>Err(None),
                    permit=timeout_at(queue_deadline,resources.model_capacity(&authority))=>match permit {
                        Ok(Ok(permit))=>Ok(permit),
                        Ok(Err(_))=>Err(Some(QueueStopKind::Unavailable)),
                        Err(_)=>Err(Some(QueueStopKind::Timeout)),
                    },
                };
                let permit = match permit {
                    Ok(permit) => permit,
                    Err(queue_kind) => {
                        journal.queue_stop = queue_kind.map(|kind| QueueStop {
                            started_us: queue_start
                                .duration_since(accepted)
                                .as_micros()
                                .min(u128::from(u64::MAX))
                                as u64,
                            finished_us: accepted.elapsed().as_micros().min(u128::from(u64::MAX))
                                as u64,
                            deadline_us: queue_deadline
                                .duration_since(accepted)
                                .as_micros()
                                .min(u128::from(u64::MAX))
                                as u64,
                            kind,
                        });
                        journal.stop();
                        let (phase, reason) = journal
                            .requested_stop()
                            .unwrap_or((RunPhase::Stopped, "model_queue_timeout"));
                        effect = state
                            .stop(phase, reason.into())
                            .map_err(|e| e.to_string())?;
                        continue;
                    }
                };
                let dispatch_control = journal.control();
                if let Some((phase, reason)) = dispatch_control.stop(authority.limits().max_run_s) {
                    drop(permit);
                    journal.stop();
                    effect = state
                        .stop(phase, reason.into())
                        .map_err(|e| e.to_string())?;
                    continue;
                }
                let effect_id = format!("model-{}", state.counters().model_turns);
                journal.record("model_planned",json!({"effect_id":effect_id,"request_sha256":prepared.sha256(),"request_bytes":prepared.bytes().len(),"model_profile":authority.model().id,
                    "queue_ms":queue_start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64}),None).await?;
                let dispatch_control = journal.control();
                if let Some((phase, reason)) = dispatch_control.stop(authority.limits().max_run_s) {
                    journal.stop();
                    journal
                        .record(
                            "model_finished",
                            json!({"effect_id":effect_id,"dispatch":"unsent","reason":reason,"control_dispatch":dispatch_control}),
                            None,
                        )
                        .await?;
                    drop(permit);
                    effect = state
                        .stop(phase, reason.into())
                        .map_err(|e| e.to_string())?;
                    continue;
                }
                state.model_started().map_err(|e| e.to_string())?;
                let exchange_start = Instant::now();
                let exchange = async {
                    match observer.as_mut() {
                        Some(observer) => client.send_with_text(&prepared, observer).await,
                        None => client.send(&prepared).await,
                    }
                };
                let (observation, usage) = tokio::select! {
                    biased;
                    _=cancel.cancelled()=>(ModelReply::Failure("cancelled_remote_outcome_unknown".into()), None),
                    reply=timeout_at(deadline,exchange)=>match reply {
                        Ok(Ok(outcome))=>(outcome.reply, outcome.usage),
                        Ok(Err(reason))=>(ModelReply::Failure(reason), None),
                        Err(_)=>(ModelReply::Failure("run_deadline_remote_outcome_unknown".into()), None),
                    }
                };
                if journal.requested_stop().is_some() {
                    journal.stop();
                }
                if let ModelReply::Answer(candidate) = &observation {
                    observed_candidate = Some(candidate.clone());
                }
                let mut metadata = json!({"effect_id":effect_id,"dispatch":"attempted","duration_ms":exchange_start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                    "control_dispatch":dispatch_control,
                    "classification":match &observation {ModelReply::Answer(_)=>"answer",ModelReply::ToolCalls{..}=>"tool_calls",ModelReply::Incomplete(_)=>"incomplete",ModelReply::Failure(_)=>"failure"}});
                let prompt_tokens = usage.and_then(|usage| usage.prompt_tokens);
                let completion_tokens = usage.and_then(|usage| usage.completion_tokens);
                if let Some(tokens) = prompt_tokens {
                    metadata["prompt_tokens"] = json!(tokens);
                }
                if let Some(tokens) = completion_tokens {
                    metadata["completion_tokens"] = json!(tokens);
                }
                journal.prompt_tokens = journal
                    .prompt_tokens
                    .zip(prompt_tokens)
                    .and_then(|(total, tokens)| total.checked_add(tokens));
                journal.completion_tokens = journal
                    .completion_tokens
                    .zip(completion_tokens)
                    .and_then(|(total, tokens)| total.checked_add(tokens));
                if authority.capture() == CaptureMode::Replay {
                    metadata["replay"] = observed_json(&observation);
                }
                journal.record("model_finished", metadata, None).await?;
                drop(permit);
                if let ModelReply::ToolCalls { calls, .. } = &observation {
                    if calls.len()
                        > (authority.limits().max_tool_calls as usize)
                            .saturating_sub(state.counters().tool_calls)
                    {
                        journal.stop();
                        effect = state
                            .stop(RunPhase::Stopped, "tool_call_limit".into())
                            .map_err(|e| e.to_string())?;
                        continue;
                    }
                    let batch: Vec<_> = calls
                        .iter()
                        .map(|call| {
                            let args = serde_json::from_str::<Value>(&call.arguments)
                                .unwrap_or_else(|_| Value::String(call.arguments.clone()));
                            json!({"name":call.name,"arguments":args})
                        })
                        .collect();
                    if previous_batch.as_ref() == Some(&batch) {
                        repeat_count += 1;
                    } else {
                        previous_batch = Some(batch);
                        repeat_count = 1;
                    }
                    if repeat_count >= authority.limits().repeat_limit {
                        journal.stop();
                        effect = state
                            .stop(RunPhase::Stopped, "repeat_limit".into())
                            .map_err(|e| e.to_string())?;
                        continue;
                    }
                }
                let mut next = state.clone();
                let next_effect = next.observe_model(observation).map_err(|e| e.to_string())?;
                let compaction = authority.limits().compaction;
                let (dropped, fits) = model::compact_until_fits(
                    &mut next,
                    authority.limits().max_history_bytes,
                    compaction.enabled,
                    compaction.floor,
                )?;
                if !fits {
                    journal.stop();
                    effect = state
                        .stop(RunPhase::Stopped, "history_bytes_limit".into())
                        .map_err(|e| e.to_string())?;
                } else {
                    // Only count drops that persist: an unfit `next` is discarded.
                    journal.compactions += dropped as u64;
                    state = next;
                    effect = next_effect;
                }
            }
            Effect::Candidate(candidate) => {
                return journal
                    .finalize(
                        RunOutcome {
                            phase: RunPhase::Completed,
                            acceptance: AcceptanceStatus::Unchecked,
                            candidate: Some(candidate),
                            reason: "answer_candidate".into(),
                        },
                        &mut state,
                        &authority,
                        &evidence,
                    )
                    .await;
            }
            Effect::Stop { phase, reason } => {
                journal.stop();
                return journal
                    .finalize(
                        RunOutcome {
                            phase,
                            acceptance: AcceptanceStatus::Unchecked,
                            candidate: observed_candidate,
                            reason,
                        },
                        &mut state,
                        &authority,
                        &evidence,
                    )
                    .await;
            }
            Effect::Tools(calls) => {
                let first_index = state.counters().tool_calls - calls.len();
                for (index, call) in calls.into_iter().enumerate() {
                    if let Some((phase, reason)) = journal.requested_stop() {
                        journal.stop();
                        effect = state
                            .stop(phase, reason.into())
                            .map_err(|e| e.to_string())?;
                        continue 'run;
                    }
                    let effect_id = format!("tool-{}", first_index + index);
                    let tool = ToolName::from_wire(&call.name);
                    // An MCP tool is recognized only if it is in the frozen set —
                    // discovered at run start for an allow-listed, operator-declared
                    // server. Discovery and a server's schema never grant authority.
                    let mcp_def = if tool.is_none() {
                        authority
                            .mcp_tools()
                            .iter()
                            .find(|def| def.wire_name() == call.name)
                            .cloned()
                    } else {
                        None
                    };
                    let args = TypedToolArgs::parse(&call.arguments);
                    // Validate MCP arguments against the discovered schema up front,
                    // so an invalid or unapproved MCP call is denied before any
                    // server is contacted.
                    let mcp_args = mcp_def.as_ref().and_then(|def| {
                        crate::mcp::validate_args(&call.arguments, &def.input_schema)
                    });
                    let denial = if let Some(name) = tool {
                        if !authority.allows_tool(&ToolRef::Compiled(name)) {
                            Some(ToolResult::failure(
                                ToolStatus::Denied,
                                "tool_denied",
                                "Tool is not allowed",
                            ))
                        } else if args.is_err() {
                            Some(ToolResult::failure(
                                ToolStatus::Denied,
                                "invalid_arguments",
                                "Tool arguments or resource path are invalid",
                            ))
                        } else {
                            None
                        }
                    } else if mcp_def.is_some() {
                        if mcp_args.is_none() {
                            Some(ToolResult::failure(
                                ToolStatus::Denied,
                                "invalid_arguments",
                                "Tool arguments or resource path are invalid",
                            ))
                        } else {
                            None
                        }
                    } else if call.name.starts_with(crate::config::MCP_TOOL_PREFIX) {
                        // A well-formed mcp__ name the run did not grant: denied, not
                        // "unknown", and never dispatched to a server.
                        Some(ToolResult::failure(
                            ToolStatus::Denied,
                            "tool_denied",
                            "Tool is not allowed",
                        ))
                    } else {
                        Some(ToolResult::failure(
                            ToolStatus::Denied,
                            "unknown_tool",
                            "Tool is not available",
                        ))
                    };
                    // Pair the MCP definition with its validated arguments for
                    // dispatch; only reachable when `denial` is None.
                    let mcp_ready = match (mcp_def, mcp_args) {
                        (Some(def), Some(arguments)) => Some((def, arguments)),
                        _ => None,
                    };
                    let mut planned = json!({"effect_id":effect_id,"call_id":call.id,
                        "model_effect_id":format!("model-{}",state.counters().model_turns-1),
                        "tool":call.name,"arguments_sha256":model::fingerprint(call.arguments.as_bytes()),
                        "dispatch":if denial.is_some(){"denied"}else{"pending"}});
                    if authority.capture() == CaptureMode::Replay {
                        planned["replay"] = json!({"arguments":call.arguments});
                    }
                    journal.record("tool_planned", planned, None).await?;
                    let tool_start = Instant::now();
                    let mut dispatched = false;
                    let mut dispatch_control = journal.control();
                    let result = if let Some(denial) = denial {
                        denial
                    } else {
                        let permit = tokio::select! {
                            biased;
                            _=cancel.cancelled()=>None,
                            permit=timeout_at(deadline, resources.tools.clone().acquire_owned())=>permit.ok().and_then(Result::ok),
                        };
                        dispatch_control = journal.control();
                        if let Some(permit) = permit.filter(|_| {
                            dispatch_control
                                .stop(authority.limits().max_run_s)
                                .is_none()
                        }) {
                            if let Some((def, arguments)) = mcp_ready {
                                // An MCP tool executes over its stdio session, not a
                                // workspace capability. Its result is an untrusted
                                // observation bounded by the same tool-result budget;
                                // it mints no evidence. The permit is held for the call.
                                let _permit = permit;
                                dispatched = true;
                                let timeout = deadline.saturating_duration_since(Instant::now());
                                match &resources.mcp {
                                    Some(pool) => {
                                        pool.call(
                                            &def.server,
                                            &def.tool,
                                            arguments,
                                            timeout,
                                            &cancel,
                                            authority.limits().max_tool_result_bytes,
                                        )
                                        .await
                                    }
                                    None => ToolResult::failure(
                                        ToolStatus::Error,
                                        "mcp_unavailable",
                                        "MCP is not configured for this run",
                                    ),
                                }
                            } else {
                                let reader =
                                    resources.workspaces.get(&authority.workspace().id).cloned();
                                let writer =
                                    resources.writers.get(&authority.workspace().id).cloned();
                                if let Some(reader) = reader {
                                    let name = tool.ok_or("validated tool missing")?;
                                    let validated =
                                        args.as_ref().map_err(|_| "validated arguments missing")?;
                                    let shape = validated.shape_for(name);
                                    let write_args = validated.clone();
                                    let path = validated.path.clone();
                                    let query = validated.query.clone();
                                    let case_sensitive = validated.case_sensitive.unwrap_or(false);
                                    let maximum = authority.limits().max_tool_result_bytes;
                                    let evidence_id =
                                        name.mints_evidence().then(|| evidence.next_id());
                                    dispatched = true;
                                    if name == ToolName::RunCommand {
                                        // A command spawns a process, so it runs on the async
                                        // path where it can be actively killed on timeout or
                                        // cancellation — unlike a filesystem syscall, which the
                                        // blocking worker below can only wait out. The runner
                                        // exists solely for a workspace granted run_command.
                                        let _permit = permit;
                                        match shape {
                                            Err((code, message)) => ToolResult::failure(
                                                ToolStatus::Denied,
                                                code,
                                                message,
                                            ),
                                            Ok(()) => match resources
                                                .command_runners
                                                .get(&authority.workspace().id)
                                                .cloned()
                                            {
                                                None => ToolResult::failure(
                                                    ToolStatus::Denied,
                                                    "command_not_authorized",
                                                    "This workspace does not grant commands.",
                                                ),
                                                Some(command_runner) => {
                                                    let argv = write_args
                                                        .command
                                                        .clone()
                                                        .unwrap_or_default();
                                                    // The command self-limits to the remaining run
                                                    // budget; cancellation kills it either way. Its
                                                    // captured output is bounded by the same
                                                    // tool-result budget every other tool honors.
                                                    let timeout = deadline
                                                        .saturating_duration_since(Instant::now());
                                                    command_runner
                                                        .execute(
                                                            &argv,
                                                            timeout,
                                                            &cancel,
                                                            authority
                                                                .limits()
                                                                .max_tool_result_bytes,
                                                        )
                                                        .await
                                                }
                                            },
                                        }
                                    } else {
                                        let mut handle = tokio::task::spawn_blocking(move || {
                                            // The actual blocking worker owns capacity until it exits.
                                            let _permit = permit;
                                            if let Err((code, message)) = shape {
                                                return ToolResult::failure(
                                                    ToolStatus::Denied,
                                                    code,
                                                    message,
                                                );
                                            }
                                            if name.is_mutating() {
                                                // A write reaches only the separate writer, which exists
                                                // solely for a workspace the operator granted writes.
                                                return match writer {
                                                    Some(writer) => {
                                                        writer.execute(name, &write_args)
                                                    }
                                                    None => ToolResult::failure(
                                                        ToolStatus::Denied,
                                                        "write_not_authorized",
                                                        "This workspace does not grant writes.",
                                                    ),
                                                };
                                            }
                                            reader.execute_with_query(
                                                name,
                                                &path,
                                                query.as_deref(),
                                                case_sensitive,
                                                maximum,
                                                evidence_id.as_deref(),
                                            )
                                        });
                                        let completed = tokio::select! {
                                            biased;
                                            _=cancel.cancelled()=>None,
                                            _=tokio::time::sleep_until(deadline)=>None,
                                            result=&mut handle=>Some(result),
                                        };
                                        // Cancellation cannot cancel a running filesystem syscall. Retain
                                        // ownership and capacity through its actual completion.
                                        let completed = match completed {
                                            Some(result) => result,
                                            None => {
                                                journal.stop();
                                                match timeout_at(
                                                    journal
                                                        .grace
                                                        .ok_or("missing settlement grace")?,
                                                    &mut handle,
                                                )
                                                .await
                                                {
                                                    Ok(result) => result,
                                                    Err(_) => {
                                                        eprintln!(
                                                            "Run {} has unresolved tool settlement; retaining ownership",
                                                            authority.run_id()
                                                        );
                                                        let result = handle.await;
                                                        journal.late_bookkeeping = true;
                                                        result
                                                    }
                                                }
                                            }
                                        };
                                        completed.unwrap_or_else(|_| {
                                            ToolResult::failure(
                                                ToolStatus::Error,
                                                "tool_worker_failed",
                                                "Tool worker failed",
                                            )
                                        })
                                    }
                                } else {
                                    drop(permit);
                                    ToolResult::failure(
                                        ToolStatus::Error,
                                        "workspace_unavailable",
                                        "Workspace capability is unavailable",
                                    )
                                }
                            }
                        } else {
                            ToolResult::failure(
                                ToolStatus::Error,
                                "tool_unsent",
                                "Run stopped before dispatch",
                            )
                        }
                    };
                    if journal.requested_stop().is_some() {
                        journal.stop();
                    }
                    let observation_seq = journal.seq;
                    let path = args.as_ref().ok().map(|args| args.path.as_str());
                    let mut metadata = json!({"effect_id":effect_id,"call_id":call.id,
                        "control_dispatch":dispatch_control,
                        "tool":call.name,"dispatch":if dispatched{"executed"}else if result.status==ToolStatus::Denied{"denied"}else{"unsent"},
                        "duration_ms":tool_start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                        "classification":result.status,"complete":!result.truncated,"result_bytes":result.encoded()?.len(),
                        "evidence_id":result.evidence_id,"resource":path,"sha256":model::fingerprint(result.body.as_bytes())});
                    if authority.capture() == CaptureMode::Replay {
                        metadata["replay"] = json!({"observation":result});
                    }
                    journal.record("tool_finished", metadata, None).await?;
                    if dispatched
                        && tool == Some(ToolName::ReadFile)
                        && let Err(reason) = evidence.add_observation(
                            &authority,
                            &effect_id,
                            path.ok_or("missing evidence resource")?,
                            &result,
                            observation_seq,
                        )
                    {
                        journal.stop();
                        effect = state
                            .stop(RunPhase::Stopped, reason)
                            .map_err(|e| e.to_string())?;
                        continue 'run;
                    }
                    if let Some((phase, reason)) = journal.requested_stop() {
                        effect = state
                            .stop(phase, reason.into())
                            .map_err(|e| e.to_string())?;
                        continue 'run;
                    }
                    if let Some(next) = state
                        .observe_tool(&call.id, result.encoded()?)
                        .map_err(|e| e.to_string())?
                    {
                        effect = next;
                    }
                    if !compact_state(&mut state, &mut journal, authority.limits())? {
                        journal.stop();
                        effect = state
                            .stop(RunPhase::Stopped, "history_bytes_limit".into())
                            .map_err(|e| e.to_string())?;
                        continue 'run;
                    }
                }
            }
        }
    }
}

/// Compact `state` to fit the history budget, recording any drops in the journal.
/// Returns whether the conversation now fits; the caller stops the run when it
/// does not. Used at the sites that operate on the live state (before a request
/// and after a tool observation); the post-model site works on a clone and counts
/// drops itself, since an unfit clone is discarded.
fn compact_state(
    state: &mut core::RunState,
    journal: &mut RunJournal,
    limits: &crate::config::Limits,
) -> Result<bool, String> {
    let (dropped, fits) = model::compact_until_fits(
        state,
        limits.max_history_bytes,
        limits.compaction.enabled,
        limits.compaction.floor,
    )?;
    journal.compactions += dropped as u64;
    Ok(fits)
}

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub phase: RunPhase,
    pub acceptance: AcceptanceStatus,
    pub candidate: Option<String>,
    pub reason: String,
}

/// Step-eight proof path. This takes only the scripted client variant and does
/// not claim a durable result; production execution will use the journal.
pub async fn run_scripted(
    instructions: String,
    prompt: String,
    client: &ModelClient,
    options: &ModelOptions,
) -> Result<RunOutcome, String> {
    if !matches!(client, ModelClient::Scripted(_)) {
        return Err("proof path only permits scripted clients".into());
    }
    let (mut state, initial) = core::initiate(instructions, prompt).map_err(|e| e.to_string())?;
    if initial != Effect::Model {
        return Err("initial effect is not model".into());
    }
    let prepared = model::prepare(state.messages(), options)?;
    state.model_started().map_err(|e| e.to_string())?;
    let observation = client
        .send(&prepared)
        .await
        .map(|outcome| outcome.reply)
        .unwrap_or_else(ModelReply::Failure);
    let effect = state
        .observe_model(observation)
        .map_err(|e| e.to_string())?;
    let outcome = match effect {
        Effect::Candidate(text) => RunOutcome {
            phase: RunPhase::Completed,
            acceptance: AcceptanceStatus::Unchecked,
            candidate: Some(text),
            reason: "answer_candidate".into(),
        },
        Effect::Stop { phase, reason } => RunOutcome {
            phase,
            acceptance: AcceptanceStatus::Unchecked,
            candidate: None,
            reason,
        },
        _ => return Err("unexpected effect in one-turn proof".into()),
    };
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ScriptStep;
    use std::time::Duration;

    fn options() -> ModelOptions {
        ModelOptions {
            origin: "scripted".into(),
            served_model: "scripted".into(),
            temperature: 0.0,
            max_output_tokens: 64,
            max_request_bytes: 131072,
            max_response_bytes: 1048576,
            stream: false,
            cache_prompt: true,
            tools: Vec::new(),
            constraint: None,
        }
    }

    #[tokio::test]
    async fn one_request_gives_an_unchecked_candidate() {
        let client = ModelClient::scripted([ModelReply::Answer("Kinesin".into()).into()]);
        let outcome = run_scripted("System".into(), "User".into(), &client, &options())
            .await
            .unwrap();
        assert_eq!(outcome.phase, RunPhase::Completed);
        assert_eq!(outcome.acceptance, AcceptanceStatus::Unchecked);
        assert_eq!(outcome.candidate.as_deref(), Some("Kinesin"));
        assert_eq!(client.captured_requests().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn model_error_and_exhaustion_do_not_retry() {
        for steps in [vec![ModelReply::Failure("transport".into()).into()], vec![]] {
            let client = ModelClient::scripted(steps);
            let outcome = run_scripted("System".into(), "User".into(), &client, &options())
                .await
                .unwrap();
            assert_eq!(outcome.phase, RunPhase::Failed);
            assert!(outcome.candidate.is_none());
            assert_eq!(client.captured_requests().unwrap().len(), 1);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn waiting_model_does_not_block_an_independent_task() {
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(1);
        let client = ModelClient::scripted([ScriptStep {
            delay: Duration::from_millis(100),
            reply: ModelReply::Answer("late".into()),
            usage: None,
        }]);
        let owner = tokio::spawn(async move {
            run_scripted("System".into(), "User".into(), &client, &options()).await
        });
        let progress = tokio::spawn(async move {
            progress_tx.send("progress").await.unwrap();
        });
        assert_eq!(progress_rx.recv().await, Some("progress"));
        progress.await.unwrap();
        assert_eq!(owner.await.unwrap().unwrap().phase, RunPhase::Completed);
    }
}
