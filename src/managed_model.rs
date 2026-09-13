//! A selected local inference server belongs to this process, never to a run.
//! Readiness precedes admission; every normal stop settles its owned tree.

use std::collections::VecDeque;
use std::io::Read;
use std::net::{Ipv4Addr, TcpListener};
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::config::{ManagedModelConfig, ModelConfig};
use crate::model::ModelClient;
use crate::process::{OwnedProcess, scrub_environment};

pub const MAX_LOG_BYTES: usize = 16 * 1024;
const READY_POLL: Duration = Duration::from_millis(100);

pub struct ManagedServer {
    profile: ModelConfig,
    process: Option<OwnedProcess>,
    logs: Arc<Mutex<VecDeque<u8>>>,
    drain: Option<JoinHandle<Result<(), String>>>,
}

impl ManagedServer {
    /// The template identifies a local profile. The returned profile has this
    /// launch's loopback origin and unpredictable alias; callers must prepare
    /// authorities, resource pools and model clients from that returned profile.
    pub async fn start(
        spec: &ManagedModelConfig,
        template: &ModelConfig,
        cancel: &CancellationToken,
    ) -> Result<Self, String> {
        validate(spec, template)?;
        if cancel.is_cancelled() {
            return Err("local model startup cancelled".into());
        }
        // llama-server does not inherit a listening socket. Reserve an unused
        // port until immediately before spawn. A raced bind fails the owned
        // child; unique alias/readiness checks never adopt an unrelated server.
        let reservation = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .map_err(|_| "cannot reserve a local model port")?;
        let port = reservation
            .local_addr()
            .map_err(|_| "cannot inspect local model port")?
            .port();
        let mut profile = template.clone();
        profile.base_url = format!("http://127.0.0.1:{port}");
        profile.model_id = format!("kinesin-managed-{}", uuid::Uuid::new_v4().simple());
        let client = ModelClient::http(&profile, crate::config::MAX_RESPONSE_BYTES)?;
        let mut command = Command::new(&spec.executable);
        command.current_dir(
            spec.executable
                .parent()
                .ok_or("model runtime needs a parent directory")?,
        );
        command
            .arg("--model")
            .arg(&spec.model_path)
            .args(["--host", "127.0.0.1", "--port", &port.to_string()])
            .args([
                "--alias",
                &profile.model_id,
                "--ctx-size",
                &profile.context_size.to_string(),
            ])
            .args([
                "--parallel",
                "1",
                "--jinja",
                "--no-context-shift",
                "--no-webui",
                "--slots",
            ])
            .args([
                "--gpu-layers",
                &spec.gpu_layers.to_string(),
                "--threads",
                &spec.threads.to_string(),
                "--threads-batch",
                &spec.threads.to_string(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        scrub_environment(&mut command);
        let deadline = Instant::now() + Duration::from_secs(spec.startup_timeout_s);
        drop(reservation);
        let mut process = OwnedProcess::spawn_hidden(&mut command)
            .map_err(|error| format!("cannot start local model runtime: {error}"))?;
        let stderr = process
            .take_stderr()
            .ok_or("local model log pipe unavailable")?;
        let logs = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_BYTES)));
        let drain = tokio::spawn(drain_logs(stderr, logs.clone()));
        let mut server = Self {
            profile,
            process: Some(process),
            logs,
            drain: Some(drain),
        };
        let ready = server.await_ready(&client, deadline, cancel).await;
        if let Err(reason) = ready {
            let stopped = server.shutdown().await;
            let tail = server.log_tail();
            let mut error = reason;
            if !tail.is_empty() {
                error.push_str("\nLocal model log:\n");
                error.push_str(&tail);
            }
            if let Err(cleanup) = stopped {
                error.push_str(&format!("\n{cleanup}"));
            }
            return Err(error);
        }
        Ok(server)
    }

    pub fn profile(&self) -> &ModelConfig {
        &self.profile
    }

    /// Observe the owned leader while keeping group/job identity for teardown.
    /// Cancellation of this wait never abandons lifecycle ownership.
    pub async fn wait_for_exit(&mut self) -> Result<ExitStatus, String> {
        self.process
            .as_mut()
            .ok_or("local model is already stopped")?
            .wait_leader()
            .await
            .map_err(|error| format!("cannot observe local model runtime: {error}"))
    }

    /// Stop and reap this launch's tree, then finish its bounded log drainer.
    /// External profiles never construct a ManagedServer and cannot reach here.
    pub async fn shutdown(&mut self) -> Result<(), String> {
        let stopped = if let Some(process) = self.process.as_mut() {
            process
                .terminate_and_wait()
                .await
                .map_err(|error| format!("cannot stop local model runtime: {error}"))
        } else {
            Ok(())
        };
        self.process.take();
        if let Some(drain) = self.drain.take() {
            if stopped.is_err() {
                drain.abort();
                let _ = drain.await;
            } else {
                drain.await.map_err(|_| "local model log reader failed")??;
            }
        }
        stopped
    }

    /// A bounded, terminal-safe diagnostic. It contains runtime messages only;
    /// ordinary successful sessions do not print this log.
    pub fn log_tail(&self) -> String {
        let bytes: Vec<u8> = self
            .logs
            .lock()
            .expect("model log mutex")
            .iter()
            .copied()
            .collect();
        let raw = String::from_utf8_lossy(&bytes);
        let mut safe = String::new();
        for character in raw.chars() {
            if (character.is_control() && character != '\n' && character != '\t')
                || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            {
                use std::fmt::Write;
                let _ = write!(safe, "\\u{{{:x}}}", u32::from(character));
            } else {
                safe.push(character);
            }
        }
        safe
    }

    async fn await_ready(
        &mut self,
        client: &ModelClient,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        let profile = self.profile.clone();
        let readiness = async {
            loop {
                if client.ready(&profile).await {
                    return;
                }
                tokio::time::sleep(READY_POLL).await;
            }
        };
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err("local model startup cancelled".into()),
            status = self.wait_for_exit() => {
                let status = status?;
                Err(format!("local model runtime exited before becoming ready ({status})"))
            }
            _ = tokio::time::sleep_until(deadline) => Err("local model startup timed out before readiness".into()),
            _ = readiness => Ok(()),
        }
    }
}

impl Drop for ManagedServer {
    fn drop(&mut self) {
        // Fallback for a cancelled owner future. Normal callers await shutdown
        // after controller/journal settlement; dropping still signals the tree.
        self.process.take();
        if let Some(drain) = self.drain.take() {
            drain.abort();
        }
    }
}

fn validate(spec: &ManagedModelConfig, profile: &ModelConfig) -> Result<(), String> {
    if spec.model != profile.id || profile.verified_slots != 1 {
        return Err("local model must select its matching single-slot profile".into());
    }
    if !(1..=600).contains(&spec.startup_timeout_s)
        || spec.gpu_layers > 1000
        || !(1..=256).contains(&spec.threads)
    {
        return Err("local model startup, GPU layer or thread setting is out of bounds".into());
    }
    regular_absolute(&spec.executable, "runtime")?;
    regular_absolute(&spec.model_path, "GGUF")?;
    let mut magic = [0; 4];
    std::fs::File::open(&spec.model_path)
        .and_then(|mut file| file.read_exact(&mut magic))
        .map_err(|_| "cannot read selected GGUF header")?;
    if &magic != b"GGUF" {
        return Err("selected model does not have a GGUF header".into());
    }
    Ok(())
}

fn regular_absolute(path: &Path, kind: &str) -> Result<(), String> {
    if !path.is_absolute() || !path.is_file() {
        return Err(format!("selected {kind} must be an existing absolute file"));
    }
    Ok(())
}

async fn drain_logs(
    mut stderr: tokio::process::ChildStderr,
    logs: Arc<Mutex<VecDeque<u8>>>,
) -> Result<(), String> {
    let mut bytes = [0; 4096];
    loop {
        let count = stderr
            .read(&mut bytes)
            .await
            .map_err(|_| "cannot read local model log")?;
        if count == 0 {
            return Ok(());
        }
        let mut tail = logs.lock().expect("model log mutex");
        for byte in &bytes[..count] {
            if tail.len() == MAX_LOG_BYTES {
                tail.pop_front();
            }
            tail.push_back(*byte);
        }
    }
}
