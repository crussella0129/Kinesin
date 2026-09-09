use std::ffi::OsString;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Output;

use kinesin::config::{BoundedConfig, CaptureMode};
use kinesin::core::{ModelReply, ToolCall};
use kinesin::model::{ModelClient, ScriptStep, Usage};
use kinesin::policy::Submission;
use kinesin::runner::{RunResources, admit, run_admitted};
use kinesin::storage::{Command, QueueLimits, Response, RunRecord, Storage, Store};
use serde_json::{Value, json};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

struct Fixture {
    root: PathBuf,
    listener: TcpListener,
}
impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("kinesin-cli-inspect-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace")).unwrap();
        std::fs::create_dir_all(root.join("exports")).unwrap();
        std::fs::create_dir_all(root.join("backups")).unwrap();
        std::fs::write(root.join("workspace/project.txt"), "language=Rust\n").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let source = format!(
            r#"
version = 1
instructions = "private instruction sentinel"
[storage]
path = "state/kinesin.sqlite"
[limits]
max_model_turns = 3
max_run_s = 10
[[workspaces]]
id = "practice"
root = "workspace"
tools = ["read_file"]
[[models]]
id = "local"
base_url = "http://{}"
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
[[owners]]
id = "alice"
workspaces = ["practice"]
models = ["local"]
tasks = ["practice-fields"]
allow_replay = true
"#,
            listener.local_addr().unwrap()
        );
        std::fs::write(root.join("kinesin.toml"), source).unwrap();
        Self { root, listener }
    }
    fn config(&self) -> PathBuf {
        self.root.join("kinesin.toml")
    }
    fn db(&self) -> PathBuf {
        self.root.join("state/kinesin.sqlite")
    }
    fn produce(&self, mode: CaptureMode, owner: Option<&str>) -> RunRecord {
        let config = BoundedConfig::load(&self.config()).unwrap();
        let submission = Submission::Checked {
            task: "practice-fields".into(),
            model: "local".into(),
            limits: None,
            capture: Some(mode),
        };
        let authority = match owner {
            Some(owner) => config.authorize(owner, submission),
            None => config.authorize_local(submission),
        }
        .unwrap();
        let storage = Storage::start(self.db(), QueueLimits::default()).unwrap();
        let store = storage.client();
        let resources = RunResources::single(1, config.concurrency().clone())
            .with_workspace("practice", &self.root.join("workspace"))
            .unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            admit(&authority, &store, None).await.unwrap();
            let client = ModelClient::scripted([
                ModelReply::ToolCalls {
                    content: None,
                    calls: vec![ToolCall {
                        id: "read-1".into(),
                        name: "read_file".into(),
                        arguments: "{\"path\":\"project.txt\"}".into(),
                    }],
                }
                .into(),
                // A checked run answers twice: prose, then the constrained
                // candidate on a turn that carries no tools.
                ModelReply::Answer("I read the file.".into()).into(),
                ModelReply::Answer(
                    json!({"facts":[{"id":"language","value":"Rust","evidence_id":"e0"}]})
                        .to_string(),
                )
                .into(),
            ]);
            let result = run_admitted(
                authority,
                client,
                store,
                resources,
                CancellationToken::new(),
                Instant::now(),
            )
            .await;
            storage.shutdown().await.unwrap();
            result.unwrap()
        })
    }
    /// Produce a completed local checked run whose every model call reports the
    /// same token usage, so the terminal counters sum to a known total.
    fn produce_reporting_usage(&self, per_call: Usage) -> RunRecord {
        let config = BoundedConfig::load(&self.config()).unwrap();
        let submission = Submission::Checked {
            task: "practice-fields".into(),
            model: "local".into(),
            limits: None,
            capture: Some(CaptureMode::Replay),
        };
        let authority = config.authorize_local(submission).unwrap();
        let storage = Storage::start(self.db(), QueueLimits::default()).unwrap();
        let store = storage.client();
        let resources = RunResources::single(1, config.concurrency().clone())
            .with_workspace("practice", &self.root.join("workspace"))
            .unwrap();
        let step = |reply: ModelReply| ScriptStep {
            delay: std::time::Duration::ZERO,
            reply,
            usage: Some(per_call),
        };
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            admit(&authority, &store, None).await.unwrap();
            let client = ModelClient::scripted([
                step(ModelReply::ToolCalls {
                    content: None,
                    calls: vec![ToolCall {
                        id: "read-1".into(),
                        name: "read_file".into(),
                        arguments: "{\"path\":\"project.txt\"}".into(),
                    }],
                }),
                step(ModelReply::Answer("I read the file.".into())),
                step(ModelReply::Answer(
                    json!({"facts":[{"id":"language","value":"Rust","evidence_id":"e0"}]})
                        .to_string(),
                )),
            ]);
            let result = run_admitted(
                authority,
                client,
                store,
                resources,
                CancellationToken::new(),
                Instant::now(),
            )
            .await;
            storage.shutdown().await.unwrap();
            result.unwrap()
        })
    }
    fn cli(&self, args: impl IntoIterator<Item = OsString>) -> Output {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_kinesin"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .unwrap();
        assert!(output.stdout.len() < 65536 && output.stderr.len() < 4096);
        output
    }
    fn command(&self, name: &str) -> Vec<OsString> {
        vec![
            name.into(),
            "--config".into(),
            self.config().into_os_string(),
        ]
    }
    fn no_model_calls(&self) {
        assert!(
            matches!(self.listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
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
                .starts_with("kinesin-cli-inspect-")
        );
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn successful(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn inspect_export_and_pure_replay_preserve_durable_acceptance_without_models() {
    let fixture = Fixture::new();
    let run = fixture.produce(CaptureMode::Replay, None);
    let mut args = fixture.command("inspect");
    args.extend(["--run".into(), run.run_id.clone().into()]);
    let inspected = successful(fixture.cli(args));
    assert_eq!(inspected["kind"], "inspect");
    assert_eq!(inspected["phase"], "completed");
    assert_eq!(inspected["acceptance_status"], "passed");
    assert_eq!(inspected["task_accepted"], true);
    assert_eq!(inspected["receipt"], json!(run.receipt));
    assert_eq!(inspected["events_complete"], true);
    assert!(
        inspected["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event.get("data").is_none())
    );
    assert!(
        !inspected
            .to_string()
            .contains("private instruction sentinel")
    );
    let export = fixture.root.join("exports/run.json");
    let mut args = fixture.command("export");
    args.extend([
        "--run".into(),
        run.run_id.into(),
        "--output".into(),
        export.clone().into_os_string(),
    ]);
    let exported = successful(fixture.cli(args.clone()));
    assert_eq!(exported["capture_present"], true);
    assert!(exported.get("exact_replay_available").is_none());
    let bytes = std::fs::read(&export).unwrap();
    let duplicate = fixture.cli(args);
    assert!(!duplicate.status.success());
    assert_eq!(std::fs::read(&export).unwrap(), bytes);
    // The replay process has neither current configuration nor source/database
    // files available. Its input still binds the historical task and evidence.
    std::fs::rename(
        fixture.root.join("workspace"),
        fixture.root.join("moved-workspace"),
    )
    .unwrap();
    std::fs::rename(fixture.config(), fixture.root.join("old-config.toml")).unwrap();
    std::fs::rename(fixture.root.join("state"), fixture.root.join("old-state")).unwrap();
    let replayed =
        successful(fixture.cli(["replay".into(), "--input".into(), export.into_os_string()]));
    assert_eq!(replayed["kind"], "replayed");
    assert_eq!(replayed["consistency"], "consistent");
    assert_eq!(replayed["acceptance_status"], "passed");
    assert_eq!(replayed["verification_replayed"], true);
    fixture.no_model_calls();
}

#[test]
fn inspect_surfaces_token_totals_when_reported() {
    let fixture = Fixture::new();
    // A checked run makes three model calls; each reports (13, 5).
    let run = fixture.produce_reporting_usage(Usage {
        prompt_tokens: 13,
        completion_tokens: 5,
    });
    let mut args = fixture.command("inspect");
    args.extend(["--run".into(), run.run_id.clone().into()]);
    let inspected = successful(fixture.cli(args));
    assert_eq!(inspected["kind"], "inspect");
    assert_eq!(inspected["phase"], "completed");
    // The acceptance criterion: inspect output shows the summed token totals.
    let counters = &inspected["counters"];
    assert_eq!(counters["prompt_tokens"], 39);
    assert_eq!(counters["completion_tokens"], 15);
    // The base counters remain, and the event summaries still drop bodies.
    assert_eq!(counters["model_turns"], 3);
    assert!(
        inspected["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event.get("data").is_none())
    );
    fixture.no_model_calls();
}

#[test]
fn inspect_omits_token_totals_when_unreported() {
    let fixture = Fixture::new();
    // The default scripted producer reports no usage.
    let run = fixture.produce(CaptureMode::Replay, None);
    let mut args = fixture.command("inspect");
    args.extend(["--run".into(), run.run_id.clone().into()]);
    let inspected = successful(fixture.cli(args));
    let counters = &inspected["counters"];
    // Honest absence: base counters present, token totals omitted (not zero).
    assert_eq!(counters["model_turns"], 3);
    assert!(
        counters.get("prompt_tokens").is_none(),
        "unreported usage stays unknown, not zero"
    );
    assert!(counters.get("completion_tokens").is_none());
    fixture.no_model_calls();
}

#[test]
fn metadata_and_tampered_exports_refuse_exact_replay() {
    let fixture = Fixture::new();
    for mode in [CaptureMode::Metadata, CaptureMode::Replay] {
        let run = fixture.produce(mode, None);
        let output = fixture.root.join(format!("exports/{}.json", run.run_id));
        let mut args = fixture.command("export");
        args.extend([
            "--run".into(),
            run.run_id.into(),
            "--output".into(),
            output.clone().into_os_string(),
        ]);
        successful(fixture.cli(args));
        let mut snapshot: Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
        let expected = if mode == CaptureMode::Metadata {
            assert!(
                snapshot["capture_notice"]
                    .as_str()
                    .unwrap()
                    .contains("unavailable")
            );
            "replay_unavailable_metadata"
        } else {
            snapshot["run"]["result"]["candidate"] = json!("changed answer");
            std::fs::write(&output, serde_json::to_vec(&snapshot).unwrap()).unwrap();
            "replay_candidate_divergence"
        };
        let failed = fixture.cli(["replay".into(), "--input".into(), output.into_os_string()]);
        assert!(!failed.status.success());
        assert!(String::from_utf8_lossy(&failed.stderr).contains(expected));
        assert!(failed.stdout.is_empty());
    }
    fixture.no_model_calls();
}

#[test]
fn owner_scope_and_operator_backups_do_not_overwrite_or_shorten_retention() {
    let fixture = Fixture::new();
    let local = fixture.produce(CaptureMode::Replay, None);
    let alice = fixture.produce(CaptureMode::Replay, Some("alice"));
    let mut args = fixture.command("inspect");
    args.extend(["--run".into(), alice.run_id.into()]);
    let denied = fixture.cli(args);
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("local run was not found"));
    let mut retain = fixture.command("retain");
    retain.extend(["--limit".into(), "1".into()]);
    assert_eq!(successful(fixture.cli(retain))["deleted"], 0);
    let backup = fixture.root.join("backups/snapshot.sqlite");
    let mut args = fixture.command("backup");
    args.extend(["--output".into(), backup.clone().into_os_string()]);
    assert_eq!(successful(fixture.cli(args.clone()))["kind"], "backed_up");
    let bytes = std::fs::read(&backup).unwrap();
    assert!(!fixture.cli(args).status.success());
    assert_eq!(std::fs::read(&backup).unwrap(), bytes);
    let mut restored = Store::open(&backup).unwrap();
    let Response::Run(Some(record)) = restored
        .execute(Command::Get {
            owner_id: "local".into(),
            run_id: local.run_id.clone(),
        })
        .unwrap()
    else {
        panic!("restored local run");
    };
    assert_eq!(*record, local);
    drop(restored);
    let mut forbidden = fixture.command("retain");
    forbidden.extend(["--minimum-hours".into(), "0".into()]);
    assert!(!fixture.cli(forbidden).status.success());
    fixture.no_model_calls();
}

#[test]
fn operator_flag_combinations_are_strict_before_startup() {
    use kinesin::cli::parse;
    for flags in [
        vec!["replay", "--input", "x", "--config", "config"],
        vec!["inspect", "--config", "x", "--run", "r", "--owner", "alice"],
        vec![
            "inspect",
            "--config",
            "x",
            "--run",
            "r",
            "--allow-unchecked",
        ],
        vec!["export", "--config", "x", "--run", "r"],
        vec!["retain", "--config", "x", "--limit", "0"],
        vec!["retain", "--config", "x", "--limit", "101"],
        vec![
            "backup", "--config", "x", "--output", "first", "--output", "second",
        ],
    ] {
        assert!(parse(flags.into_iter().map(OsString::from)).is_err());
    }
}

#[test]
fn single_streamed_run_displays_provisional_text_before_durable_completion() {
    use std::io::{BufRead, Read, Write};
    use std::process::Stdio;
    use std::time::{Duration, Instant as StdInstant};

    let fixture = Fixture::new();
    let config = std::fs::read_to_string(fixture.config())
        .unwrap()
        .replace("temperature = 0.0", "temperature = 0.0\nstream = true");
    std::fs::write(fixture.config(), config).unwrap();
    let listener = fixture.listener.try_clone().unwrap();
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let provider = std::thread::spawn(move || {
        let deadline = StdInstant::now() + Duration::from_secs(10);
        let (mut connection, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(StdInstant::now() < deadline, "provider accept deadline");
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(error) => panic!("provider accept: {error}"),
            }
        };
        connection
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            assert!(header.len() < 16384);
            let mut byte = [0];
            connection.read_exact(&mut byte).unwrap();
            header.push(byte[0]);
        }
        let header = String::from_utf8(header).unwrap();
        let length: usize = header
            .lines()
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;
                key.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().unwrap())
            })
            .unwrap();
        assert!(length <= 131072);
        let mut bytes = vec![0; length];
        connection.read_exact(&mut bytes).unwrap();
        let request: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(request["stream"], true);
        let first = format!(
            "data: {}\n\n",
            json!({"choices":[{"index":0,"delta":{"role":"assistant","content":"early"},"finish_reason":null}]})
        );
        let last = format!(
            "data: {}\n\ndata: [DONE]\n\n",
            json!({"choices":[{"index":0,"delta":{"content":" answer"},"finish_reason":"stop"}]})
        );
        write!(connection,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",first.len()+last.len(),first).unwrap();
        connection.flush().unwrap();
        gate.recv_timeout(Duration::from_secs(5)).unwrap();
        connection.write_all(last.as_bytes()).unwrap();
    });
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = ChildGuard(
        std::process::Command::new(env!("CARGO_BIN_EXE_kinesin"))
            .args([
                OsString::from("run"),
                "--config".into(),
                fixture.config().into_os_string(),
                "--workspace".into(),
                "practice".into(),
                "--model".into(),
                "local".into(),
                "--prompt".into(),
                "Say hello".into(),
                "--allow-unchecked".into(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (line_tx, line_rx) = std::sync::mpsc::sync_channel(4);
    let reader = std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines().take(4) {
            line_tx.send(line.unwrap()).unwrap();
        }
    });
    let first: Value =
        serde_json::from_str(&line_rx.recv_timeout(Duration::from_secs(10)).unwrap()).unwrap();
    assert_eq!(first["kind"], "text_delta");
    assert_eq!(first["text"], "early");
    assert_eq!(first["provisional"], true);
    assert!(first.get("acceptance_status").is_none());
    assert!(child.0.try_wait().unwrap().is_none());
    release.send(()).unwrap();
    let terminal = loop {
        let line: Value =
            serde_json::from_str(&line_rx.recv_timeout(Duration::from_secs(10)).unwrap()).unwrap();
        if line["kind"] == "run" {
            break line;
        }
        assert_eq!(line["provisional"], true);
    };
    assert_eq!(terminal["phase"], "completed");
    assert_eq!(terminal["acceptance_status"], "unchecked");
    assert_eq!(terminal["task_accepted"], false);
    assert_eq!(terminal["result"]["candidate"], "early answer");
    assert!(child.0.wait().unwrap().success());
    reader.join().unwrap();
    provider.join().unwrap();
}
