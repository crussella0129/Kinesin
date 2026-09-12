//! Bounded controller ownership from submission transfer through runner settlement.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{Notify, mpsc, oneshot};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::config::ConcurrencyConfig;
use crate::model::ModelClient;
use crate::policy::RunAuthority;
use crate::runner::{self, RunResources};
use crate::storage::{Command, Response, RunRecord, StorageClient};

/// Resources and clients are prepared once by trusted startup, then cloned.
#[derive(Clone)]
pub struct Job {
    pub authority: RunAuthority,
    pub client: ModelClient,
    pub resources: RunResources,
    pub display: Option<crate::model::TextObserver>,
}

impl Job {
    /// Attach operator declarations only. Connecting and discovery belong to
    /// the admitted active runner, after queue/idempotency arbitration. A Job
    /// cannot own a pre-admission MCP process.
    pub fn discover_mcp(mut self, config: &crate::config::Config) -> Self {
        self.resources = self
            .resources
            .with_mcp_servers(config.mcp_servers().to_vec().into());
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControllerStats {
    /// Includes admission, execution, finalization, and completed unjoined tasks.
    pub active_runs: usize,
    /// Includes queued payloads still in the bounded controller inbox.
    pub queued_runs: usize,
    pub queued_input_bytes: usize,
    pub peak_active_runs: usize,
    pub peak_queued_runs: usize,
    pub peak_queued_input_bytes: usize,
    pub rejected_runs: usize,
    pub completed_runners: usize,
    pub runner_errors: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotKind {
    Active,
    Queued,
}
struct Slot {
    owner: String,
    kind: SlotKind,
    bytes: usize,
    cancel: CancellationToken,
}
#[derive(Default)]
struct OwnerCounts {
    active: usize,
    queued: usize,
}
struct State {
    accepting: bool,
    limits: ConcurrencyConfig,
    service_mode: bool,
    slots: BTreeMap<String, Slot>,
    owners: BTreeMap<String, OwnerCounts>,
    stats: ControllerStats,
    changed: Arc<Notify>,
}

impl State {
    fn owner_ready(&self, owner: &str) -> bool {
        !self.service_mode
            || self.owners.get(owner).map_or(0, |counts| counts.active)
                < self.limits.per_owner_active_runs
    }
    fn promote(&mut self, run: &str) -> bool {
        let slot = self.slots.get(run).expect("owned queue reservation");
        if slot.kind == SlotKind::Active {
            return true;
        }
        if self.stats.active_runs >= self.limits.max_active_runs || !self.owner_ready(&slot.owner) {
            return false;
        }
        let slot = self.slots.get_mut(run).expect("owned queue reservation");
        slot.kind = SlotKind::Active;
        self.stats.queued_runs -= 1;
        self.stats.queued_input_bytes -= slot.bytes;
        self.stats.active_runs += 1;
        self.stats.peak_active_runs = self.stats.peak_active_runs.max(self.stats.active_runs);
        let counts = self
            .owners
            .get_mut(&slot.owner)
            .expect("owned owner counts");
        counts.queued -= 1;
        counts.active += 1;
        self.changed.notify_one();
        true
    }
    fn stop_admission(&mut self) {
        self.accepting = false;
        for slot in self.slots.values() {
            slot.cancel.cancel();
        }
        self.changed.notify_one();
    }
}

/// This reservation remains in a JoinSet output until the controller drains it.
struct Ticket {
    run: String,
    shared: Arc<Mutex<State>>,
}
impl Drop for Ticket {
    fn drop(&mut self) {
        let mut state = self.shared.lock().expect("controller state mutex");
        let Some(slot) = state.slots.remove(&self.run) else {
            return;
        };
        match slot.kind {
            SlotKind::Active => state.stats.active_runs -= 1,
            SlotKind::Queued => {
                state.stats.queued_runs -= 1;
                state.stats.queued_input_bytes -= slot.bytes;
            }
        }
        if let Some(counts) = state.owners.get_mut(&slot.owner) {
            match slot.kind {
                SlotKind::Active => counts.active -= 1,
                SlotKind::Queued => counts.queued -= 1,
            }
            if counts.active == 0 && counts.queued == 0 {
                state.owners.remove(&slot.owner);
            }
        }
        state.changed.notify_one();
    }
}

type RunReply = Result<RunRecord, String>;
struct Envelope {
    job: Job,
    key: Option<String>,
    accepted: Instant,
    cancel: CancellationToken,
    ticket: Ticket,
    admitted: Option<oneshot::Sender<RunReply>>,
    finished: oneshot::Sender<RunReply>,
}

pub struct PendingRun {
    run_id: String,
    admitted: Option<oneshot::Receiver<RunReply>>,
    finished: oneshot::Receiver<RunReply>,
}
impl PendingRun {
    /// Proposed ID. An idempotency race may return a different existing ID at admission.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub async fn admitted(&mut self) -> RunReply {
        let response = self
            .admitted
            .take()
            .ok_or("admission_already_observed")?
            .await
            .map_err(|_| "controller_admission_dropped")?;
        if let Ok(record) = &response {
            self.run_id = record.run_id.clone();
        }
        response
    }
    pub async fn finished(self) -> RunReply {
        self.finished
            .await
            .map_err(|_| "controller_runner_dropped")?
    }
}

#[derive(Clone)]
pub struct ControllerHandle {
    sender: mpsc::Sender<Envelope>,
    shared: Arc<Mutex<State>>,
    shutdown: CancellationToken,
    store: StorageClient,
    changed: Arc<Notify>,
}
pub struct Controller {
    task: JoinHandle<ControllerStats>,
}

impl Controller {
    /// Call inside the single application Tokio runtime. Storage has already started.
    pub fn start(
        limits: ConcurrencyConfig,
        store: StorageClient,
        service_mode: bool,
    ) -> Result<(ControllerHandle, Self), String> {
        let inbox_size = limits
            .max_active_runs
            .checked_add(limits.max_queued_runs)
            .ok_or("invalid_controller_limits")?;
        if limits.max_active_runs == 0
            || inbox_size > tokio::sync::Semaphore::MAX_PERMITS
            || limits.max_queued_input_bytes == 0
            || limits.journal_admission_timeout_s == 0
            || (service_mode && limits.per_owner_active_runs == 0)
        {
            return Err("invalid_controller_limits".into());
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| "controller_requires_tokio_runtime")?;
        let changed = Arc::new(Notify::new());
        let shared = Arc::new(Mutex::new(State {
            accepting: true,
            limits,
            service_mode,
            slots: BTreeMap::new(),
            owners: BTreeMap::new(),
            stats: ControllerStats::default(),
            changed: changed.clone(),
        }));
        let shutdown = CancellationToken::new();
        let (sender, inbox) = mpsc::channel(inbox_size);
        let handle = ControllerHandle {
            sender,
            shared: shared.clone(),
            shutdown: shutdown.clone(),
            store: store.clone(),
            changed,
        };
        let task = runtime.spawn(control(inbox, shared, shutdown, store));
        Ok((handle, Self { task }))
    }

    /// Stop admission with handle.shutdown(), then await all owned work here.
    /// Dropping a JoinHandle does not abort its controller or runner tasks.
    pub async fn join(self) -> Result<ControllerStats, String> {
        self.task.await.map_err(|_| "controller_panicked".into())
    }
}

impl ControllerHandle {
    /// Authorize the current owner and requested resources before this lookup.
    /// Call before new-run capacity checks. A hit is the original historical run;
    /// the caller must not dispatch or silently rejudge it.
    pub async fn lookup_retry(
        &self,
        owner: &str,
        key: &str,
        fingerprint: &str,
    ) -> Result<Option<RunRecord>, String> {
        let wait = self
            .shared
            .lock()
            .expect("controller state mutex")
            .limits
            .journal_admission_timeout_s;
        match self
            .store
            .execute(
                Command::Lookup {
                    owner_id: owner.into(),
                    key: key.into(),
                    submission_sha256: fingerprint.into(),
                },
                Instant::now() + Duration::from_secs(wait),
            )
            .await
            .map_err(|error| error.to_string())?
        {
            Response::Run(record) => Ok(record.map(|record| *record)),
            _ => Err("unexpected_idempotency_response".into()),
        }
    }

    /// No waiting sender or runner is spawned before count/byte reservation.
    /// An identical authorized retry should use lookup_retry before this method.
    pub fn try_submit(&self, job: Job, key: Option<String>) -> Result<PendingRun, String> {
        if key.as_ref().is_some_and(|key| {
            key.is_empty()
                || key.len() > 128
                || !key.is_ascii()
                || key.bytes().any(|b| b.is_ascii_control())
        }) {
            return Err("invalid_idempotency_key".into());
        }
        let bytes = retained_input_bytes(&job.authority)?
            .checked_add(key.as_ref().map_or(0, String::len))
            .ok_or("input_size_overflow")?;
        let run_id = job.authority.run_id().to_owned();
        let owner = job.authority.owner().to_owned();
        let cancel = CancellationToken::new();
        {
            let mut state = self.shared.lock().expect("controller state mutex");
            if !state.accepting {
                return Err("controller_closed".into());
            }
            if state.slots.contains_key(&run_id) {
                return Err("duplicate_live_run_id".into());
            }
            let eligible_queue = state
                .slots
                .values()
                .any(|slot| slot.kind == SlotKind::Queued && state.owner_ready(&slot.owner));
            let kind = if state.stats.active_runs < state.limits.max_active_runs
                && state.owner_ready(&owner)
                && !eligible_queue
            {
                SlotKind::Active
            } else {
                let owner_queue = state.owners.get(&owner).map_or(0, |counts| counts.queued);
                if state.service_mode && owner_queue >= state.limits.per_owner_queued_runs {
                    state.stats.rejected_runs += 1;
                    return Err("controller_owner_overloaded".into());
                }
                if state.stats.queued_runs >= state.limits.max_queued_runs
                    || bytes
                        > state
                            .limits
                            .max_queued_input_bytes
                            .saturating_sub(state.stats.queued_input_bytes)
                {
                    state.stats.rejected_runs += 1;
                    return Err("controller_overloaded".into());
                }
                SlotKind::Queued
            };
            match kind {
                SlotKind::Active => {
                    state.stats.active_runs += 1;
                    state.owners.entry(owner.clone()).or_default().active += 1;
                }
                SlotKind::Queued => {
                    state.stats.queued_runs += 1;
                    state.stats.queued_input_bytes += bytes;
                    state.owners.entry(owner.clone()).or_default().queued += 1;
                }
            }
            state.stats.peak_active_runs =
                state.stats.peak_active_runs.max(state.stats.active_runs);
            state.stats.peak_queued_runs =
                state.stats.peak_queued_runs.max(state.stats.queued_runs);
            state.stats.peak_queued_input_bytes = state
                .stats
                .peak_queued_input_bytes
                .max(state.stats.queued_input_bytes);
            state.slots.insert(
                run_id.clone(),
                Slot {
                    owner,
                    kind,
                    bytes,
                    cancel: cancel.clone(),
                },
            );
        }
        let ticket = Ticket {
            run: run_id.clone(),
            shared: self.shared.clone(),
        };
        let (admitted, admission_rx) = oneshot::channel();
        let (finished, finish_rx) = oneshot::channel();
        let envelope = Envelope {
            job,
            key,
            accepted: Instant::now(),
            cancel,
            ticket,
            admitted: Some(admitted),
            finished,
        };
        // A closed/full inbox drops the envelope and returns both reservations.
        self.sender
            .try_send(envelope)
            .map_err(|_| "controller_closed")?;
        Ok(PendingRun {
            run_id,
            admitted: Some(admission_rx),
            finished: finish_rx,
        })
    }

    /// Caller supplies its authenticated owner; another owner's ID does not match.
    pub fn cancel(&self, owner: &str, run_id: &str) -> bool {
        let state = self.shared.lock().expect("controller state mutex");
        if let Some(slot) = state.slots.get(run_id).filter(|slot| slot.owner == owner) {
            slot.cancel.cancel();
            true
        } else {
            false
        }
    }
    pub fn stats(&self) -> ControllerStats {
        self.shared
            .lock()
            .expect("controller state mutex")
            .stats
            .clone()
    }
    /// One bounded local batch producer may wait for released capacity. HTTP
    /// admission remains nonblocking through try_submit; it does not spawn waiters.
    pub async fn capacity_changed(&self) {
        self.changed.notified().await;
    }
    pub fn shutdown(&self) {
        self.shared
            .lock()
            .expect("controller state mutex")
            .stop_admission();
        self.shutdown.cancel();
    }
}

fn retained_input_bytes(authority: &RunAuthority) -> Result<usize, String> {
    struct Counter(usize);
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("input size overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, authority).map_err(|_| "input_size_overflow")?;
    Ok(counter.0)
}

fn spawn_runner(envelope: Envelope, runners: &mut JoinSet<(Ticket, bool)>, store: &StorageClient) {
    let store = store.clone();
    runners.spawn(async move {
        let Envelope {
            job,
            accepted,
            cancel,
            ticket,
            finished,
            ..
        } = envelope;
        let result = runner::run_admitted_with_text(
            job.authority,
            job.client,
            store,
            job.resources,
            cancel,
            accepted,
            job.display,
        )
        .await;
        let failed = result.is_err();
        let _ = finished.send(result);
        (ticket, failed)
    });
}

fn dispatch_ready(
    queue: &mut VecDeque<Envelope>,
    runners: &mut JoinSet<(Ticket, bool)>,
    shared: &Arc<Mutex<State>>,
    store: &StorageClient,
    last_owner: &mut Option<String>,
) {
    loop {
        let selected = {
            let mut state = shared.lock().expect("controller state mutex");
            if state.stats.active_runs >= state.limits.max_active_runs {
                return;
            }
            let index = if !state.service_mode {
                if queue.is_empty() {
                    return;
                }
                0
            } else {
                let owners: BTreeSet<_> = queue
                    .iter()
                    .map(|entry| entry.job.authority.owner())
                    .filter(|owner| state.owner_ready(owner))
                    .collect();
                let selected_owner = owners
                    .iter()
                    .find(|owner| {
                        last_owner
                            .as_ref()
                            .is_none_or(|last| **owner > last.as_str())
                    })
                    .or_else(|| owners.first());
                let Some(owner) = selected_owner else {
                    return;
                };
                queue
                    .iter()
                    .position(|entry| entry.job.authority.owner() == *owner)
                    .expect("queued owner")
            };
            let entry = &queue[index];
            if !state.promote(&entry.ticket.run) {
                return;
            }
            *last_owner = Some(entry.job.authority.owner().to_owned());
            index
        };
        spawn_runner(
            queue.remove(selected).expect("selected queue entry"),
            runners,
            store,
        );
    }
}

async fn control(
    mut inbox: mpsc::Receiver<Envelope>,
    shared: Arc<Mutex<State>>,
    shutdown: CancellationToken,
    store: StorageClient,
) -> ControllerStats {
    let mut runners = JoinSet::new();
    let mut queue = VecDeque::new();
    let mut last_owner = None;
    let mut closing = false;
    let mut inbox_open = true;
    loop {
        dispatch_ready(&mut queue, &mut runners, &shared, &store, &mut last_owner);
        if !inbox_open && queue.is_empty() && runners.is_empty() {
            break;
        }
        tokio::select! {
            biased;
            completed = runners.join_next(), if !runners.is_empty() => {
                match completed {
                    Some(Ok((ticket, failed))) => {
                        {
                            let mut state = shared.lock().expect("controller state mutex");
                            state.stats.completed_runners += 1;
                            state.stats.runner_errors += usize::from(failed);
                        }
                        drop(ticket);
                    }
                    Some(Err(_)) => shared.lock().expect("controller state mutex").stats.runner_errors += 1,
                    None => {}
                }
            }
            _ = shutdown.cancelled(), if !closing => {
                closing = true;
                shared.lock().expect("controller state mutex").stop_admission();
                inbox.close();
            }
            entry = inbox.recv(), if inbox_open => {
                let Some(mut entry) = entry else {
                    inbox_open = false;
                    closing = true;
                    shared.lock().expect("controller state mutex").stop_admission();
                    continue;
                };
                // The actor, rather than a request's future, owns this commit.
                // It must settle even when a client drops both reply receivers.
                match runner::admit_once(&entry.job.authority, &store, entry.key.take()).await {
                    Ok((record, created)) => {
                        let terminal = matches!(record.phase.as_str(), "completed" | "stopped" | "failed" | "cancelled" | "interrupted");
                        let admitted = entry.admitted.take().expect("admission acknowledgement");
                        let _ = admitted.send(Ok(record.clone()));
                        if !created {
                            let _ = entry.finished.send(if terminal { Ok(record) } else { Err("existing_run_pending".into()) });
                            continue;
                        }
                        let active = shared.lock().expect("controller state mutex").slots.get(&entry.ticket.run)
                            .is_some_and(|slot| slot.kind == SlotKind::Active);
                        if active {
                            last_owner = Some(entry.job.authority.owner().to_owned());
                            spawn_runner(entry, &mut runners, &store);
                        } else { queue.push_back(entry); }
                    }
                    Err(error) => {
                        let _ = entry.admitted.take().expect("admission acknowledgement").send(Err(error.clone()));
                        let _ = entry.finished.send(Err(error));
                    }
                }
            }
        }
    }
    shared.lock().expect("controller state mutex").stats.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        Config,
        test_support::{BASE, Fixture},
    };
    use crate::core::ModelReply;
    use crate::model::ScriptStep;
    use crate::policy::Submission;
    use crate::storage::{QueueLimits, Storage};
    use tokio::time::timeout;

    /// Detects a hung wait; it is not a latency assertion. A cancelled run can
    /// legitimately wait through the journal admission timeout and then the
    /// settlement grace, so this must exceed their sum rather than equal one of
    /// them. Latency belongs to the separate benchmarks.
    const WATCHDOG: Duration = Duration::from_secs(30);

    fn test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    }

    /// Wait for a scripted run to actually record its model request. The fake
    /// captures the prepared bytes before its configured delay, so this observes
    /// a started request instead of inferring one from another run's finish.
    async fn started_model_request(client: &ModelClient, label: &str) {
        timeout(WATCHDOG, async {
            loop {
                if !client.captured_requests().unwrap().is_empty() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("{label} never started its model request"));
    }

    fn config(fixture: &Fixture) -> Config {
        fixture
            .parse(&BASE.replace("tools = [\"read_file\"]", "tools = []"))
            .unwrap()
    }
    fn job(
        config: &Config,
        owner: Option<&str>,
        label: &str,
        delay_ms: u64,
        resources: &RunResources,
    ) -> Job {
        let submission = Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            continues: None,
            prompt: label.into(),
            limits: None,
            capture: None,
        };
        let authority = match owner {
            Some(owner) => config.authorize(owner, submission),
            None => config.authorize_local(submission),
        }
        .unwrap();
        Job {
            authority,
            client: ModelClient::scripted([ScriptStep {
                delay: Duration::from_millis(delay_ms),
                reply: ModelReply::Answer(label.into()),
                usage: None,
            }]),
            resources: resources.clone(),
            display: None,
        }
    }

    #[test]
    fn active_queue_and_bytes_are_reserved_before_any_spawn_and_cancel_is_scoped() {
        let fixture = Fixture::new();
        let config = config(&fixture);
        let storage =
            Storage::start(config.storage().path.clone(), QueueLimits::default()).unwrap();
        test_runtime().block_on(async {
            let limits = ConcurrencyConfig {
                max_active_runs: 1,
                max_queued_runs: 1,
                ..Default::default()
            };
            let resources = RunResources::single(1, limits.clone());
            let (handle, controller) = Controller::start(limits, storage.client(), false).unwrap();
            let mut first = handle
                .try_submit(job(&config, None, "first", 30_000, &resources), None)
                .unwrap();
            let mut second = handle
                .try_submit(job(&config, None, "second", 0, &resources), None)
                .unwrap();
            assert_eq!(
                handle
                    .try_submit(job(&config, None, "overflow", 0, &resources), None)
                    .err()
                    .unwrap(),
                "controller_overloaded"
            );
            let stats = handle.stats();
            assert_eq!((stats.active_runs, stats.queued_runs), (1, 1));
            assert!(stats.queued_input_bytes > 0 && stats.queued_input_bytes <= 1_048_576);
            first.admitted().await.unwrap();
            second.admitted().await.unwrap();
            assert!(!handle.cancel("another-owner", first.run_id()));
            assert!(handle.cancel("local", first.run_id()));
            assert_eq!(
                timeout(WATCHDOG, first.finished())
                    .await
                    .unwrap()
                    .unwrap()
                    .phase,
                "cancelled"
            );
            let second = timeout(WATCHDOG, second.finished()).await.unwrap().unwrap();
            assert_eq!(second.result.unwrap()["candidate"], "second");
            handle.shutdown();
            let final_stats = controller.join().await.unwrap();
            assert_eq!(
                (
                    final_stats.active_runs,
                    final_stats.queued_runs,
                    final_stats.queued_input_bytes
                ),
                (0, 0, 0)
            );
            assert_eq!(
                (final_stats.peak_active_runs, final_stats.peak_queued_runs),
                (1, 1)
            );
            storage.shutdown().await.unwrap();
        });
    }

    #[test]
    fn two_independent_runs_overlap_while_preserving_reply_and_cancellation_ownership() {
        let fixture = Fixture::new();
        let config = fixture
            .parse(
                &BASE
                    .replace("tools = [\"read_file\"]", "tools = []")
                    .replace("verified_slots = 1", "verified_slots = 2"),
            )
            .unwrap();
        let storage =
            Storage::start(config.storage().path.clone(), QueueLimits::default()).unwrap();
        test_runtime().block_on(async {
            let limits = ConcurrencyConfig {
                max_active_runs: 2,
                max_queued_runs: 0,
                max_inflight_model_requests: 2,
                ..Default::default()
            };
            let resources = RunResources::single(2, limits.clone());
            let (handle, controller) = Controller::start(limits, storage.client(), false).unwrap();
            let slow_job = job(&config, None, "slow-isolated", 30_000, &resources);
            let slow_client = slow_job.client.clone();
            let fast_job = job(&config, None, "fast-isolated", 100, &resources);
            let fast_client = fast_job.client.clone();
            let mut slow = handle.try_submit(slow_job, None).unwrap();
            let mut fast = handle.try_submit(fast_job, None).unwrap();
            slow.admitted().await.unwrap();
            fast.admitted().await.unwrap();
            // Overlap is observed, not inferred: the slow run holds an in-flight
            // request of its own before the fast run is allowed to finish. Each
            // wait carries its own watchdog so one slow start cannot consume the
            // budget of the other.
            started_model_request(&slow_client, "slow-isolated").await;
            started_model_request(&fast_client, "fast-isolated").await;
            let fast = timeout(WATCHDOG, fast.finished()).await.unwrap().unwrap();
            assert_eq!(fast.result.unwrap()["candidate"], "fast-isolated");
            assert_eq!(handle.stats().peak_active_runs, 2);
            let slow_request =
                String::from_utf8(slow_client.captured_requests().unwrap()[0].clone()).unwrap();
            let fast_request =
                String::from_utf8(fast_client.captured_requests().unwrap()[0].clone()).unwrap();
            assert!(
                slow_request.contains("slow-isolated") && !slow_request.contains("fast-isolated")
            );
            assert!(
                fast_request.contains("fast-isolated") && !fast_request.contains("slow-isolated")
            );
            handle.cancel("local", slow.run_id());
            assert_eq!(slow.finished().await.unwrap().phase, "cancelled");
            handle.shutdown();
            assert_eq!(controller.join().await.unwrap().completed_runners, 2);
            storage.shutdown().await.unwrap();
        });
    }

    #[test]
    fn writer_failure_during_two_active_effects_rejects_more_work_and_owned_shutdown_joins() {
        let fixture = Fixture::new();
        let config = fixture
            .parse(
                &BASE
                    .replace("tools = [\"read_file\"]", "tools = []")
                    .replace("verified_slots = 1", "verified_slots = 2"),
            )
            .unwrap();
        let path = config.storage().path.clone();
        let storage = Storage::start(path.clone(), QueueLimits::default()).unwrap();
        test_runtime().block_on(async {
            let limits = ConcurrencyConfig {
                max_active_runs: 2,
                max_queued_runs: 1,
                max_inflight_model_requests: 2,
                ..Default::default()
            };
            let resources = RunResources::single(2, limits.clone());
            let (handle, controller) = Controller::start(limits, storage.client(), false).unwrap();
            let first_job = job(&config, None, "first-active", 30_000, &resources);
            let first_client = first_job.client.clone();
            let second_job = job(&config, None, "second-active", 30_000, &resources);
            let second_client = second_job.client.clone();
            let mut first = handle.try_submit(first_job, None).unwrap();
            let mut second = handle.try_submit(second_job, None).unwrap();
            let first_id = first.admitted().await.unwrap().run_id;
            let second_id = second.admitted().await.unwrap().run_id;
            timeout(WATCHDOG, async {
                while first_client.captured_requests().unwrap().len() != 1
                    || second_client.captured_requests().unwrap().len() != 1 {
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
            }).await.unwrap();
            assert_eq!(handle.stats().active_runs, 2);
            assert_eq!(resources.models.available_permits(), 0);
            let failure_path = path.clone();
            let elapsed_ms = tokio::task::spawn_blocking(move || {
                let connection = rusqlite::Connection::open(failure_path).unwrap();
                connection.busy_timeout(Duration::from_secs(2)).unwrap();
                connection.execute_batch("CREATE TRIGGER reject_events BEFORE INSERT ON events BEGIN SELECT RAISE(ABORT,'test writer failure'); END;").unwrap();
                let elapsed: i64 = connection.query_row("SELECT max(elapsed_ms) FROM events", [], |row|row.get(0)).unwrap();
                u64::try_from(elapsed).unwrap()
            }).await.unwrap();
            let error = storage.client().execute(Command::Append {
                owner_id: "local".into(),
                run_id: first_id.clone(),
                event: crate::storage::Event { seq:3, kind:"cancel_requested".into(), elapsed_ms, data:serde_json::json!({}) },
                phase: None,
            }, Instant::now() + Duration::from_secs(2)).await.unwrap_err();
            assert_eq!(error.code, "storage_database");
            assert!(!storage.client().is_accepting());
            // Even before the caller initiates shutdown, durable admission
            // rejects further work and cannot dispatch a third model effect.
            let third_job = job(&config, None, "must-not-start", 0, &resources);
            let third_client = third_job.client.clone();
            let mut third = handle.try_submit(third_job, None).unwrap();
            assert!(timeout(WATCHDOG, third.admitted()).await.unwrap().unwrap_err().contains("storage_closed"));
            assert!(third.finished().await.unwrap_err().contains("storage_closed"));
            assert!(third_client.captured_requests().unwrap().is_empty());
            // This is the documented caller-owned shutdown sequence after a
            // journal error, not an automatic storage-health watcher.
            handle.shutdown();
            for pending in [first, second] {
                assert!(timeout(WATCHDOG, pending.finished()).await.unwrap().unwrap_err().contains("storage_closed"));
            }
            let stats = timeout(WATCHDOG, controller.join()).await.unwrap().unwrap();
            assert_eq!((stats.completed_runners, stats.runner_errors), (2, 2));
            assert_eq!((stats.active_runs, stats.queued_runs, stats.queued_input_bytes), (0, 0, 0));
            assert_eq!(resources.models.available_permits(), 2);
            assert_eq!(storage.client().outstanding(), (0, 0));
            assert_eq!(first_client.captured_requests().unwrap().len(), 1);
            assert_eq!(second_client.captured_requests().unwrap().len(), 1);
            storage.shutdown().await.unwrap();
            tokio::task::spawn_blocking(move || {
                let connection = rusqlite::Connection::open(&path).unwrap();
                let settled: i64 = connection.query_row("SELECT count(*) FROM runs WHERE phase != 'running' OR acceptance_status='passed' OR acceptance_json IS NOT NULL", [], |row| row.get(0)).unwrap();
                assert_eq!(settled, 0, "failed writer cannot claim persisted terminal success");
                let events: i64 = connection.query_row("SELECT count(*) FROM events WHERE kind IN ('cancel_requested','run_finished')", [], |row| row.get(0)).unwrap();
                assert_eq!(events, 0, "failed event transaction must roll back");
                connection.execute_batch("DROP TRIGGER reject_events").unwrap();
                drop(connection);
                let mut restarted = crate::storage::Store::open(&path).unwrap();
                for run_id in [first_id, second_id] {
                    let Response::Run(Some(run)) = restarted.execute(Command::Get {owner_id:"local".into(),run_id}).unwrap() else {panic!("recovered run exists")};
                    assert_eq!(run.phase, "interrupted");
                    assert!(!run.task_accepted());
                }
            }).await.unwrap();
        });
    }

    #[test]
    fn byte_overload_and_zero_queue_reject_immediately_without_retained_inputs() {
        for (queue, bytes) in [(1, 1), (0, 1_048_576)] {
            let fixture = Fixture::new();
            let config = config(&fixture);
            let storage =
                Storage::start(config.storage().path.clone(), QueueLimits::default()).unwrap();
            test_runtime().block_on(async {
                let limits = ConcurrencyConfig {
                    max_active_runs: 1,
                    max_queued_runs: queue,
                    max_queued_input_bytes: bytes,
                    ..Default::default()
                };
                let resources = RunResources::single(1, limits.clone());
                let (handle, controller) =
                    Controller::start(limits, storage.client(), false).unwrap();
                let pending = handle
                    .try_submit(job(&config, None, "active", 30_000, &resources), None)
                    .unwrap();
                assert!(
                    handle
                        .try_submit(job(&config, None, "excess", 0, &resources), None)
                        .is_err()
                );
                assert_eq!(handle.stats().queued_input_bytes, 0);
                drop(pending);
                handle.shutdown();
                assert_eq!(controller.join().await.unwrap().active_runs, 0);
                storage.shutdown().await.unwrap();
            });
        }
    }

    #[test]
    fn dropped_admission_receiver_does_not_strand_a_committed_run() {
        let fixture = Fixture::new();
        let config = config(&fixture);
        let storage =
            Storage::start(config.storage().path.clone(), QueueLimits::default()).unwrap();
        test_runtime().block_on(async {
            let resources = RunResources::single(1, ConcurrencyConfig::default());
            let (handle, controller) =
                Controller::start(ConcurrencyConfig::default(), storage.client(), false).unwrap();
            let pending = handle
                .try_submit(
                    job(&config, None, "survives disconnect", 0, &resources),
                    None,
                )
                .unwrap();
            let run_id = pending.run_id().to_owned();
            drop(pending);
            let record = timeout(WATCHDOG, async {
                loop {
                    let response = storage
                        .client()
                        .execute(
                            Command::Get {
                                owner_id: "local".into(),
                                run_id: run_id.clone(),
                            },
                            Instant::now() + Duration::from_secs(1),
                        )
                        .await
                        .unwrap();
                    if let Response::Run(Some(record)) = response
                        && record.phase == "completed"
                    {
                        break record;
                    }
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
            })
            .await
            .unwrap();
            assert_eq!(
                record.result.as_ref().unwrap()["candidate"],
                "survives disconnect"
            );
            handle.shutdown();
            controller.join().await.unwrap();
            storage.shutdown().await.unwrap();
        });
    }

    #[test]
    fn retries_lookup_before_capacity_and_transaction_races_never_redispatch() {
        let fixture = Fixture::new();
        let config = config(&fixture);
        let storage =
            Storage::start(config.storage().path.clone(), QueueLimits::default()).unwrap();
        test_runtime().block_on(async {
            let limits = ConcurrencyConfig {
                max_active_runs: 1,
                max_queued_runs: 1,
                ..Default::default()
            };
            let resources = RunResources::single(1, limits.clone());
            let (handle, controller) = Controller::start(limits, storage.client(), false).unwrap();
            let first_job = job(&config, None, "same", 30_000, &resources);
            let fingerprint = first_job.authority.submission_sha256().to_owned();
            let mut first = handle.try_submit(first_job, Some("key".into())).unwrap();
            let original = first.admitted().await.unwrap();
            assert_eq!(
                handle
                    .lookup_retry("local", "key", &fingerprint)
                    .await
                    .unwrap()
                    .unwrap()
                    .run_id,
                original.run_id
            );
            let retry_job = job(&config, None, "same", 0, &resources);
            let retry_client = retry_job.client.clone();
            let mut retry = handle.try_submit(retry_job, Some("key".into())).unwrap();
            assert_eq!(retry.admitted().await.unwrap().run_id, original.run_id);
            assert_eq!(retry.finished().await.unwrap_err(), "existing_run_pending");
            assert!(retry_client.captured_requests().unwrap().is_empty());
            handle.cancel("local", first.run_id());
            first.finished().await.unwrap();
            handle.shutdown();
            assert_eq!(controller.join().await.unwrap().completed_runners, 1);
            storage.shutdown().await.unwrap();
        });
    }

    #[test]
    fn service_queue_intersects_owner_limits_and_dispatches_ready_owners_in_turn() {
        let fixture = Fixture::new();
        let owners = r#"
[[owners]]
id = "alice"
workspaces = ["practice"]
models = ["local"]
allow_freeform = true
[[owners]]
id = "bob"
workspaces = ["practice"]
models = ["local"]
allow_freeform = true
"#;
        let config = fixture
            .parse(&format!(
                "{}{owners}",
                BASE.replace("tools = [\"read_file\"]", "tools = []")
            ))
            .unwrap();
        let storage =
            Storage::start(config.storage().path.clone(), QueueLimits::default()).unwrap();
        test_runtime().block_on(async {
            let limits = ConcurrencyConfig {
                max_active_runs: 1,
                max_queued_runs: 2,
                per_owner_active_runs: 1,
                per_owner_queued_runs: 1,
                ..Default::default()
            };
            let resources = RunResources::single(1, limits.clone());
            let (handle, controller) = Controller::start(limits, storage.client(), true).unwrap();
            let mut first = handle
                .try_submit(
                    job(&config, Some("alice"), "alice active", 30_000, &resources),
                    None,
                )
                .unwrap();
            let mut next_alice = handle
                .try_submit(
                    job(&config, Some("alice"), "alice queued", 30_000, &resources),
                    None,
                )
                .unwrap();
            assert!(
                handle
                    .try_submit(
                        job(&config, Some("alice"), "alice excess", 0, &resources),
                        None
                    )
                    .is_err()
            );
            let mut bob = handle
                .try_submit(job(&config, Some("bob"), "bob ready", 0, &resources), None)
                .unwrap();
            first.admitted().await.unwrap();
            next_alice.admitted().await.unwrap();
            bob.admitted().await.unwrap();
            assert!(!handle.cancel("bob", first.run_id()));
            handle.cancel("alice", first.run_id());
            first.finished().await.unwrap();
            assert_eq!(
                timeout(WATCHDOG, bob.finished())
                    .await
                    .unwrap()
                    .unwrap()
                    .result
                    .unwrap()["candidate"],
                "bob ready"
            );
            handle.cancel("alice", next_alice.run_id());
            next_alice.finished().await.unwrap();
            handle.shutdown();
            let stats = controller.join().await.unwrap();
            assert_eq!(stats.completed_runners, 3);
            assert_eq!((stats.peak_active_runs, stats.peak_queued_runs), (1, 2));
            storage.shutdown().await.unwrap();
        });
    }
}
