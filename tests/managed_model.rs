//! Native owned-process proofs with a llama-server-shaped local test fixture.

use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use kinesin::config::{ManagedModelConfig, ModelConfig};
use kinesin::core::{Message, Role};
use kinesin::managed_model::{MAX_LOG_BYTES, ManagedServer};
use kinesin::model::{ModelClient, ModelOptions, prepare};
use serde_json::{Value, json};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

struct Fixture {
    root: PathBuf,
    model: PathBuf,
}

impl Fixture {
    fn new(plan: Value) -> Self {
        let root = std::env::temp_dir().join(format!("kinesin-managed-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let model = root.join("model with spaces.gguf");
        let mut bytes = b"GGUF\x03\0\0\0".to_vec();
        bytes.extend_from_slice(&serde_json::to_vec(&plan).unwrap());
        std::fs::write(&model, bytes).unwrap();
        Self {
            root: root.canonicalize().unwrap(),
            model: model.canonicalize().unwrap(),
        }
    }

    fn spec(&self) -> ManagedModelConfig {
        ManagedModelConfig {
            model: "local".into(),
            executable: PathBuf::from(env!("CARGO_BIN_EXE_cmd-fixture"))
                .canonicalize()
                .unwrap(),
            model_path: self.model.clone(),
            startup_timeout_s: 3,
            gpu_layers: 0,
            threads: 2,
        }
    }

    async fn wait_started(&self) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !self.model.with_extension("pid").exists() {
            assert!(Instant::now() < deadline, "fixture did not start");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    async fn descendant_stopped(&self) {
        let marker = self.model.with_extension("heartbeat");
        assert!(marker.exists(), "test must observe a started descendant");
        let before = std::fs::metadata(&marker).unwrap().len();
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(
            std::fs::metadata(&marker).unwrap().len(),
            before,
            "descendant survived tree cleanup"
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let temporary = std::env::temp_dir().canonicalize().unwrap();
        assert!(self.root.is_absolute() && self.root.starts_with(temporary));
        assert!(
            self.root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("kinesin-managed-")
        );
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn template() -> ModelConfig {
    ModelConfig {
        id: "local".into(),
        base_url: "http://127.0.0.1:8080".into(),
        model_id: "selected-gguf".into(),
        context_size: 4096,
        verified_slots: 1,
        temperature: 0.0,
        stream: false,
        request_timeout_s: 5,
        connect_timeout_s: 1,
        read_timeout_s: 2,
        model_queue_timeout_s: 2,
        cache_prompt: true,
    }
}

fn failure(result: Result<ManagedServer, String>) -> String {
    match result {
        Ok(_) => panic!("managed startup unexpectedly succeeded"),
        Err(error) => error,
    }
}

#[tokio::test]
async fn readiness_gate_uses_unique_loopback_identity_and_preserves_external_listener() {
    let fixture = Fixture::new(json!({"ready_delay_ms":300,"descendant":true}));
    let external = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut profile = template();
    profile.base_url = format!("http://{}", external.local_addr().unwrap());
    let started = Instant::now();
    let mut server = ManagedServer::start(&fixture.spec(), &profile, &CancellationToken::new())
        .await
        .unwrap();
    assert!(started.elapsed() >= Duration::from_millis(300));
    assert_ne!(server.profile().base_url, profile.base_url);
    assert_ne!(server.profile().model_id, profile.model_id);
    let endpoint = url::Url::parse(&server.profile().base_url).unwrap();
    assert_eq!(endpoint.host_str(), Some("127.0.0.1"));
    assert_ne!(endpoint.port(), Some(0));
    assert_eq!(server.profile().id, "local");
    let argv: Vec<String> =
        serde_json::from_slice(&std::fs::read(fixture.model.with_extension("args.json")).unwrap())
            .unwrap();
    assert_eq!(argv[0], "--model");
    assert_eq!(
        PathBuf::from(&argv[1]),
        fixture.model,
        "spaces stay within one model argv"
    );
    for flag in ["--jinja", "--no-context-shift", "--no-webui", "--slots"] {
        assert!(argv.iter().any(|arg| arg == flag));
    }
    let requests = std::fs::read_to_string(fixture.model.with_extension("requests")).unwrap();
    assert!(
        requests.contains("/health")
            && requests.contains("/v1/models")
            && requests.contains("/slots")
    );
    assert!(
        !requests.contains("/v1/chat/completions"),
        "startup never generates a model request"
    );
    let client = ModelClient::http(server.profile(), 65536).unwrap();
    assert!(client.ready(server.profile()).await);
    server.shutdown().await.unwrap();
    server.shutdown().await.unwrap();
    fixture.descendant_stopped().await;
    assert!(
        TcpListener::bind(external.local_addr().unwrap()).is_err(),
        "managed cleanup must not touch the external listener"
    );
}

#[tokio::test]
async fn startup_timeout_rejects_loading_or_wrong_identity_and_cleans_descendants() {
    for mode in ["never-ready", "wrong-model", "wrong-context"] {
        let fixture = Fixture::new(json!({"mode":mode,"descendant":true}));
        let mut spec = fixture.spec();
        spec.startup_timeout_s = 1;
        let started = Instant::now();
        let error =
            failure(ManagedServer::start(&spec, &template(), &CancellationToken::new()).await);
        assert!(error.contains("startup timed out"), "{mode}: {error}");
        assert!(started.elapsed() < Duration::from_secs(5));
        fixture.descendant_stopped().await;
    }
}

#[tokio::test]
async fn cancellation_during_loading_settles_the_owned_tree() {
    let fixture = Fixture::new(json!({"mode":"never-ready","descendant":true}));
    let cancel = CancellationToken::new();
    let spec = fixture.spec();
    let profile = template();
    let loading = ManagedServer::start(&spec, &profile, &cancel);
    let cancel_after_spawn = async {
        fixture.wait_started().await;
        let deadline = Instant::now() + Duration::from_secs(3);
        while !fixture.model.with_extension("heartbeat").exists() {
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        cancel.cancel();
    };
    let (result, ()) = tokio::join!(loading, cancel_after_spawn);
    assert!(failure(result).contains("startup cancelled"));
    fixture.descendant_stopped().await;
}

#[tokio::test]
async fn early_exit_keeps_bounded_terminal_safe_diagnostics_and_cleans_tree() {
    let fixture =
        Fixture::new(json!({"mode":"exit-before-ready","descendant":true,"log_bytes":262144}));
    let error = failure(
        ManagedServer::start(&fixture.spec(), &template(), &CancellationToken::new()).await,
    );
    assert!(error.contains("exited before becoming ready"), "{error}");
    assert!(error.contains("fixture startup failure"));
    assert!(error.contains("\\u{1b}"));
    assert!(!error.contains('\x1b'));
    assert!(
        error.len() < MAX_LOG_BYTES + 1024,
        "stderr must not accumulate without a bound"
    );
    fixture.descendant_stopped().await;
}

#[tokio::test]
async fn leader_death_during_model_request_is_observed_and_descendants_are_reaped() {
    let fixture = Fixture::new(json!({"mode":"die-on-request","descendant":true}));
    let mut server = ManagedServer::start(&fixture.spec(), &template(), &CancellationToken::new())
        .await
        .unwrap();
    let profile = server.profile().clone();
    let client = ModelClient::http(&profile, 65536).unwrap();
    let request = prepare(
        &[Message::text(Role::User, "Hello".into())],
        &ModelOptions {
            origin: profile.base_url.clone(),
            served_model: profile.model_id.clone(),
            temperature: 0.0,
            max_output_tokens: 32,
            max_request_bytes: 65536,
            max_history_bytes: 65536,
            max_response_bytes: 65536,
            stream: false,
            cache_prompt: true,
            tools: vec![],
            structured_actions: false,
            constraint: None,
        },
    )
    .unwrap();
    let (status, response) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(server.wait_for_exit(), client.send(&request))
    })
    .await
    .unwrap();
    assert_eq!(status.unwrap().code(), Some(23));
    assert!(
        response.is_err(),
        "dead model must not produce a successful answer"
    );
    assert!(fixture.model.with_extension("request-seen").exists());
    server.shutdown().await.unwrap();
    fixture.descendant_stopped().await;
}

#[tokio::test]
async fn dropping_a_loading_future_still_terminates_its_process_tree() {
    let fixture = Fixture::new(json!({"mode":"never-ready","descendant":true}));
    let spec = fixture.spec();
    let profile = template();
    let cancel = CancellationToken::new();
    let mut loading = Box::pin(ManagedServer::start(&spec, &profile, &cancel));
    let observed = async {
        fixture.wait_started().await;
        let deadline = Instant::now() + Duration::from_secs(3);
        while !fixture.model.with_extension("heartbeat").exists() {
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    };
    tokio::select! {
        _ = &mut loading => panic!("fixture must remain loading"),
        _ = observed => {},
    }
    drop(loading);
    // Fallback sends termination synchronously; allow the OS to schedule it
    // before independently checking that the descendant has stopped writing.
    tokio::time::sleep(Duration::from_millis(100)).await;
    fixture.descendant_stopped().await;
}

#[tokio::test]
async fn invalid_files_and_precancelled_start_do_not_spawn() {
    let fixture = Fixture::new(json!({}));
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        failure(ManagedServer::start(&fixture.spec(), &template(), &cancel).await)
            .contains("cancelled")
    );
    assert!(!fixture.model.with_extension("pid").exists());
    std::fs::write(&fixture.model, b"not a GGUF").unwrap();
    assert!(
        failure(
            ManagedServer::start(&fixture.spec(), &template(), &CancellationToken::new()).await
        )
        .contains("GGUF header")
    );
    assert!(!fixture.model.with_extension("pid").exists());
}
