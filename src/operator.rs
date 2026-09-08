//! Trusted provisioning and service startup; no model-selected paths or secrets.

use crate::auth::{self, Credentials, VerifierFile};
use crate::config::{BoundedConfig, Config};
use crate::ingress::{self, IngressLimits, IngressStats};
use crate::model::ModelClient;
use crate::private_state::{PrivateStatePolicy, PrivateStateReport, validate_private_tree};
use crate::runner::RunResources;
use crate::scheduler::{Controller, ControllerHandle};
use crate::service::{self, ServiceState};
use crate::storage::{QueueLimits, Storage};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

fn private_roots(config: &Config) -> Result<Vec<PrivateStateReport>, String> {
    let service = config
        .service()
        .ok_or("service configuration is required")?;
    let policy = PrivateStatePolicy::current(&service.trusted_state_sids)?;
    let state = config
        .storage()
        .path
        .parent()
        .ok_or("state needs a directory")?;
    let verifiers = service
        .credential_verifiers
        .parent()
        .ok_or("verifiers need a directory")?;
    let mut reports = vec![validate_private_tree(state, &policy)?];
    if verifiers != state {
        reports.push(validate_private_tree(verifiers, &policy)?);
    }
    Ok(reports)
}

fn owners(config: &Config) -> BTreeSet<String> {
    config
        .owners()
        .iter()
        .map(|owner| owner.id.clone())
        .collect()
}

pub fn provision(config_path: &Path, owner: &str, hours: u64) -> Result<u8, String> {
    let config = BoundedConfig::load(config_path)?;
    if !(1..=8760).contains(&hours) || !owners(&config).contains(owner) {
        return Err("invalid provisioning owner or expiry".into());
    }
    let _private = private_roots(&config)?;
    let path = &config
        .service()
        .ok_or("service configuration is required")?
        .credential_verifiers;
    let parent = path.parent().ok_or("verifiers need a directory")?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join("credentials.lock"))
        .map_err(|_| "cannot open provisioning lock")?;
    lock.try_lock()
        .map_err(|_| "another credential provisioning operation owns the lock")?;
    let mut records = match File::open(path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(65_537)
                .read_to_end(&mut bytes)
                .map_err(|_| "cannot read verifier records")?;
            Credentials::parse(&bytes, &owners(&config))?;
            serde_json::from_slice::<VerifierFile>(&bytes)
                .map_err(|_| "invalid verifier records")?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => VerifierFile {
            version: 1,
            credentials: Vec::new(),
        },
        Err(_) => return Err("cannot open verifier records".into()),
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "invalid clock")?
        .as_secs();
    let token = auth::provision(
        owner,
        now.checked_add(hours * 3600).ok_or("expiry overflow")?,
    )?;
    records.credentials.push(token.record.clone());
    let bytes = serde_json::to_vec_pretty(&records).map_err(|_| "cannot encode verifiers")?;
    Credentials::parse(&bytes, &owners(&config))?;
    let temporary = parent.join(format!("credentials-{}.tmp", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| "cannot create verifier update")?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "cannot persist verifier update")?;
    drop(file);
    std::fs::rename(&temporary, path).map_err(|_| "cannot publish verifier update")?;
    // Deliberate provisioning output. Normal commands never print this secret.
    writeln!(std::io::stdout().lock(), "{}", token.take_secret())
        .map_err(|_| "cannot deliver token; revoke its new verifier and provision again")?;
    Ok(0)
}

pub fn serve(config_path: &Path) -> Result<u8, String> {
    let config = Arc::new(BoundedConfig::load(config_path)?);
    let _private = private_roots(&config)?;
    let service = config
        .service()
        .ok_or("service configuration is required")?;
    let address = service.listen;
    let credentials = Arc::new(Credentials::load(
        &service.credential_verifiers,
        &owners(&config),
    )?);
    let resources = Arc::new(RunResources::from_config(&config)?);
    let clients = Arc::new(
        config
            .models()
            .iter()
            .map(|profile| {
                Ok((
                    profile.id.clone(),
                    ModelClient::http(profile, config.limits().max_response_bytes)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?,
    );
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| "runtime initialization failed")?;
    let concurrency = config.concurrency().clone();
    let storage = Storage::start_with_policy(
        config.storage().path.clone(),
        QueueLimits {
            commands: concurrency.journal_queue_events,
            bytes: concurrency.journal_queue_bytes,
            max_waiters: concurrency.max_active_runs + 48,
        },
        config.storage().policy.clone(),
    )
    .map_err(|error| error.to_string())?;
    let outcome = runtime.block_on(async move {
        let outcome = async {
            let listener = TcpListener::bind(address)
                .await
                .map_err(|_| "cannot bind private listener")?;
            let shutdown = CancellationToken::new();
            let (controller, owner) = Controller::start(concurrency, storage.client(), true)?;
            let state = match ServiceState::new(
                config.clone(),
                credentials.clone(),
                controller.clone(),
                storage.client(),
                resources,
                clients.clone(),
                shutdown.clone(),
            ) {
                Ok(state) => Arc::new(state),
                Err(error) => {
                    controller.shutdown();
                    owner.join().await?;
                    return Err(error);
                }
            };
            state.set_ready(false);
            let signal = {
                let shutdown = shutdown.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        biased;
                        _ = shutdown.cancelled() => Ok(()),
                        result = crate::signal::ctrl_c() => {
                            shutdown.cancel();
                            result.map_err(|_| "signal handler failed".to_owned())
                        },
                    }
                })
            };
            let monitor = tokio::spawn(readiness_monitor(
                config,
                credentials,
                clients,
                state.clone(),
                shutdown.clone(),
            ));
            eprintln!("Private service listening on {address}; remote exposure remains gated");
            let served = supervise(listener, state, controller, monitor, signal, shutdown).await;
            // No error propagation can skip the actual controller join. The
            // writer remains owned by the outer scope until every run settles.
            let joined = owner.join().await;
            eprintln!("Service shutdown: controller joined");
            joined?;
            served?;
            Ok(0)
        }
        .await;
        let stopped = storage.shutdown().await.map_err(|error| error.to_string());
        eprintln!("Service shutdown: journal joined");
        stopped?;
        outcome
    });
    eprintln!("Service shutdown: stopping runtime");
    drop(runtime);
    eprintln!("Service shutdown complete");
    outcome
}

async fn readiness_monitor(
    config: Arc<Config>,
    credentials: Arc<Credentials>,
    clients: Arc<BTreeMap<String, ModelClient>>,
    state: Arc<ServiceState>,
    shutdown: CancellationToken,
) -> Result<(), String> {
    loop {
        if shutdown.is_cancelled() {
            state.set_ready(false);
            return Ok(());
        }
        let path = config
            .service()
            .ok_or("service configuration is required")?
            .credential_verifiers
            .clone();
        let allowed_owners = owners(&config);
        let loaded = settle_reload(
            tokio::task::spawn_blocking(move || Credentials::load(&path, &allowed_owners)),
            &shutdown,
        )
        .await?;
        let Some(loaded) = loaded else {
            state.set_ready(false);
            return Ok(());
        };
        let mut ready = match loaded {
            Ok(replacement) => credentials.replace(replacement).is_ok(),
            Err(_) => false,
        };
        if !ready {
            credentials.disable_all();
            state.set_ready(false);
        }
        if ready {
            for profile in config.models() {
                let client = clients
                    .get(&profile.id)
                    .ok_or("readiness profile unavailable")?;
                let checked = tokio::select! {
                    biased;
                    _ = shutdown.cancelled() => { state.set_ready(false); return Ok(()); },
                    ready = client.ready(profile) => ready,
                };
                if !checked {
                    ready = false;
                    state.set_ready(false);
                    break;
                }
            }
        }
        // A stop between the final network response and publication cannot
        // restore readiness. Supervision also closes controller admission.
        if shutdown.is_cancelled() {
            state.set_ready(false);
            return Ok(());
        }
        state.set_ready(ready);
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => { state.set_ready(false); return Ok(()); },
            _ = tokio::time::sleep(Duration::from_secs(5)) => {},
        }
    }
}

/// Filesystem work cannot be cancelled by dropping a blocking JoinHandle. On
/// shutdown the supervisor stops admission immediately while this owner waits
/// for the one already submitted reload, discarding its result after a stop.
async fn settle_reload(
    mut reload: JoinHandle<Result<Credentials, String>>,
    shutdown: &CancellationToken,
) -> Result<Option<Result<Credentials, String>>, String> {
    tokio::select! {
        biased;
        _ = shutdown.cancelled() => {
            let _settled = reload.await.map_err(|_| "credential reload task failed")?;
            Ok(None)
        },
        loaded = &mut reload => {
            let loaded = loaded.map_err(|_| "credential reload task failed")?;
            if shutdown.is_cancelled() { Ok(None) } else { Ok(Some(loaded)) }
        },
    }
}

/// All background tasks are supervised while ingress is live. A panic or an
/// unexpected normal task exit closes readiness, admission, and ingress before
/// joining the remaining owners. Never detach a monitor on another error path.
async fn supervise(
    listener: TcpListener,
    state: Arc<ServiceState>,
    controller: ControllerHandle,
    mut monitor: JoinHandle<Result<(), String>>,
    mut signal: JoinHandle<Result<(), String>>,
    shutdown: CancellationToken,
) -> Result<(), String> {
    let serving = ingress::serve(
        listener,
        service::router(state.clone()),
        shutdown.clone(),
        IngressLimits::default(),
        Arc::new(IngressStats::default()),
    );
    tokio::pin!(serving);
    let mut served = None;
    let mut monitored = None;
    let mut signalled = None;
    tokio::select! {
        biased;
        result = &mut monitor => {
            let result = task_result(result, "readiness task failed");
            monitored = Some(if result.is_ok() && !shutdown.is_cancelled() {
                Err("readiness task stopped unexpectedly".into())
            } else { result });
        },
        result = &mut signal => {
            let result = task_result(result, "signal task failed");
            signalled = Some(if result.is_ok() && !shutdown.is_cancelled() {
                Err("signal task stopped unexpectedly".into())
            } else { result });
        },
        _ = shutdown.cancelled() => {},
        result = &mut serving => served = Some(result),
    }
    state.set_ready(false);
    controller.shutdown();
    shutdown.cancel();
    eprintln!("Service shutdown: admission closed");
    let served = match served {
        Some(result) => result,
        None => serving.await,
    };
    eprintln!("Service shutdown: ingress joined");
    let monitored = match monitored {
        Some(result) => result,
        None => task_result(monitor.await, "readiness task failed"),
    };
    eprintln!("Service shutdown: readiness monitor joined");
    let signalled = match signalled {
        Some(result) => result,
        None => task_result(signal.await, "signal task failed"),
    };
    eprintln!("Service shutdown: signal handler joined");
    // These results are all collected before any `?` can leave this scope.
    monitored?;
    signalled?;
    served
}

fn task_result(
    result: Result<Result<(), String>, tokio::task::JoinError>,
    code: &str,
) -> Result<(), String> {
    result.map_err(|_| code.to_owned())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_support::{BASE, Fixture, OWNER, TASK};
    use crate::policy::Submission;
    use crate::scheduler::Job;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::time::timeout;

    const SERVICE: &str = r#"
[service]
listen = "127.0.0.1:8081"
credential_verifiers = "verifiers.json"
max_submission_bytes = 65536
max_page_size = 20
idempotency_retention_hours = 24
"#;

    struct Harness {
        _fixture: Fixture,
        config: Arc<Config>,
        credentials: Arc<Credentials>,
        clients: Arc<BTreeMap<String, ModelClient>>,
        resources: Arc<BTreeMap<String, RunResources>>,
        state: Arc<ServiceState>,
        controller: ControllerHandle,
        owner: Controller,
        storage: Storage,
        shutdown: CancellationToken,
    }
    impl Harness {
        async fn new(text: &str) -> Self {
            let fixture = Fixture::new();
            let config = Arc::new(fixture.parse(text).unwrap());
            let token = auth::provision("alice", u64::MAX).unwrap();
            let bytes = serde_json::to_vec(&VerifierFile {
                version: 1,
                credentials: vec![token.record.clone()],
            })
            .unwrap();
            let credentials = Arc::new(Credentials::parse(&bytes, &owners(&config)).unwrap());
            std::fs::write(&config.service().unwrap().credential_verifiers, bytes).unwrap();
            let path = config.storage().path.clone();
            let storage =
                tokio::task::spawn_blocking(move || Storage::start(path, QueueLimits::default()))
                    .await
                    .unwrap()
                    .unwrap();
            let (controller, owner) =
                Controller::start(config.concurrency().clone(), storage.client(), true).unwrap();
            let resources = Arc::new(RunResources::from_config(&config).unwrap());
            let clients = Arc::new(
                config
                    .models()
                    .iter()
                    .map(|profile| {
                        (
                            profile.id.clone(),
                            ModelClient::http(profile, config.limits().max_response_bytes).unwrap(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
            );
            let shutdown = CancellationToken::new();
            let state = Arc::new(
                ServiceState::new(
                    config.clone(),
                    credentials.clone(),
                    controller.clone(),
                    storage.client(),
                    resources.clone(),
                    clients.clone(),
                    shutdown.clone(),
                )
                .unwrap(),
            );
            Self {
                _fixture: fixture,
                config,
                credentials,
                clients,
                resources,
                state,
                controller,
                owner,
                storage,
                shutdown,
            }
        }
        fn job(&self) -> Job {
            Job {
                authority: self
                    .config
                    .authorize(
                        "alice",
                        Submission::Freeform {
                            workspace: "practice".into(),
                            model: "local".into(),
                            prompt: "hello".into(),
                            limits: None,
                            capture: None,
                        },
                    )
                    .unwrap(),
                client: self.clients["local"].clone(),
                resources: self.resources["local"].clone(),
                display: None,
            }
        }
        async fn close(self) {
            self.shutdown.cancel();
            self.controller.shutdown();
            self.owner.join().await.unwrap();
            self.storage.shutdown().await.unwrap();
        }
    }

    fn signal(
        shutdown: CancellationToken,
        settled: Arc<AtomicBool>,
    ) -> JoinHandle<Result<(), String>> {
        tokio::spawn(async move {
            shutdown.cancelled().await;
            settled.store(true, Ordering::Release);
            Ok(())
        })
    }

    #[tokio::test]
    async fn readiness_panic_is_supervised_and_closes_admission_before_returning() {
        let harness = Harness::new(&format!("{BASE}{TASK}{OWNER}{SERVICE}")).await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let signal_settled = Arc::new(AtomicBool::new(false));
        let signal = signal(harness.shutdown.clone(), signal_settled.clone());
        let monitor = tokio::spawn(async { panic!("injected readiness monitor panic") });
        let result = timeout(
            Duration::from_secs(1),
            supervise(
                listener,
                harness.state.clone(),
                harness.controller.clone(),
                monitor,
                signal,
                harness.shutdown.clone(),
            ),
        )
        .await
        .unwrap();
        assert_eq!(result.unwrap_err(), "readiness task failed");
        assert!(harness.shutdown.is_cancelled());
        assert!(signal_settled.load(Ordering::Acquire));
        assert_eq!(
            harness
                .controller
                .try_submit(harness.job(), None)
                .err()
                .unwrap(),
            "controller_closed"
        );
        harness.close().await;
    }

    #[tokio::test]
    async fn shutdown_closes_admission_but_retains_an_inflight_blocking_reload_until_joined() {
        let harness = Harness::new(&format!("{BASE}{TASK}{OWNER}{SERVICE}")).await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let signal_settled = Arc::new(AtomicBool::new(false));
        let signal = signal(harness.shutdown.clone(), signal_settled.clone());
        let (release, released) = std::sync::mpsc::channel();
        let reloaded = Arc::new(AtomicBool::new(false));
        let worker = {
            let reloaded = reloaded.clone();
            tokio::task::spawn_blocking(move || {
                released.recv_timeout(Duration::from_secs(2)).unwrap();
                reloaded.store(true, Ordering::Release);
                Err("injected reload result".into())
            })
        };
        let monitor = {
            let shutdown = harness.shutdown.clone();
            tokio::spawn(async move {
                assert!(settle_reload(worker, &shutdown).await?.is_none());
                Ok(())
            })
        };
        let supervisor = tokio::spawn(supervise(
            listener,
            harness.state.clone(),
            harness.controller.clone(),
            monitor,
            signal,
            harness.shutdown.clone(),
        ));
        harness.shutdown.cancel();
        // The supervisor can close admission without abandoning the native read.
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(
            harness
                .controller
                .try_submit(harness.job(), None)
                .err()
                .unwrap(),
            "controller_closed"
        );
        assert!(!supervisor.is_finished());
        assert!(!reloaded.load(Ordering::Acquire));
        release.send(()).unwrap();
        timeout(Duration::from_secs(1), supervisor)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(reloaded.load(Ordering::Acquire));
        assert!(signal_settled.load(Ordering::Acquire));
        harness.close().await;
    }

    #[tokio::test]
    async fn stopping_readiness_cancels_the_network_wait_and_skips_remaining_profiles() {
        let first = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let second = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = BASE.replace(
            "http://127.0.0.1:8080",
            &format!("http://{}", first.local_addr().unwrap()),
        );
        let extra = format!(
            r#"
[[models]]
id = "second"
base_url = "http://{}"
model_id = "second-model"
context_size = 4096
verified_slots = 1
temperature = 0.0
"#,
            second.local_addr().unwrap()
        );
        let harness = Harness::new(&format!("{base}{extra}{TASK}{OWNER}{SERVICE}")).await;
        let monitor = tokio::spawn(readiness_monitor(
            harness.config.clone(),
            harness.credentials.clone(),
            harness.clients.clone(),
            harness.state.clone(),
            harness.shutdown.clone(),
        ));
        let (_blocked_socket, _) = timeout(Duration::from_secs(2), first.accept())
            .await
            .unwrap()
            .unwrap();
        harness.shutdown.cancel();
        timeout(Duration::from_millis(250), monitor)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            timeout(Duration::from_millis(30), second.accept())
                .await
                .is_err()
        );
        harness.close().await;
    }
}
