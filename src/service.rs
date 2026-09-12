//! Authenticated, bounded HTTP projections over the same local controller.
//! The connection server supplies a second bound before HTTP parsing. This
//! router never creates a task per submission or a queue per SSE observer.

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::body::to_bytes;
use axum::extract::{Extension, Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response, Sse, sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;

use crate::auth::Credentials;
use crate::config::Config;
use crate::model::ModelClient;
use crate::policy::Submission;
use crate::runner::RunResources;
use crate::scheduler::{ControllerHandle, Job};
use crate::storage::{
    Command, Event, Response as StoreResponse, RunCursor, RunRecord, StorageClient,
};

const MAX_FRAME_BYTES: usize = 8_192;
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const REQUEST_LIFETIME: Duration = Duration::from_secs(10);
const STREAM_LIFETIME: Duration = Duration::from_secs(300);

pub struct ServiceState {
    config: Arc<Config>,
    credentials: Arc<Credentials>,
    controller: ControllerHandle,
    store: StorageClient,
    resources: Arc<BTreeMap<String, RunResources>>,
    clients: Arc<BTreeMap<String, ModelClient>>,
    shutdown: CancellationToken,
    handlers: Arc<Semaphore>,
    observers: Arc<Observers>,
    ready: AtomicBool,
}

impl ServiceState {
    pub fn new(
        config: Arc<Config>,
        credentials: Arc<Credentials>,
        controller: ControllerHandle,
        store: StorageClient,
        resources: Arc<BTreeMap<String, RunResources>>,
        clients: Arc<BTreeMap<String, ModelClient>>,
        shutdown: CancellationToken,
    ) -> Result<Self, String> {
        if config.service().is_none()
            || config
                .models()
                .iter()
                .any(|m| !resources.contains_key(&m.id) || !clients.contains_key(&m.id))
        {
            return Err("service_startup_resources_missing".into());
        }
        let observers = Observers {
            global: Arc::new(Semaphore::new(
                config.concurrency().max_observers_global.min(16),
            )),
            per_run: config.concurrency().max_observers_per_run.min(2),
            runs: Mutex::new(BTreeMap::new()),
        };
        Ok(Self {
            config,
            credentials,
            controller,
            store,
            resources,
            clients,
            shutdown,
            handlers: Arc::new(Semaphore::new(32)),
            observers: Arc::new(observers),
            ready: AtomicBool::new(true),
        })
    }

    /// The trusted startup/health monitor controls readiness. Retrieval remains
    /// available while a backend is unavailable; new admissions fail closed.
    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::Release);
    }

    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire) && self.store.is_accepting()
    }

    /// Bounded operational counters; no owner, prompt, or credential labels.
    pub fn observer_count(&self) -> usize {
        self.observers
            .runs
            .lock()
            .expect("observer state mutex")
            .values()
            .sum()
    }

    async fn query(&self, command: Command) -> Result<StoreResponse, ApiError> {
        self.store
            .execute(
                command,
                Instant::now()
                    + Duration::from_secs(
                        self.config.concurrency().journal_admission_timeout_s.min(5),
                    ),
            )
            .await
            .map_err(|error| backend_error(error.code))
    }

    fn may_access(&self, owner: &str, run: &RunRecord) -> bool {
        self.config
            .owners()
            .iter()
            .find(|o| o.id == owner)
            .is_some_and(|o| {
                run.owner_id == owner
                    && o.workspaces.contains(&run.workspace_id)
                    && o.models.contains(&run.model_profile_id)
                    && (run.task_mode == "freeform" && o.allow_freeform
                        || run.task_mode == "checked"
                            && run
                                .task_profile_id
                                .as_ref()
                                .is_some_and(|id| o.tasks.contains(id)))
            })
    }

    async fn run(&self, owner: &str, id: &str) -> Result<RunRecord, ApiError> {
        validate_run_id(id)?;
        match self
            .query(Command::Get {
                owner_id: owner.into(),
                run_id: id.into(),
            })
            .await?
        {
            StoreResponse::Run(Some(run)) if self.may_access(owner, &run) => Ok(*run),
            StoreResponse::Run(_) => Err(not_found()),
            _ => Err(unavailable()),
        }
    }
}

pub fn router(state: Arc<ServiceState>) -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/v1/runs", post(create).get(list))
        .route("/v1/runs/{id}", get(status))
        .route("/v1/runs/{id}/cancel", post(cancel))
        .route("/v1/runs/{id}/export", get(export))
        .route("/v1/runs/{id}/events", get(events))
        .layer(middleware::from_fn_with_state(state.clone(), authenticate))
        // Process liveness has no storage, model, owner, or credential content.
        // It remains useful while readiness or verifier loading has failed.
        .route("/health", get(health))
        .with_state(state)
}

async fn health(uri: Uri) -> Result<Response, ApiError> {
    no_query(&uri)?;
    json_response(StatusCode::OK, json!({"alive":true}))
}

async fn ready(State(state): State<Arc<ServiceState>>, uri: Uri) -> Result<Response, ApiError> {
    no_query(&uri)?;
    let ready = state.is_ready();
    json_response(
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        json!({"ready":ready}),
    )
}

#[derive(Clone)]
struct Principal(String);

async fn authenticate(
    State(state): State<Arc<ServiceState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Header parsing and credential verification precede body/query decoding and
    // every storage request. No client-supplied owner extension is trusted.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let owner = single_header(request.headers(), header::AUTHORIZATION.as_str(), 160)
        .ok()
        .flatten()
        .and_then(|value| state.credentials.authenticate(value, now));
    let Some(owner) = owner else {
        return ApiError(StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    };
    if state.shutdown.is_cancelled() {
        return unavailable().into_response();
    }
    let Ok(_permit) = state.handlers.clone().try_acquire_owned() else {
        return unavailable().into_response();
    };
    if request.uri().query().is_some_and(|q| q.len() > 512) {
        return invalid().into_response();
    }
    request.extensions_mut().insert(Principal(owner));
    let mut response = match timeout(REQUEST_LIFETIME, next.run(request)).await {
        Ok(response) => response,
        Err(_) => ApiError(StatusCode::GATEWAY_TIMEOUT, "request_timeout").into_response(),
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    response
}

fn single_header<'a>(
    headers: &'a HeaderMap,
    name: &str,
    cap: usize,
) -> Result<Option<&'a str>, ApiError> {
    let mut values = headers.get_all(name).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() || value.as_bytes().len() > cap {
        return Err(invalid());
    }
    value.to_str().map(Some).map_err(|_| invalid())
}

async fn create(
    State(state): State<Arc<ServiceState>>,
    Extension(Principal(owner)): Extension<Principal>,
    request: Request,
) -> Result<Response, ApiError> {
    no_query(request.uri())?;
    let key = single_header(request.headers(), "idempotency-key", 128)?.ok_or_else(invalid)?;
    if key.is_empty() || !key.is_ascii() || key.bytes().any(|b| b.is_ascii_control()) {
        return Err(invalid());
    }
    let key = key.to_owned();
    let content_type =
        single_header(request.headers(), "content-type", 128)?.ok_or_else(invalid)?;
    if content_type.split(';').next().unwrap_or("").trim() != "application/json" {
        return Err(invalid());
    }
    let cap = state.config.service().unwrap().max_submission_bytes;
    if let Some(length) = single_header(request.headers(), "content-length", 20)? {
        let length = length.parse::<usize>().map_err(|_| invalid())?;
        if length > cap {
            return Err(ApiError(
                StatusCode::PAYLOAD_TOO_LARGE,
                "submission_too_large",
            ));
        }
    }
    let bytes = to_bytes(request.into_body(), cap)
        .await
        .map_err(|_| ApiError(StatusCode::PAYLOAD_TOO_LARGE, "submission_too_large"))?;
    let submission: Submission = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let authority = state
        .config
        .authorize(&owner, submission)
        .map_err(|_| invalid())?;
    if let Some(run) = state
        .controller
        .lookup_retry(&owner, &key, authority.submission_sha256())
        .await
        .map_err(|e| backend_error(&e))?
    {
        if !state.may_access(&owner, &run) {
            return Err(not_found());
        }
        return json_response(StatusCode::ACCEPTED, public_run(&run, true));
    }
    let model = &authority.model().id;
    if !state.is_ready() {
        return Err(unavailable());
    }
    let job = Job {
        display: None,
        client: state.clients.get(model).ok_or_else(unavailable)?.clone(),
        resources: state.resources.get(model).ok_or_else(unavailable)?.clone(),
        authority,
    };
    // Only declarations are attached here; active controller ownership must
    // precede process creation, including idempotency and queue arbitration.
    let job = job.discover_mcp(&state.config);
    // Ownership has transferred to the controller before this first await. A
    // dropped/timed-out HTTP future cannot abandon a committed admission.
    let mut pending = state
        .controller
        .try_submit(job, Some(key))
        .map_err(|e| backend_error(&e))?;
    let run = pending.admitted().await.map_err(|e| backend_error(&e))?;
    if !state.may_access(&owner, &run) {
        return Err(not_found());
    }
    json_response(StatusCode::ACCEPTED, public_run(&run, true))
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ListQuery {
    limit: Option<usize>,
    after_created_ms: Option<i64>,
    after_run_id: Option<String>,
}

async fn list(
    State(state): State<Arc<ServiceState>>,
    Extension(Principal(owner)): Extension<Principal>,
    uri: Uri,
) -> Result<Response, ApiError> {
    let query: ListQuery = Query::try_from_uri(&uri).map_err(|_| invalid())?.0;
    let limit = page_limit(&state, query.limit)?;
    let after = match (query.after_created_ms, query.after_run_id) {
        (None, None) => None,
        (Some(time), Some(id)) if time >= 0 => {
            validate_run_id(&id)?;
            Some(RunCursor {
                created_unix_ms: time,
                run_id: id,
            })
        }
        _ => return Err(invalid()),
    };
    let StoreResponse::Runs(runs) = state
        .query(Command::List {
            owner_id: owner.clone(),
            after,
            limit,
        })
        .await?
    else {
        return Err(unavailable());
    };
    let next = runs
        .last()
        .map(|r| json!({"after_created_ms":r.created_unix_ms,"after_run_id":r.run_id}));
    json_response(
        StatusCode::OK,
        json!({"runs":runs.iter().filter(|r| state.may_access(&owner,r)).map(|r| public_run(r,false)).collect::<Vec<_>>(), "next":next}),
    )
}

async fn status(
    State(state): State<Arc<ServiceState>>,
    Extension(Principal(owner)): Extension<Principal>,
    Path(id): Path<String>,
    uri: Uri,
) -> Result<Response, ApiError> {
    no_query(&uri)?;
    json_response(
        StatusCode::OK,
        public_run(&state.run(&owner, &id).await?, true),
    )
}

async fn cancel(
    State(state): State<Arc<ServiceState>>,
    Extension(Principal(owner)): Extension<Principal>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    no_query(request.uri())?;
    let bytes = to_bytes(request.into_body(), 0)
        .await
        .map_err(|_| invalid())?;
    if !bytes.is_empty() {
        return Err(invalid());
    }
    let run = state.run(&owner, &id).await?;
    cancel_run(&state, &owner, run).await
}

async fn cancel_run(
    state: &ServiceState,
    owner: &str,
    run: RunRecord,
) -> Result<Response, ApiError> {
    if terminal(&run) {
        return json_response(StatusCode::OK, public_run(&run, true));
    }
    let signalled = state.controller.cancel(owner, &run.run_id);
    // The runner may commit and release its slot after the initial owner query.
    // Refresh on either outcome; accepting a signal is not proof that work was
    // still running, and an absent slot cannot acknowledge a cancellation.
    let current = state.run(owner, &run.run_id).await?;
    if terminal(&current) {
        return json_response(StatusCode::OK, public_run(&current, true));
    }
    let mut value = public_run(&current, false);
    value["cancellation_requested"] = json!(signalled);
    if signalled {
        json_response(StatusCode::ACCEPTED, value)
    } else {
        value["error"] = json!("run_controller_unavailable");
        json_response(StatusCode::SERVICE_UNAVAILABLE, value)
    }
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ExportQuery {
    limit: Option<usize>,
    after: Option<u64>,
    replay: bool,
}

async fn export(
    State(state): State<Arc<ServiceState>>,
    Extension(Principal(owner)): Extension<Principal>,
    Path(id): Path<String>,
    uri: Uri,
) -> Result<Response, ApiError> {
    let query: ExportQuery = Query::try_from_uri(&uri).map_err(|_| invalid())?.0;
    if query.after.is_some_and(|seq| seq > i64::MAX as u64) {
        return Err(invalid());
    }
    let limit = page_limit(&state, query.limit)?;
    let run = state.run(&owner, &id).await?;
    if query.replay
        && !state
            .config
            .owners()
            .iter()
            .any(|o| o.id == owner && o.allow_replay)
    {
        return Err(ApiError(StatusCode::FORBIDDEN, "replay_not_permitted"));
    }
    if query.replay && run.capture != "replay" {
        return Err(ApiError(StatusCode::CONFLICT, "replay_unavailable"));
    }
    let StoreResponse::Events(events) = state
        .query(Command::Events {
            owner_id: owner,
            run_id: id,
            after: query.after,
            limit,
        })
        .await?
    else {
        return Err(unavailable());
    };
    let next = events.last().map(|e| e.seq);
    let exported: Vec<Value> = events
        .iter()
        .map(|event| {
            if query.replay {
                json!(event)
            } else {
                public_event(event, &run)
            }
        })
        .collect();
    // Replay pages explicitly contain private captures, not executable authority
    // or a claim of exact replay. Clients combine pages for offline validation.
    json_response(
        StatusCode::OK,
        json!({"format":if query.replay {"private_replay_page_v1"} else {"public_events_page_v1"},
        "run":if query.replay {json!(&run)} else {public_run(&run,true)},"task_accepted":run.task_accepted(),"events":exported,"next_after":next}),
    )
}

fn page_limit(state: &ServiceState, requested: Option<usize>) -> Result<usize, ApiError> {
    let cap = state.config.service().unwrap().max_page_size;
    let value = requested.unwrap_or(cap.min(20));
    if value == 0 || value > cap {
        return Err(invalid());
    }
    Ok(value)
}

// Serialize a deliberate whitelist. Admission keys, request fingerprints,
// ambient paths, prompts, and raw replay members never enter public summaries.
fn public_run(run: &RunRecord, include_result: bool) -> Value {
    let mut value = json!({"run_id":run.run_id,"workspace_id":run.workspace_id,"model_profile_id":run.model_profile_id,
        "task_mode":run.task_mode,"phase":run.phase,"acceptance_status":run.acceptance_status,
        "task_accepted":run.task_accepted(),"created_unix_ms":run.created_unix_ms,
        "contract":{"profile_id":run.task_profile_id,"profile_version":run.task_profile_version,
            "spec_sha256":run.task_spec_sha256,"checker_id":run.checker_id,"checker_version":run.checker_version},
        "terminal_reason":run.terminal_reason,"receipt":public_receipt(run.receipt.as_ref())});
    if include_result {
        value["result"] = run.result.clone().unwrap_or(Value::Null);
    }
    value
}

fn public_receipt(receipt: Option<&Value>) -> Value {
    let Some(receipt) = receipt else {
        return Value::Null;
    };
    let criteria = receipt["criteria"].as_array().map(|items| items.iter().take(4).map(|item| {
        let mut value = json!({"id":item["id"],"status":item["status"],"code":item["code"]});
        if let Some(evidence) = item.get("evidence") {
            value["evidence"] = json!({"evidence_id":evidence["evidence_id"],"effect_id":evidence["effect_id"],"seq":evidence["seq"],"sha256":evidence["sha256"]});
        }
        value
    }).collect::<Vec<_>>()).unwrap_or_default();
    let contract = receipt.get("contract").filter(|c| !c.is_null()).map(|c| json!({"profile_id":c["profile_id"],"profile_version":c["profile_version"],"checker_id":c["checker_id"],"checker_version":c["checker_version"],"spec_sha256":c["spec_sha256"]}));
    json!({"status":receipt["status"],"reason":receipt["reason"],"contract":contract,"candidate_sha256":receipt["candidate_sha256"],"criteria":criteria,"scope":receipt["scope"],"duration_ms":receipt["duration_ms"]})
}

fn public_event(event: &Event, run: &RunRecord) -> Value {
    let mut value = json!({"seq":event.seq,"kind":event_kind(event),"elapsed_ms":event.elapsed_ms});
    let mut summary = public_run(run, false);
    if matches!(event.kind.as_str(), "run_finished" | "recovery_interrupted") {
        value["run"] = summary;
    } else {
        // A retained historical event may be read after the run has finished.
        // Name this current state explicitly; the final receipt belongs only
        // to the terminal lifecycle event, following its atomic commit.
        summary.as_object_mut().unwrap().remove("receipt");
        summary.as_object_mut().unwrap().remove("terminal_reason");
        value["current_run"] = summary;
    }
    value
}

fn event_kind(event: &Event) -> &'static str {
    match event.kind.as_str() {
        "run_accepted" => "run_accepted",
        "model_planned" => "model_planned",
        "model_finished" => "model_finished",
        "tool_planned" => "tool_planned",
        "tool_finished" => "tool_finished",
        "run_finished" => "run_finished",
        "recovery_interrupted" => "recovery_interrupted",
        _ => "lifecycle_event",
    }
}

struct Observers {
    global: Arc<Semaphore>,
    per_run: usize,
    runs: Mutex<BTreeMap<(String, String), usize>>,
}
struct ObserverPermit {
    observers: Arc<Observers>,
    key: (String, String),
    _global: OwnedSemaphorePermit,
}
impl Observers {
    fn reserve(self: &Arc<Self>, owner: &str, id: &str) -> Result<ObserverPermit, ApiError> {
        let permit = self
            .global
            .clone()
            .try_acquire_owned()
            .map_err(|_| unavailable())?;
        let key = (owner.to_owned(), id.to_owned());
        let mut runs = self.runs.lock().expect("observer state mutex");
        let count = runs.entry(key.clone()).or_default();
        if *count >= self.per_run {
            return Err(ApiError(StatusCode::TOO_MANY_REQUESTS, "observer_limit"));
        }
        *count += 1;
        Ok(ObserverPermit {
            observers: self.clone(),
            key,
            _global: permit,
        })
    }
}
impl Drop for ObserverPermit {
    fn drop(&mut self) {
        let mut runs = self.observers.runs.lock().expect("observer state mutex");
        if let Some(count) = runs.get_mut(&self.key) {
            *count -= 1;
            if *count == 0 {
                runs.remove(&self.key);
            }
        }
    }
}

struct Observer {
    state: Arc<ServiceState>,
    owner: String,
    run_id: String,
    after: Option<u64>,
    changed: watch::Receiver<u64>,
    deadline: Instant,
    done: bool,
    _permit: ObserverPermit,
}

async fn events(
    State(state): State<Arc<ServiceState>>,
    Extension(Principal(owner)): Extension<Principal>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    no_query(request.uri())?;
    let after = single_header(request.headers(), "last-event-id", 20)?
        .map(|s| {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            let seq = s.parse::<u64>().map_err(|_| invalid())?;
            if seq > i64::MAX as u64 {
                return Err(invalid());
            }
            Ok(seq)
        })
        .transpose()?;
    // Subscribe before the first query; the watch channel is only a wakeup,
    // not event data. A commit between catch-up and changed() cannot be lost.
    let changed = state.store.subscribe();
    state.run(&owner, &id).await?;
    let permit = state.observers.reserve(&owner, &id)?;
    let observer = Observer {
        state,
        owner,
        run_id: id,
        after,
        changed,
        deadline: Instant::now() + STREAM_LIFETIME,
        done: false,
        _permit: permit,
    };
    let stream = stream::unfold(observer, |mut observer| async move {
        if observer.done {
            return None;
        }
        // A stalled consumer may first poll after the whole stream lifetime.
        // Stop before a historical lookup can compete with an expired timer.
        if let Some(code) = observer.stop_reason() {
            observer.done = true;
            return Some((Ok::<_, Infallible>(close_frame(code)), observer));
        }
        let shutdown = observer.state.shutdown.clone();
        let frame = tokio::select! {
            biased;
            _ = shutdown.cancelled() => Some(close_frame("service_shutdown")),
            _ = tokio::time::sleep_until(observer.deadline) => Some(close_frame("observer_lifetime_exceeded")),
            result = observer.next() => result,
        };
        // A lookup may finish at the same time as expiry or shutdown. Its
        // lifecycle frame must not replace the explicit closure notification.
        let frame = if let Some(code) = observer.stop_reason() {
            observer.done = true;
            Some(close_frame(code))
        } else {
            frame
        };
        frame.map(|frame| (Ok::<_, Infallible>(frame), observer))
    });
    Ok(Sse::new(stream)
        .keep_alive(sse::KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response())
}

impl Observer {
    fn stop_reason(&self) -> Option<&'static str> {
        if self.state.shutdown.is_cancelled() {
            Some("service_shutdown")
        } else if Instant::now() >= self.deadline {
            Some("observer_lifetime_exceeded")
        } else {
            None
        }
    }

    async fn next(&mut self) -> Option<sse::Event> {
        loop {
            let result = self
                .state
                .query(Command::Events {
                    owner_id: self.owner.clone(),
                    run_id: self.run_id.clone(),
                    after: self.after,
                    limit: 1,
                })
                .await;
            let Ok(StoreResponse::Events(mut events)) = result else {
                self.done = true;
                return Some(close_frame("storage_unavailable"));
            };
            let run = match self.state.run(&self.owner, &self.run_id).await {
                Ok(run) => run,
                Err(_) => {
                    self.done = true;
                    return Some(close_frame("run_unavailable"));
                }
            };
            if let Some(event) = events.pop() {
                let value = public_event(&event, &run);
                let Ok(data) = serde_json::to_string(&value) else {
                    self.done = true;
                    return Some(close_frame("projection_unavailable"));
                };
                // Include SSE field syntax, the decimal cursor and final newline.
                if data.len() + event.seq.to_string().len() + event_kind(&event).len() + 32
                    > MAX_FRAME_BYTES
                {
                    self.done = true;
                    return Some(close_frame("projection_limit"));
                }
                self.after = Some(event.seq);
                self.done = matches!(event.kind.as_str(), "run_finished" | "recovery_interrupted");
                return Some(
                    sse::Event::default()
                        .id(event.seq.to_string())
                        .event(event_kind(&event))
                        .data(data),
                );
            }
            // A terminal cursor reconnect has no new events. A catch-up that
            // saw a commit after its empty event query must query once more.
            if terminal(&run) {
                match self
                    .state
                    .query(Command::Events {
                        owner_id: self.owner.clone(),
                        run_id: self.run_id.clone(),
                        after: self.after,
                        limit: 1,
                    })
                    .await
                {
                    Ok(StoreResponse::Events(events)) if !events.is_empty() => continue,
                    Ok(StoreResponse::Events(_)) => return None,
                    _ => {
                        self.done = true;
                        return Some(close_frame("storage_unavailable"));
                    }
                }
            }
            if self.changed.changed().await.is_err() {
                self.done = true;
                return Some(close_frame("storage_unavailable"));
            }
        }
    }
}

fn close_frame(code: &str) -> sse::Event {
    sse::Event::default()
        .event("stream_closed")
        .data(json!({"code":code}).to_string())
}
fn terminal(run: &RunRecord) -> bool {
    matches!(
        run.phase.as_str(),
        "completed" | "stopped" | "failed" | "cancelled" | "interrupted"
    )
}
fn validate_run_id(id: &str) -> Result<(), ApiError> {
    if id.len() != 36 || uuid::Uuid::parse_str(id).is_err() {
        return Err(not_found());
    }
    Ok(())
}
fn no_query(uri: &Uri) -> Result<(), ApiError> {
    if uri.query().is_some() {
        Err(invalid())
    } else {
        Ok(())
    }
}

#[derive(Debug)]
struct ApiError(StatusCode, &'static str);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (self.0, Json(json!({"error":self.1}))).into_response();
        if self.0 == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, "Bearer".parse().unwrap());
        }
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
        response
    }
}
fn invalid() -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, "invalid_request")
}
fn not_found() -> ApiError {
    ApiError(StatusCode::NOT_FOUND, "run_not_found")
}
fn unavailable() -> ApiError {
    ApiError(StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
}
fn backend_error(code: &str) -> ApiError {
    match code {
        "storage_idempotency_conflict" => ApiError(StatusCode::CONFLICT, "idempotency_conflict"),
        "controller_owner_overloaded" => {
            ApiError(StatusCode::TOO_MANY_REQUESTS, "owner_quota_exhausted")
        }
        "invalid_idempotency_key" => invalid(),
        _ => unavailable(),
    }
}
fn json_response(status: StatusCode, value: Value) -> Result<Response, ApiError> {
    let bytes = serde_json::to_vec(&value).map_err(|_| unavailable())?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(unavailable());
    }
    Ok((status, [(header::CONTENT_TYPE, "application/json")], bytes).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{VerifierFile, provision};
    use crate::config::test_support::{BASE, Fixture, OWNER, TASK};
    use crate::runner;
    use crate::scheduler::Controller;
    use crate::storage::{QueueLimits, Storage};

    #[tokio::test]
    async fn cancellation_refreshes_a_stale_read_after_completion_and_never_acknowledges_an_absent_owner()
     {
        let fixture = Fixture::new();
        let config = Arc::new(
            fixture
                .parse(&format!(
                    r#"{BASE}{TASK}{OWNER}
[service]
listen = "127.0.0.1:8081"
credential_verifiers = "verifiers.json"
max_submission_bytes = 65536
max_page_size = 20
idempotency_retention_hours = 24
"#
                ))
                .unwrap(),
        );
        let path = config.storage().path.clone();
        let storage =
            tokio::task::spawn_blocking(move || Storage::start(path, QueueLimits::default()))
                .await
                .unwrap()
                .unwrap();
        let (controller, owner) =
            Controller::start(config.concurrency().clone(), storage.client(), true).unwrap();
        let token = provision("alice", u64::MAX).unwrap();
        let credentials = Arc::new(
            Credentials::parse(
                &serde_json::to_vec(&VerifierFile {
                    version: 1,
                    credentials: vec![token.record.clone()],
                })
                .unwrap(),
                &std::collections::BTreeSet::from(["alice".into()]),
            )
            .unwrap(),
        );
        let resources = Arc::new(RunResources::from_config(&config).unwrap());
        let client = ModelClient::scripted([crate::core::ModelReply::Answer(
            "finished before cancellation".into(),
        )
        .into()]);
        let clients = Arc::new(BTreeMap::from([("local".into(), client.clone())]));
        let shutdown = CancellationToken::new();
        let state = ServiceState::new(
            config.clone(),
            credentials,
            controller.clone(),
            storage.client(),
            resources.clone(),
            clients,
            shutdown.clone(),
        )
        .unwrap();
        let submission = Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            continues: None,
            prompt: "hello".into(),
            limits: None,
            capture: None,
        };
        let mut pending = controller
            .try_submit(
                Job {
                    authority: config.authorize("alice", submission.clone()).unwrap(),
                    client,
                    resources: resources["local"].clone(),
                    display: None,
                },
                None,
            )
            .unwrap();
        // Capture exactly the stale nonterminal projection a handler can hold
        // while its owning runner commits the terminal transaction.
        let stale = pending.admitted().await.unwrap();
        assert!(!terminal(&stale));
        let finished = pending.finished().await.unwrap();
        timeout(Duration::from_secs(1), async {
            while controller.stats().active_runs != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let response = cancel_run(&state, "alice", stale).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
        assert_eq!(value["phase"], finished.phase);
        assert_eq!(value["result"], finished.result.unwrap());
        assert!(value.get("cancellation_requested").is_none());

        let authority = config.authorize("alice", submission).unwrap();
        let orphaned = runner::admit(&authority, &storage.client(), None)
            .await
            .unwrap();
        let response = cancel_run(&state, "alice", orphaned).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
        assert_eq!(value["cancellation_requested"], false);
        assert_eq!(value["error"], "run_controller_unavailable");
        assert_eq!(value["phase"], "queued");
        shutdown.cancel();
        controller.shutdown();
        owner.join().await.unwrap();
        storage.shutdown().await.unwrap();
    }
}
