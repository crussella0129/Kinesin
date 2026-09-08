//! Synthetic measurements with the real journal; never contacts a live model.
use futures_util::stream::{FuturesUnordered, StreamExt};
use kinesin::config::Config;
use kinesin::core::ModelReply;
use kinesin::model::{ModelClient, ScriptStep};
use kinesin::policy::{RunAuthority, Submission, sha256};
use kinesin::runner::{self, RunResources};
use kinesin::scheduler::{Controller, ControllerHandle, ControllerStats, Job};
use kinesin::storage::{Command, Event, QueueLimits, Response, Storage, StorageClient};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;

const CONFIG: &str = r#"
version = 1
instructions = "Synthetic measurement; no live model or file effects."
[storage]
path = "state/kinesin.sqlite"
capture = "metadata"
[limits]
max_model_turns = 1
max_run_s = 30
[[workspaces]]
id = "practice"
root = "workspace"
tools = []
[[models]]
id = "fake"
base_url = "http://127.0.0.1:1"
model_id = "scripted"
context_size = 4096
verified_slots = 2
temperature = 0.0
[[owners]]
id = "alice"
workspaces = ["practice"]
models = ["fake"]
allow_freeform = true
[[owners]]
id = "bob"
workspaces = ["practice"]
models = ["fake"]
allow_freeform = true
"#;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Options {
    output: PathBuf,
    warm_trials: usize,
    cancel_trials: usize,
    soak_seconds: u64,
    warm_only: bool,
    curves_only: bool,
}

fn options() -> Result<Options> {
    let mut args = std::env::args().skip(1);
    let mut options = Options {
        output: args.next().ok_or("measure NEW_OUTPUT_DIRECTORY [--warm-only | --curves-only] [--soak-seconds 0..600] [--warm-trials 1..1000] [--cancel-trials 1..100]")?.into(),
        warm_trials: 1000,
        cancel_trials: 100,
        soak_seconds: 0,
        warm_only: false,
        curves_only: false,
    };
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--warm-only" => options.warm_only = true,
            "--curves-only" => options.curves_only = true,
            "--soak-seconds" => {
                options.soak_seconds = args.next().ok_or("missing soak seconds")?.parse()?
            }
            "--warm-trials" => {
                options.warm_trials = args.next().ok_or("missing warm trials")?.parse()?
            }
            "--cancel-trials" => {
                options.cancel_trials = args.next().ok_or("missing cancel trials")?.parse()?
            }
            _ => return Err(format!("unknown option: {flag}").into()),
        }
    }
    if !(1..=1000).contains(&options.warm_trials)
        || !(1..=100).contains(&options.cancel_trials)
        || options.soak_seconds > 600
        || (options.warm_only && options.soak_seconds != 0)
        || (options.curves_only && (options.warm_only || options.soak_seconds != 0))
    {
        return Err(
            "measurement options exceed bounded ranges or combine incompatible modes".into(),
        );
    }
    Ok(options)
}

fn file_hash(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes = file.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buffer[..bytes]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn csv(path: &Path, header: &str) -> Result<BufWriter<File>> {
    let mut file = BufWriter::new(File::create_new(path)?);
    writeln!(file, "{header}")?;
    Ok(file)
}

fn authority(config: &Config, owner: Option<&str>) -> Result<RunAuthority> {
    let submission = Submission::Freeform {
        workspace: "practice".into(),
        model: "fake".into(),
        continues: None,
        prompt: "measurement".into(),
        limits: None,
        capture: None,
    };
    Ok(match owner {
        Some(owner) => config.authorize(owner, submission)?,
        None => config.authorize_local(submission)?,
    })
}

fn scripted_job(authority: RunAuthority, resources: &RunResources, delay_ms: u64) -> Job {
    Job {
        authority,
        client: ModelClient::scripted([ScriptStep {
            delay: Duration::from_millis(delay_ms),
            reply: ModelReply::Answer("synthetic candidate".into()),
        }]),
        resources: resources.clone(),
        display: None,
    }
}

async fn events(store: &StorageClient, authority: &RunAuthority) -> Result<Vec<Event>> {
    match store
        .execute(
            Command::Events {
                owner_id: authority.owner().into(),
                run_id: authority.run_id().into(),
                after: None,
                limit: 100,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await?
    {
        Response::Events(events) => Ok(events),
        _ => Err("unexpected event response".into()),
    }
}

async fn warm(
    options: &Options,
    config: &Config,
    store: &StorageClient,
    resources: &RunResources,
) -> Result<()> {
    let mut samples = csv(
        &options.output.join("samples.csv"),
        "experiment,index,elapsed_us,admission_us,phase,acceptance",
    )?;
    let mut measured = Vec::with_capacity(options.warm_trials);
    for index in 0..options.warm_trials + 20 {
        let authority = authority(config, None)?;
        let accepted = Instant::now();
        runner::admit(&authority, store, None).await?;
        let admission_us = accepted.elapsed().as_micros();
        let job = scripted_job(authority.clone(), resources, 0);
        let record = runner::run_admitted(
            job.authority,
            job.client,
            store.clone(),
            job.resources,
            CancellationToken::new(),
            accepted,
        )
        .await?;
        let total_us = accepted.elapsed().as_micros();
        if record.phase != "completed" || record.acceptance_status != "unchecked" {
            return Err("warm run did not complete unchecked".into());
        }
        if index >= 20 {
            writeln!(
                samples,
                "warm,{},{total_us},{admission_us},{},{}",
                index - 20,
                record.phase,
                record.acceptance_status
            )?;
            measured.push((authority, total_us, admission_us));
        }
        if index >= 20 && (index - 19) % 250 == 0 {
            println!("warm {}", index - 19);
        }
    }
    samples.flush()?;
    // Query traces after all samples, so profiling reads do not perturb their workload.
    let mut breakdown = csv(
        &options.output.join("breakdown.csv"),
        "index,total_us,admission_us,journal_before_terminal_ms,terminal_event_ms,terminal_tail_upper_us,model_duration_ms",
    )?;
    for (index, (authority, total, admission)) in measured.iter().enumerate() {
        let events = events(store, authority).await?;
        let terminal = events
            .iter()
            .find(|event| event.kind == "run_finished")
            .ok_or("terminal event missing")?;
        let journal = terminal.data["journal_wait_before_terminal_ms"]
            .as_u64()
            .ok_or("journal timing missing")?;
        let model_ms: u64 = events
            .iter()
            .filter(|event| event.kind == "model_finished")
            .filter_map(|event| event.data["duration_ms"].as_u64())
            .sum();
        let tail = total.saturating_sub(u128::from(terminal.elapsed_ms) * 1000);
        writeln!(
            breakdown,
            "{index},{total},{admission},{journal},{},{tail},{model_ms}",
            terminal.elapsed_ms
        )?;
    }
    breakdown.flush()?;
    Ok(())
}

fn settled(stats: &ControllerStats) -> Result<()> {
    if stats.active_runs != 0
        || stats.queued_runs != 0
        || stats.queued_input_bytes != 0
        || stats.runner_errors != 0
    {
        return Err(format!("controller did not settle cleanly: {stats:?}").into());
    }
    Ok(())
}

async fn idle(controller: &ControllerHandle) -> Result<()> {
    timeout(Duration::from_secs(5), async {
        loop {
            let changed = controller.capacity_changed();
            tokio::pin!(changed);
            let stats = controller.stats();
            if stats.active_runs == 0 && stats.queued_runs == 0 {
                return;
            }
            changed.await;
        }
    })
    .await
    .map_err(|_| "controller did not become idle")?;
    Ok(())
}

// The peer accepts one real HTTP request, then withholds the advertised response
// body. Its EOF measures transport cleanup, not a remote model's compute lifetime.
async fn stalled_peer(
    listener: TcpListener,
    ready: tokio::sync::oneshot::Sender<()>,
    stop: CancellationToken,
) -> Result<Option<Instant>> {
    let (mut stream, _) = tokio::select! {
        _ = stop.cancelled() => return Ok(None),
        accepted = listener.accept() => accepted?,
    };
    let mut request = Vec::new();
    let mut buffer = [0_u8; 2048];
    loop {
        let count = tokio::select! {
            _ = stop.cancelled() => return Ok(None),
            read = stream.read(&mut buffer) => read?,
        };
        if count == 0 {
            return Err("request closed before completion".into());
        }
        request.extend_from_slice(&buffer[..count]);
        if request.len() > 131072 {
            return Err("synthetic request exceeded bound".into());
        }
        if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            let headers = std::str::from_utf8(&request[..end])?;
            let length: usize = headers
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .ok_or("content-length missing")?
                .1
                .trim()
                .parse()?;
            if length > 131072 - end - 4 {
                return Err("synthetic request body exceeded bound".into());
            }
            if request.len() >= end + 4 + length {
                break;
            }
        }
    }
    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 65536\r\nConnection: close\r\n\r\n").await?;
    let _ = ready.send(());
    tokio::select! {
        _ = stop.cancelled() => Ok(None),
        read = stream.read(&mut buffer[..1]) => match read {
            Ok(0) => Ok(Some(Instant::now())),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted) => Ok(Some(Instant::now())),
            Ok(_) => Err("unexpected peer bytes after request".into()),
            Err(error) => Err(error.into()),
        },
    }
}

async fn cancellations(options: &Options, config: &Config, store: &StorageClient) -> Result<()> {
    let mut output = csv(
        &options.output.join("cancellation.csv"),
        "index,cancel_call_us,peer_close_us,terminal_us,phase,acceptance",
    )?;
    let (controller, owner) =
        Controller::start(config.concurrency().clone(), store.clone(), false)?;
    let work: Result<()> = async {
        for index in 0..options.cancel_trials {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let origin = format!("http://{}", listener.local_addr()?);
            let http_config = Config::parse(
                &CONFIG.replace("http://127.0.0.1:1", &origin),
                &options.output.join("measure.toml"),
            )?;
            let authority = authority(&http_config, None)?;
            let resources = RunResources::from_config(&http_config)?
                .remove("fake")
                .ok_or("missing model resources")?;
            let client =
                ModelClient::http(authority.model(), authority.limits().max_response_bytes)?;
            let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
            let stop = CancellationToken::new();
            // The spawned peer uses a Send error type across the task boundary.
            let mut peer = tokio::spawn({
                let stop = stop.clone();
                async move {
                    stalled_peer(listener, ready_tx, stop)
                        .await
                        .map_err(|error| error.to_string())
                }
            });
            let trial: Result<_> = async {
                let pending = controller.try_submit(
                    Job {
                        authority: authority.clone(),
                        client,
                        resources,
                        display: None,
                    },
                    None,
                )?;
                timeout(Duration::from_secs(5), ready_rx).await??;
                let requested = Instant::now();
                if !controller.cancel(authority.owner(), pending.run_id()) {
                    return Err("cancellation did not find owned run".into());
                }
                let cancel_us = requested.elapsed().as_micros();
                let record = pending.finished().await?;
                let terminal_us = requested.elapsed().as_micros();
                if record.phase != "cancelled" || record.acceptance_status != "unchecked" {
                    return Err("cancelled run outcome mismatch".into());
                }
                let events = events(store, &authority).await?;
                if events
                    .iter()
                    .filter(|event| event.kind == "model_planned")
                    .count()
                    != 1
                    || events.iter().any(|event| event.kind == "tool_planned")
                {
                    return Err("cancellation dispatched unexpected effects".into());
                }
                Ok((requested, cancel_us, terminal_us, record))
            }
            .await;
            if trial.is_err() {
                stop.cancel();
            }
            let joined = match timeout(Duration::from_secs(5), &mut peer).await {
                Ok(joined) => joined,
                Err(_) => {
                    stop.cancel();
                    peer.await
                }
            };
            let closed = joined??;
            let (requested, cancel_us, terminal_us, record) = trial?;
            let peer_us = closed
                .map(|closed| {
                    closed
                        .saturating_duration_since(requested)
                        .as_micros()
                        .to_string()
                })
                .unwrap_or_else(|| "unobserved".into());
            writeln!(
                output,
                "{index},{cancel_us},{peer_us},{terminal_us},{},{}",
                record.phase, record.acceptance_status
            )?;
            if (index + 1) % 25 == 0 {
                println!("cancellations {}", index + 1);
            }
        }
        output.flush()?;
        Ok(())
    }
    .await;
    controller.shutdown();
    let joined = owner.join().await;
    work?;
    settled(&joined?)
}

async fn fairness(
    options: &Options,
    config: &Config,
    store: &StorageClient,
    resources: &RunResources,
) -> Result<()> {
    let mut output = csv(
        &options.output.join("fairness.csv"),
        "cycle,owner_rejection_us,bob_finished_us,alice_accepted,bob_accepted",
    )?;
    let (controller, owner) = Controller::start(config.concurrency().clone(), store.clone(), true)?;
    let work: Result<()> = async {
        for cycle in 0..20 {
            let mut pending = Vec::new();
            for _ in 0..6 { pending.push(controller.try_submit(scripted_job(authority(config, Some("alice"))?, resources, 100), None)?); }
            let excess = scripted_job(authority(config, Some("alice"))?, resources, 100);
            let rejected = Instant::now();
            if !matches!(controller.try_submit(excess, None), Err(ref code) if code == "controller_owner_overloaded") {
                return Err("fairness saturation was not sustained; inspect host stalls and owner limits".into());
            }
            let rejected_us = rejected.elapsed().as_micros();
            let bob = scripted_job(authority(config, Some("bob"))?, resources, 1);
            let bob_start = Instant::now();
            let record = controller.try_submit(bob, None)?.finished().await?;
            let bob_us = bob_start.elapsed().as_micros();
            if record.phase != "completed" { return Err("other owner did not complete".into()); }
            for pending in pending { if pending.finished().await?.phase != "completed" { return Err("saturated owner did not complete".into()); } }
            idle(&controller).await?;
            writeln!(output, "{cycle},{rejected_us},{bob_us},6,1")?;
        }
        output.flush()?;
        Ok(())
    }.await;
    controller.shutdown();
    let joined = owner.join().await;
    work?;
    settled(&joined?)
}

async fn soak(
    options: &Options,
    config: &Config,
    store: &StorageClient,
    resources: &RunResources,
) -> Result<()> {
    let mut output = csv(&options.output.join("overload.csv"), "cycle,elapsed_us")?;
    let (controller, owner) =
        Controller::start(config.concurrency().clone(), store.clone(), false)?;
    let work: Result<_> = async {
        let started = Instant::now();
        let mut cycle = 0;
        let mut completed = 0;
        let mut reported = 0;
        while started.elapsed() < Duration::from_secs(options.soak_seconds) {
            let mut pending = Vec::with_capacity(24);
            for _ in 0..25 {
                let job = scripted_job(authority(config, None)?, resources, 100);
                let attempt = Instant::now();
                match controller.try_submit(job, None) {
                    Ok(run) => pending.push(run),
                    Err(code) if code == "controller_overloaded" => {
                        writeln!(output, "{cycle},{}", attempt.elapsed().as_micros())?
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            let stats = controller.stats();
            if stats.active_runs > 8
                || stats.queued_runs > 16
                || stats.queued_input_bytes > 1_048_576
            {
                return Err("controller exceeded configured bounds".into());
            }
            for pending in pending {
                let record = pending.finished().await?;
                if record.phase != "completed" || record.acceptance_status != "unchecked" {
                    return Err("soak run outcome mismatch".into());
                }
                completed += 1;
            }
            idle(&controller).await?;
            cycle += 1;
            let elapsed = started.elapsed().as_secs();
            if elapsed / 10 > reported {
                reported = elapsed / 10;
                println!("soak {elapsed}s, {completed} completed, {cycle} cycles");
            }
        }
        output.flush()?;
        Ok((completed, cycle, started.elapsed().as_millis()))
    }
    .await;
    controller.shutdown();
    let joined = owner.join().await;
    let (completed, cycles, elapsed_ms) = work?;
    let stats = joined?;
    settled(&stats)?;
    std::fs::write(
        options.output.join("soak-summary.json"),
        serde_json::to_vec_pretty(&json!({
            "completed":completed,"cycles":cycles,"elapsed_ms":elapsed_ms,"rejections":stats.rejected_runs,
            "peak_active":stats.peak_active_runs,"peak_queued":stats.peak_queued_runs,"peak_queued_bytes":stats.peak_queued_input_bytes,
            "final_active":stats.active_runs,"final_queued":stats.queued_runs,"final_queued_bytes":stats.queued_input_bytes,"runner_errors":stats.runner_errors,
            "workload":"closed-loop bursts; 25 offered, bounded 8 active + 16 queued, two backend slots held for 100ms"
        }))?,
    )?;
    if stats.rejected_runs == 0 {
        return Err("soak produced no overload samples; overload latency is unmeasured".into());
    }
    Ok(())
}

fn csv_cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

/// Closed-loop phases; the futures below observe controller-owned runs and do
/// not spawn another task or acquire ownership of the underlying runner.
async fn curve_scenario(
    options: &Options,
    config: &Config,
    store: &StorageClient,
    resources: &RunResources,
    name: &str,
    active_cap: usize,
    phases: &[(&str, usize, u64)],
) -> Result<()> {
    let mut output = csv(
        &options.output.join(format!("{name}-runs.csv")),
        "phase,cycle,slot,active_cap,backend_slots,script_delay_ms,elapsed_us,phase_outcome,acceptance,error",
    )?;
    let mut limits = config.concurrency().clone();
    limits.max_active_runs = active_cap;
    limits.max_queued_runs = 0;
    let (controller, owner) = Controller::start(limits, store.clone(), false)?;
    let mut summaries = Vec::new();
    let work: Result<()> = async {
        for &(phase, batches, delay_ms) in phases {
            let started = Instant::now();
            let (mut offered, mut admitted, mut rejected, mut completed, mut failed) = (0, 0, 0, 0, 0);
            let mut peak_active = 0;
            let phase_work: Result<()> = async {
                for cycle in 0..batches {
                    let mut completions = FuturesUnordered::new();
                    for slot in 0..active_cap {
                        let job = scripted_job(authority(config, None)?, resources, delay_ms);
                        offered += 1;
                        let submitted = Instant::now();
                        match controller.try_submit(job, None) {
                            Ok(pending) => {
                                admitted += 1;
                                peak_active = peak_active.max(controller.stats().active_runs);
                                completions.push(async move {
                                    let record = pending.finished().await;
                                    (slot, submitted.elapsed().as_micros(), record)
                                });
                            }
                            Err(error) => {
                                rejected += 1;
                                writeln!(output, "{phase},{cycle},{slot},{active_cap},2,{delay_ms},{},rejected,,{}", submitted.elapsed().as_micros(), csv_cell(&error))?;
                            }
                        }
                    }
                    while let Some((slot, elapsed_us, result)) = completions.next().await {
                        match result {
                            Ok(record) => {
                                if record.phase == "completed" && record.acceptance_status == "unchecked" {
                                    completed += 1;
                                } else {
                                    failed += 1;
                                }
                                writeln!(output, "{phase},{cycle},{slot},{active_cap},2,{delay_ms},{elapsed_us},{},{},", record.phase, record.acceptance_status)?;
                            }
                            Err(error) => {
                                failed += 1;
                                writeln!(output, "{phase},{cycle},{slot},{active_cap},2,{delay_ms},{elapsed_us},runner_error,,{}", csv_cell(&error))?;
                            }
                        }
                    }
                    idle(&controller).await?;
                }
                if rejected != 0 || failed != 0 || completed != batches * active_cap || peak_active != active_cap {
                    return Err("curve did not achieve its intended concurrency and outcomes".into());
                }
                Ok(())
            }.await;
            let stats = controller.stats();
            summaries.push(json!({
                "phase":phase,"batches":batches,"script_delay_ms":delay_ms,"offered":offered,
                "admitted":admitted,"rejected":rejected,"completed_unchecked":completed,"failed_outcomes":failed,
                "elapsed_ms":started.elapsed().as_millis(),"peak_active":peak_active,
                "final_active":stats.active_runs,"final_queued":stats.queued_runs,"final_queued_bytes":stats.queued_input_bytes,
                "measurement_error":phase_work.as_ref().err().map(ToString::to_string)
            }));
            output.flush()?;
            phase_work?;
            println!("{name}/{phase}: {completed} completed, {rejected} rejected, peak {peak_active}");
        }
        Ok(())
    }.await;
    controller.shutdown();
    let joined = owner.join().await;
    let final_stats = joined.as_ref().ok().map(|stats| json!({
        "active":stats.active_runs,"queued":stats.queued_runs,"queued_bytes":stats.queued_input_bytes,
        "runner_errors":stats.runner_errors,"completed_runners":stats.completed_runners
    }));
    std::fs::write(
        options.output.join(format!("{name}-summary.json")),
        serde_json::to_vec_pretty(&json!({
            "name":name,"active_cap":active_cap,"queue_cap":0,"backend_slots":2,"phases":summaries,
            "final_stats":final_stats,"measurement_error":work.as_ref().err().map(ToString::to_string),
            "scope":"closed-loop fixed batches; per-run latency includes injected positive delay; descriptive tails, not open-loop service load"
        }))?,
    )?;
    work?;
    settled(&joined?)
}

async fn curves(
    options: &Options,
    config: &Config,
    store: &StorageClient,
    resources: &RunResources,
) -> Result<()> {
    for active in [1, 2, 4, 8] {
        curve_scenario(
            options,
            config,
            store,
            resources,
            &format!("concurrency-{active}"),
            active,
            &[("steady", 20, 100)],
        )
        .await?;
    }
    // Keep the same controller/resources alive through slowdown and recovery.
    curve_scenario(
        options,
        config,
        store,
        resources,
        "slowdown",
        4,
        &[
            ("initial", 10, 100),
            ("slow", 10, 400),
            ("recovered", 10, 100),
        ],
    )
    .await
}

fn main() -> Result<()> {
    let options = options()?;
    // Fresh outputs preserve earlier negatives and avoid reusing accumulated state.
    std::fs::create_dir(&options.output)?;
    std::fs::create_dir(options.output.join("workspace"))?;
    std::fs::write(options.output.join("measure.toml"), CONFIG)?;
    let config = Config::parse(CONFIG, &options.output.join("measure.toml"))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let resources = runtime
        .block_on(async { RunResources::from_config(&config) })?
        .remove("fake")
        .ok_or("model resources missing")?;
    std::fs::write(
        options.output.join("metadata.json"),
        serde_json::to_vec_pretty(&json!({
            "executable_sha256":file_hash(&std::env::current_exe()?)?,"config_sha256":sha256(CONFIG.as_bytes()),
            "cargo_lock_sha256":file_hash(Path::new("Cargo.lock"))?,"sqlite_version":rusqlite::version(),
            "writer_startup_checks":"WAL, synchronous FULL, foreign_keys ON", "retention_policy":config.storage().policy,
            "warmups":if options.curves_only {0} else {20},"warm_trials":if options.curves_only {0} else {options.warm_trials},"cancel_trials":if options.warm_only || options.curves_only {0} else {options.cancel_trials},
            "soak_seconds":options.soak_seconds,"warm_only":options.warm_only,"curves_only":options.curves_only,
            "scope":"synthetic freeform unchecked, metadata capture; real admission and SQLite journal; excludes configuration/authority construction"
        }))?,
    )?;
    let storage = Storage::start_with_policy(
        config.storage().path.clone(),
        QueueLimits::default(),
        config.storage().policy.clone(),
    )?;
    let store = storage.client();
    let experiments: Result<()> = runtime.block_on(async {
        if options.curves_only {
            curves(&options, &config, &store, &resources).await?;
        } else {
            warm(&options, &config, &store, &resources).await?;
        }
        if !options.warm_only && !options.curves_only {
            cancellations(&options, &config, &store).await?;
            fairness(&options, &config, &store, &resources).await?;
            if options.soak_seconds > 0 { soak(&options, &config, &store, &resources).await?; }
        }
        match store.execute(Command::Health, Instant::now() + Duration::from_secs(5)).await? {
            Response::Health { database_bytes, wal_bytes, reserved_bytes, runs } => {
                std::fs::write(options.output.join("storage-health.json"), serde_json::to_vec_pretty(&json!({"database_bytes":database_bytes,"wal_bytes":wal_bytes,"reserved_bytes":reserved_bytes,"runs":runs}))?)?;
                if reserved_bytes != 0 { return Err("storage reservations remained after measurements".into()); }
            }
            _ => return Err("unexpected storage health response".into()),
        }
        Ok(())
    });
    // Shutdown is unconditional, including CSV/measurement/controller failures.
    let shutdown = runtime.block_on(storage.shutdown());
    experiments?;
    shutdown?;
    println!("measurements and all owned workers settled");
    Ok(())
}
