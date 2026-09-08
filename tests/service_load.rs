//! Explicit fixed-arrival experiment; excluded from ordinary offline checks.
//! Uses the real authenticated HTTP ingress/controller/writer and a delayed fake.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use kinesin::auth::{Credentials, VerifierFile, provision};
use kinesin::config::Config;
use kinesin::core::ModelReply;
use kinesin::ingress::{self, IngressLimits, IngressStats};
use kinesin::model::{ModelClient, ScriptStep};
use kinesin::runner::RunResources;
use kinesin::scheduler::{Controller, ControllerStats};
use kinesin::service::{ServiceState, router};
use kinesin::storage::{Command, QueueLimits, Response, Storage};
use serde::Serialize;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until, timeout};
use tokio_util::sync::CancellationToken;

const OFFERED: usize = 40;
const MAX_REQUESTS: usize = 8;
const MAX_RESPONSE_BYTES: usize = 8192;
const MODEL_DELAY: Duration = Duration::from_millis(100);
const CONFIG: &str = r#"
version = 1
instructions = "Synthetic load fixture."
[storage]
path = "state/kinesin.sqlite"
capture = "metadata"
[limits]
max_run_s = 30
[concurrency]
max_active_runs = 2
max_queued_runs = 4
max_queued_input_bytes = 65536
max_inflight_model_requests = 1
per_owner_active_runs = 2
per_owner_queued_runs = 4
[service]
listen = "127.0.0.1:8081"
credential_verifiers = "private/verifiers.json"
max_submission_bytes = 65536
max_page_size = 100
idempotency_retention_hours = 24
[[workspaces]]
id = "practice"
root = "workspace"
tools = []
[[models]]
id = "fake"
base_url = "http://127.0.0.1:1"
model_id = "synthetic-delayed"
context_size = 4096
verified_slots = 1
temperature = 0.0
[[owners]]
id = "load"
workspaces = ["practice"]
models = ["fake"]
allow_freeform = true
"#;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("kinesin-service-load-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        Self(root)
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
                .starts_with("kinesin-service-load-")
        );
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Serialize)]
struct Sample {
    tick: usize,
    send_lag_us: u64,
    response_us: u64,
    status: Option<u16>,
    error: Option<&'static str>,
    run_id: Option<String>,
}

#[derive(Serialize)]
struct Report {
    arrivals_per_second: u32,
    offered_ticks: usize,
    missed_tick_tolerance_us: u64,
    skipped_late_ticks: usize,
    skipped_generator_capacity: usize,
    dispatched_requests: usize,
    generator_peak_requests: usize,
    http_outcomes: BTreeMap<u16, usize>,
    response_errors: usize,
    acknowledged_admissions: usize,
    server_rejections: usize,
    durable_admissions: usize,
    completed_runs: usize,
    passed_tasks: usize,
    model_requests: usize,
    elapsed_until_requests_joined_ms: u64,
    elapsed_until_runs_settled_ms: u64,
    peak_active_runs: usize,
    peak_queued_runs: usize,
    peak_queued_input_bytes: usize,
    peak_ingress_connections: usize,
    ingress_rejections: usize,
    remaining_active_runs: usize,
    remaining_queued_runs: usize,
    remaining_queued_input_bytes: usize,
    remaining_ingress_connections: usize,
    remaining_observers: usize,
    remaining_journal_commands: usize,
    remaining_journal_bytes: usize,
    settled_before_shutdown: bool,
    controller_joined: bool,
    ingress_joined: bool,
    writer_joined: bool,
    tick_lag_us: Vec<u64>,
    requests: Vec<Sample>,
}

fn micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}
fn millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

async fn request(client: reqwest::Client, url: String, tick: usize, scheduled: Instant) -> Sample {
    let started = Instant::now();
    let mut sample = Sample {
        tick,
        send_lag_us: micros(started.saturating_duration_since(scheduled)),
        response_us: 0,
        status: None,
        error: None,
        run_id: None,
    };
    let result = async {
        let mut response = client.post(url)
            .header("idempotency-key", format!("arrival-{tick}"))
            .header("content-type", "application/json")
            .body(r#"{"mode":"freeform","workspace":"practice","model":"fake","prompt":"synthetic load"}"#)
            .send().await.map_err(|_| "request_transport")?;
        sample.status = Some(response.status().as_u16());
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| "response_transport")? {
            if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err("response_limit");
            }
            body.extend_from_slice(&chunk);
        }
        if sample.status == Some(202) {
            let value: serde_json::Value = serde_json::from_slice(&body).map_err(|_| "response_json")?;
            let id = value["run_id"].as_str().ok_or("response_run_id")?;
            uuid::Uuid::parse_str(id).map_err(|_| "response_run_id")?;
            sample.run_id = Some(id.to_owned());
        }
        Ok::<_, &'static str>(())
    }.await;
    sample.response_us = micros(started.elapsed());
    sample.error = result.err();
    sample
}

async fn scenario(rate: u32) -> Report {
    assert!((1..=100).contains(&rate) && OFFERED <= 100);
    let fixture = Fixture::new();
    let config = Arc::new(Config::parse(CONFIG, &fixture.0.join("kinesin.toml")).unwrap());
    let token = provision("load", u64::MAX).unwrap();
    let credentials = Arc::new(
        Credentials::parse(
            &serde_json::to_vec(&VerifierFile {
                version: 1,
                credentials: vec![token.record.clone()],
            })
            .unwrap(),
            &BTreeSet::from(["load".into()]),
        )
        .unwrap(),
    );
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token.take_secret()).parse().unwrap(),
    );
    let http = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .http1_only()
        .pool_max_idle_per_host(MAX_REQUESTS)
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let path = config.storage().path.clone();
    let storage = tokio::task::spawn_blocking(move || Storage::start(path, QueueLimits::default()))
        .await
        .unwrap()
        .unwrap();
    let (handle, controller) =
        Controller::start(config.concurrency().clone(), storage.client(), true).unwrap();
    let model = ModelClient::scripted((0..OFFERED).map(|_| ScriptStep {
        delay: MODEL_DELAY,
        reply: ModelReply::Answer("synthetic candidate".into()),
    }));
    let shutdown = CancellationToken::new();
    let state = Arc::new(
        ServiceState::new(
            config.clone(),
            credentials,
            handle.clone(),
            storage.client(),
            Arc::new(RunResources::from_config(&config).unwrap()),
            Arc::new(BTreeMap::from([("fake".into(), model.clone())])),
            shutdown.clone(),
        )
        .unwrap(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/v1/runs", listener.local_addr().unwrap());
    let ingress_stats = Arc::new(IngressStats::default());
    let server = tokio::spawn(ingress::serve(
        listener,
        router(state.clone()),
        shutdown.clone(),
        IngressLimits {
            max_connections: 16,
            ..Default::default()
        },
        ingress_stats.clone(),
    ));
    let period = Duration::from_secs_f64(1.0 / f64::from(rate));
    let started = Instant::now() + Duration::from_millis(25);
    let mut pending = JoinSet::new();
    let mut requests = Vec::with_capacity(OFFERED);
    let mut tick_lag_us = Vec::with_capacity(OFFERED);
    let mut skipped_late_ticks = 0;
    let mut skipped_generator_capacity = 0;
    let mut generator_peak_requests = 0;
    for tick in 0..OFFERED {
        let scheduled = started + period * tick as u32;
        loop {
            tokio::select! {
                biased;
                done = pending.join_next(), if !pending.is_empty() => requests.push(done.unwrap().unwrap()),
                _ = sleep_until(scheduled) => break,
            }
        }
        let lag = Instant::now().saturating_duration_since(scheduled);
        tick_lag_us.push(micros(lag));
        // Tolerate one interval of scheduling jitter. A full missed interval
        // is dropped explicitly; the generator never sends a catch-up burst.
        if lag >= period {
            skipped_late_ticks += 1;
        } else if pending.len() >= MAX_REQUESTS {
            skipped_generator_capacity += 1;
        } else {
            pending.spawn(request(http.clone(), url.clone(), tick, scheduled));
            generator_peak_requests = generator_peak_requests.max(pending.len());
        }
    }
    while let Some(done) = pending.join_next().await {
        requests.push(done.unwrap());
    }
    let elapsed_until_requests_joined_ms = millis(started.elapsed());
    let settled = timeout(Duration::from_secs(15), async {
        loop {
            let stats = handle.stats();
            if stats.active_runs == 0 && stats.queued_runs == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .is_ok();
    let elapsed_until_runs_settled_ms = millis(started.elapsed());
    // Even if settling times out, retain every owned task through cleanup.
    handle.shutdown();
    shutdown.cancel();
    drop(http);
    let server_result = server.await;
    let controller_result = controller.join().await;
    let final_stats = controller_result
        .as_ref()
        .cloned()
        .unwrap_or_else(|_| ControllerStats::default());
    let rows = storage
        .client()
        .execute(
            Command::List {
                owner_id: "load".into(),
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await;
    let (durable_admissions, completed_runs, passed_tasks) = match rows {
        Ok(Response::Runs(rows)) => (
            rows.len(),
            rows.iter().filter(|r| r.phase == "completed").count(),
            rows.iter().filter(|r| r.task_accepted()).count(),
        ),
        _ => (0, 0, 0),
    };
    let (remaining_journal_commands, remaining_journal_bytes) = storage.client().outstanding();
    let writer_joined = storage.shutdown().await.is_ok();
    requests.sort_by_key(|sample| sample.tick);
    let mut http_outcomes = BTreeMap::new();
    for sample in &requests {
        if let Some(status) = sample.status {
            *http_outcomes.entry(status).or_insert(0) += 1;
        }
    }
    Report {
        arrivals_per_second: rate,
        offered_ticks: OFFERED,
        missed_tick_tolerance_us: micros(period),
        skipped_late_ticks,
        skipped_generator_capacity,
        dispatched_requests: requests.len(),
        generator_peak_requests,
        response_errors: requests.iter().filter(|s| s.error.is_some()).count(),
        acknowledged_admissions: requests.iter().filter(|s| s.run_id.is_some()).count(),
        server_rejections: requests
            .iter()
            .filter(|s| matches!(s.status, Some(429 | 503)))
            .count(),
        http_outcomes,
        durable_admissions,
        completed_runs,
        passed_tasks,
        model_requests: model.captured_requests().unwrap().len(),
        elapsed_until_requests_joined_ms,
        elapsed_until_runs_settled_ms,
        peak_active_runs: final_stats.peak_active_runs,
        peak_queued_runs: final_stats.peak_queued_runs,
        peak_queued_input_bytes: final_stats.peak_queued_input_bytes,
        peak_ingress_connections: ingress_stats.peak.load(Ordering::Relaxed),
        ingress_rejections: ingress_stats.rejected.load(Ordering::Relaxed),
        remaining_active_runs: final_stats.active_runs,
        remaining_queued_runs: final_stats.queued_runs,
        remaining_queued_input_bytes: final_stats.queued_input_bytes,
        remaining_ingress_connections: ingress_stats.active.load(Ordering::Relaxed),
        remaining_observers: state.observer_count(),
        remaining_journal_commands,
        remaining_journal_bytes,
        settled_before_shutdown: settled,
        controller_joined: controller_result.is_ok(),
        ingress_joined: matches!(server_result, Ok(Ok(()))),
        writer_joined,
        tick_lag_us,
        requests,
    }
}

#[test]
#[ignore = "bounded timing experiment; run explicitly in a quiet local window"]
fn fixed_arrival_service_load_records_generator_and_server_outcomes() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let reports = runtime.block_on(async { vec![scenario(5).await, scenario(100).await] });
    drop(runtime);
    let output = std::env::var_os("KINESIN_SERVICE_LOAD_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(format!(
                "validation-output/service-load-{}.json",
                uuid::Uuid::new_v4()
            ))
        });
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let evidence = serde_json::json!({
        "version":1,"harness_version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,
        "generated_unix_ms":SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        "model_kind":"scripted","model_delay_ms":MODEL_DELAY.as_millis(),"model_slots":1,
        "max_generator_requests":MAX_REQUESTS,"max_active_runs":2,"max_queued_runs":4,
        "max_queued_input_bytes":65536,"request_timeout_s":5,"max_response_bytes":MAX_RESPONSE_BYTES,
        "reports":reports,
    });
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .unwrap();
    let bytes = serde_json::to_vec_pretty(&evidence).unwrap();
    assert!(bytes.len() <= 128 * 1024);
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    println!("service-load evidence: {}", output.display());
    for report in &reports {
        assert_eq!(
            report.offered_ticks,
            report.dispatched_requests
                + report.skipped_late_ticks
                + report.skipped_generator_capacity
        );
        assert!(report.generator_peak_requests <= MAX_REQUESTS);
        assert_eq!(report.response_errors, 0);
        assert_eq!(
            report.dispatched_requests,
            report.acknowledged_admissions + report.server_rejections
        );
        assert_eq!(report.durable_admissions, report.acknowledged_admissions);
        assert_eq!(report.completed_runs, report.durable_admissions);
        assert_eq!(report.model_requests, report.durable_admissions);
        assert_eq!(
            report.passed_tasks, 0,
            "freeform completion remains unchecked"
        );
        assert!(report.acknowledged_admissions > 0);
        assert!(
            report.peak_active_runs <= 2
                && report.peak_queued_runs <= 4
                && report.peak_queued_input_bytes <= 65536
        );
        assert!(report.peak_ingress_connections <= 16);
        assert_eq!(
            (
                report.remaining_active_runs,
                report.remaining_queued_runs,
                report.remaining_queued_input_bytes
            ),
            (0, 0, 0)
        );
        assert_eq!(
            (
                report.remaining_ingress_connections,
                report.remaining_observers,
                report.remaining_journal_commands,
                report.remaining_journal_bytes
            ),
            (0, 0, 0, 0)
        );
        assert!(
            report.settled_before_shutdown
                && report.controller_joined
                && report.ingress_joined
                && report.writer_joined
        );
    }
    assert!(
        reports[1].server_rejections > 0,
        "the high-rate scenario must actually exercise overload"
    );
}
