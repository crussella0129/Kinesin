//! Local commands share one bounded controller and emit explicit result receipts.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::config::{BoundedConfig, CaptureMode, Config};
use crate::model::ModelClient;
use crate::policy::{LOCAL_OWNER, Submission};
use crate::replay::{self, ReplayReport, Snapshot};
use crate::runner::RunResources;
use crate::scheduler::{Controller, ControllerHandle, Job, PendingRun};
use crate::storage::{Command, Event, QueueLimits, Response, RunRecord, Storage, StorageClient};

mod presentation;
mod session;

pub const USAGE: &str = "Kinesin - bounded local harness

Usage:
  kinesin [--model-path GGUF] [--runtime-path EXECUTABLE]
  kinesin --external
  kinesin --config PATH [--json]
  kinesin [--config PATH] --workspace ALIAS --model ALIAS --prompt TEXT [--allow-unchecked] [--capture MODE]
  kinesin [--config PATH] --task ALIAS --model ALIAS [--capture MODE]
  kinesin batch --config PATH --input PATH
  kinesin inspect --config PATH --run ID
  kinesin export --config PATH --run ID --output NEW_FILE
  kinesin replay --input FILE
  kinesin retain --config PATH [--limit 100]
  kinesin backup --config PATH --output NEW_FILE
  kinesin serve --config PATH
  kinesin provision --config PATH --owner ALIAS --hours HOURS
  kinesin --help | -h

Bare kinesin starts an interactive session with bounded recent conversation memory.
Sessions show readable answers; --json selects JSONL receipts and provisional deltas.
Interactive startup requires a working folder and offers local GGUF models.
Enter uses the selected model; arrow keys choose another. Kinesin starts its local server.
--model-path skips the model chooser; --runtime-path selects llama-server.
--external explicitly connects to a separately managed server.
These setup options require a terminal and cannot combine with --config, --json or run inputs.
Piped/JSON sessions and one-shot runs default to kinesin.toml in the current working directory.
Use --config PATH to select another configuration file. Aliases come from that file.
Capture MODE is metadata or replay; replay capture retains private inputs and outputs.
Help must be used alone and does not load configuration or start a model.";
pub const MAX_BATCH_ENTRIES: usize = 1_024;
pub const MAX_BATCH_LINE_BYTES: usize = 65_536;
pub const MAX_BATCH_BYTES: usize = 16 * 1_048_576;
const MAX_OUTPUT_LINE_BYTES: usize = 8 * 1_048_576;

pub struct RunCommand {
    pub config: PathBuf,
    pub submission: Submission,
    pub allow_unchecked: bool,
}
pub struct BatchCommand {
    pub config: PathBuf,
    pub input: PathBuf,
}
pub struct InspectCommand {
    pub config: PathBuf,
    pub run: String,
}
pub struct ExportCommand {
    pub config: PathBuf,
    pub run: String,
    pub output: PathBuf,
}
pub struct ReplayCommand {
    pub input: PathBuf,
}
pub struct RetainCommand {
    pub config: PathBuf,
    pub limit: usize,
}
pub struct BackupCommand {
    pub config: PathBuf,
    pub output: PathBuf,
}
pub enum CliCommand {
    Help,
    /// No run input (no arguments, or only `--config`): an interactive session.
    /// Each entry continues the one before it, so following up needs no run id
    /// and no flag.
    Session {
        config: PathBuf,
        json: bool,
        explicit_config: bool,
        options: crate::onboarding::SessionOptions,
    },
    Run(RunCommand),
    Batch(BatchCommand),
    Inspect(InspectCommand),
    Export(ExportCommand),
    Replay(ReplayCommand),
    Retain(RetainCommand),
    Backup(BackupCommand),
    Serve {
        config: PathBuf,
    },
    Provision {
        config: PathBuf,
        owner: String,
        hours: u64,
    },
}

pub const DEFAULT_CONFIG: &str = "kinesin.toml";

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<CliCommand, String> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(CliCommand::Session {
            config: PathBuf::from(DEFAULT_CONFIG),
            json: false,
            explicit_config: false,
            options: crate::onboarding::SessionOptions::default(),
        });
    };
    let command = first.into_string().map_err(|_| USAGE)?;
    if command == "--help" || command == "-h" {
        return if args.next().is_none() {
            Ok(CliCommand::Help)
        } else {
            Err("help must be used alone".into())
        };
    }
    // Running is what bare `kinesin` does. A leading flag (or no argument at all)
    // is the default entry: a one-shot run when run inputs are given, or the
    // interactive session when only --config is. Only operations that do
    // something other than run the app are named subcommands.
    if command.starts_with("--") {
        return parse_default(command, args);
    }
    if ![
        "batch",
        "inspect",
        "export",
        "replay",
        "retain",
        "backup",
        "serve",
        "provision",
    ]
    .contains(&command.as_str())
    {
        return Err(USAGE.into());
    }
    let mut values = BTreeMap::new();
    for _ in 0..16 {
        let Some(flag) = args.next() else { break };
        let flag = flag.into_string().map_err(|_| "flag must be UTF-8")?;
        let supported = match command.as_str() {
            "batch" => ["--config", "--input"].contains(&flag.as_str()),
            "inspect" => ["--config", "--run"].contains(&flag.as_str()),
            "export" => ["--config", "--run", "--output"].contains(&flag.as_str()),
            "replay" => flag == "--input",
            "retain" => ["--config", "--limit"].contains(&flag.as_str()),
            "backup" => ["--config", "--output"].contains(&flag.as_str()),
            "serve" => flag == "--config",
            "provision" => ["--config", "--owner", "--hours"].contains(&flag.as_str()),
            _ => false,
        };
        if !supported {
            return Err("unknown argument; see command usage".into());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        if values.insert(flag.clone(), value).is_some() {
            return Err(format!("duplicate {flag}"));
        }
    }
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    if command == "replay" {
        return Ok(CliCommand::Replay(ReplayCommand {
            input: PathBuf::from(values.remove("--input").ok_or("missing --input")?),
        }));
    }
    let config = PathBuf::from(values.remove("--config").ok_or("missing --config")?);
    if command == "serve" {
        return Ok(CliCommand::Serve { config });
    }
    if command == "provision" {
        let owner = take_text(&mut values, "--owner")?;
        crate::config::validate_id(&owner)?;
        let hours = take_text(&mut values, "--hours")?
            .parse::<u64>()
            .map_err(|_| "hours must be an integer from 1 to 8760")?;
        if !(1..=8760).contains(&hours) {
            return Err("hours must be from 1 to 8760".into());
        }
        return Ok(CliCommand::Provision {
            config,
            owner,
            hours,
        });
    }
    if command == "inspect" || command == "export" {
        let run = take_text(&mut values, "--run")?;
        crate::config::validate_id(&run)?;
        return if command == "inspect" {
            Ok(CliCommand::Inspect(InspectCommand { config, run }))
        } else {
            Ok(CliCommand::Export(ExportCommand {
                config,
                run,
                output: PathBuf::from(values.remove("--output").ok_or("missing --output")?),
            }))
        };
    }
    if command == "retain" {
        let limit = values
            .remove("--limit")
            .map(|value| {
                value
                    .into_string()
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .ok_or("limit must be an integer from 1 to 100")
            })
            .transpose()?
            .unwrap_or(100);
        if !(1..=100).contains(&limit) {
            return Err("limit must be an integer from 1 to 100".into());
        }
        return Ok(CliCommand::Retain(RetainCommand { config, limit }));
    }
    if command == "backup" {
        return Ok(CliCommand::Backup(BackupCommand {
            config,
            output: PathBuf::from(values.remove("--output").ok_or("missing --output")?),
        }));
    }
    if command == "batch" {
        return Ok(CliCommand::Batch(BatchCommand {
            config,
            input: PathBuf::from(values.remove("--input").ok_or("missing --input")?),
        }));
    }
    // Every operator verb returns above; the whitelist guarantees no other reaches here.
    Err(USAGE.into())
}

/// The default entry when the first argument is a flag (or absent): a one-shot
/// run when run inputs are present, otherwise the interactive session. There is
/// no `run` verb — running is what bare `kinesin` does.
fn parse_default(
    first_flag: String,
    rest: impl Iterator<Item = OsString>,
) -> Result<CliCommand, String> {
    let mut values = BTreeMap::new();
    let mut allow_unchecked = false;
    let mut json = false;
    let mut external = false;
    let mut flags = std::iter::once(OsString::from(first_flag)).chain(rest);
    for _ in 0..16 {
        let Some(flag) = flags.next() else { break };
        let flag = flag.into_string().map_err(|_| "flag must be UTF-8")?;
        if flag == "--external" {
            if external {
                return Err("duplicate --external".into());
            }
            external = true;
            continue;
        }
        if flag == "--json" {
            if json {
                return Err("duplicate --json".into());
            }
            json = true;
            continue;
        }
        if flag == "--allow-unchecked" {
            if allow_unchecked {
                return Err("duplicate --allow-unchecked".into());
            }
            allow_unchecked = true;
            continue;
        }
        if ![
            "--config",
            "--workspace",
            "--model",
            "--prompt",
            "--task",
            "--capture",
            "--model-path",
            "--runtime-path",
        ]
        .contains(&flag.as_str())
        {
            return Err(USAGE.into());
        }
        let value = flags
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        if values.insert(flag.clone(), value).is_some() {
            return Err(format!("duplicate {flag}"));
        }
    }
    if flags.next().is_some() {
        return Err("too many arguments".into());
    }
    let explicit_config = values.contains_key("--config");
    let options = crate::onboarding::SessionOptions {
        model_path: values.remove("--model-path").map(PathBuf::from),
        runtime_path: values.remove("--runtime-path").map(PathBuf::from),
        external,
    };
    let setup_options = options.model_path.is_some() || options.runtime_path.is_some() || external;
    if external && (options.model_path.is_some() || options.runtime_path.is_some()) {
        return Err("--external cannot combine with local model/runtime paths".into());
    }
    let config = values
        .remove("--config")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG));
    // With no run input, this is the interactive session. --model or --capture
    // alone is neither a session nor a complete run.
    let has_run_input = allow_unchecked
        || values.contains_key("--workspace")
        || values.contains_key("--prompt")
        || values.contains_key("--task");
    if setup_options && (explicit_config || json || has_run_input) {
        return Err("model setup options require an interactive session without --config, --json or run inputs".into());
    }
    if !has_run_input {
        if !values.is_empty() {
            return Err(USAGE.into());
        }
        return Ok(CliCommand::Session {
            config,
            json,
            explicit_config,
            options,
        });
    }
    if json {
        return Err("--json is a session option; one-shot runs already emit JSON".into());
    }
    let model = take_text(&mut values, "--model")?;
    let capture = values
        .remove("--capture")
        .map(|value| match value.to_str() {
            Some("metadata") => Ok(CaptureMode::Metadata),
            Some("replay") => Ok(CaptureMode::Replay),
            _ => Err("capture must be metadata or replay"),
        })
        .transpose()?;
    let submission = if let Some(task) = values.remove("--task") {
        if allow_unchecked || values.contains_key("--prompt") || values.contains_key("--workspace")
        {
            return Err(
                "checked task forbids prompt, workspace override and --allow-unchecked".into(),
            );
        }
        Submission::Checked {
            task: task.into_string().map_err(|_| "task must be UTF-8")?,
            model,
            limits: None,
            capture,
        }
    } else {
        Submission::Freeform {
            workspace: take_text(&mut values, "--workspace")?,
            model,
            prompt: take_text(&mut values, "--prompt")?,
            continues: None,
            limits: None,
            capture,
        }
    };
    if !values.is_empty() {
        return Err("unsupported argument combination".into());
    }
    Ok(CliCommand::Run(RunCommand {
        config,
        submission,
        allow_unchecked,
    }))
}

fn take_text(values: &mut BTreeMap<String, OsString>, key: &str) -> Result<String, String> {
    values
        .remove(key)
        .ok_or_else(|| format!("missing {key}"))?
        .into_string()
        .map_err(|_| format!("{key} must be UTF-8"))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchItem {
    pub submission: Submission,
    #[serde(default)]
    pub allow_unchecked: bool,
}
impl BatchItem {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let item: Self =
            serde_json::from_slice(bytes).map_err(|_| "invalid batch JSON or fields")?;
        if item.allow_unchecked && matches!(item.submission, Submission::Checked { .. }) {
            return Err("checked batch rows forbid allow_unchecked".into());
        }
        Ok(item)
    }
}

type ParsedBatchLine = (usize, Result<BatchItem, String>);
struct BatchReader {
    input: BufReader<File>,
    total_bytes: usize,
    entries: usize,
}
impl BatchReader {
    fn open(path: &PathBuf) -> Result<Self, String> {
        let input = File::open(path).map_err(|_| "cannot open batch input")?;
        let metadata = input.metadata().map_err(|_| "cannot inspect batch input")?;
        if !metadata.is_file() || metadata.len() > MAX_BATCH_BYTES as u64 {
            return Err("batch input must be a regular file of at most 16 MiB".into());
        }
        Ok(Self {
            input: BufReader::with_capacity(8_192, input),
            total_bytes: 0,
            entries: 0,
        })
    }

    fn next(&mut self) -> Result<Option<ParsedBatchLine>, String> {
        let mut line = Vec::new();
        loop {
            let available = self
                .input
                .fill_buf()
                .map_err(|_| "cannot read batch input")?;
            if available.is_empty() {
                break;
            }
            if self.entries >= MAX_BATCH_ENTRIES {
                return Err("batch exceeds 1024 entries".into());
            }
            let consumed = available
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(available.len(), |offset| offset + 1);
            if line.len() + consumed > MAX_BATCH_LINE_BYTES {
                return Err("batch line exceeds 64 KiB".into());
            }
            if self.total_bytes + consumed > MAX_BATCH_BYTES {
                return Err("batch exceeds 16 MiB".into());
            }
            let complete = available[consumed - 1] == b'\n';
            line.extend_from_slice(&available[..consumed]);
            self.input.consume(consumed);
            self.total_bytes += consumed;
            if complete {
                break;
            }
        }
        if line.is_empty() {
            return Ok(None);
        }
        self.entries += 1;
        Ok(Some((self.entries, BatchItem::parse(&line))))
    }
}

struct Startup {
    config: Config,
    models: BTreeMap<String, ModelClient>,
    resources: BTreeMap<String, RunResources>,
}
impl Startup {
    /// A session takes no aliases. With one workspace and one model there is
    /// nothing to choose; with several the operator must say which, because
    /// guessing would silently pick an authority.
    fn session_defaults(&self) -> Result<(String, String), String> {
        let workspaces = self.config.workspaces();
        let models = self.config.models();
        let workspace = match workspaces {
            [only] => only.id.clone(),
            _ => return Err("a session needs exactly one configured workspace".into()),
        };
        let model = match models {
            [only] => only.id.clone(),
            _ => return Err("a session needs exactly one configured model".into()),
        };
        Ok((workspace, model))
    }

    fn job(&self, submission: Submission) -> Result<Job, String> {
        self.job_continued(submission, None)
    }

    fn job_continued(
        &self,
        submission: Submission,
        prior: Option<crate::policy::PriorAnswer>,
    ) -> Result<Job, String> {
        let authority = self.config.authorize_local_continued(submission, prior)?;
        self.job_authorized(authority)
    }

    fn job_authorized(&self, authority: crate::policy::RunAuthority) -> Result<Job, String> {
        let model = &authority.model().id;
        Ok(Job {
            display: None,
            client: self
                .models
                .get(model)
                .ok_or("unknown prepared model")?
                .clone(),
            resources: self
                .resources
                .get(model)
                .ok_or("unknown prepared resources")?
                .clone(),
            authority,
        }
        .discover_mcp(&self.config))
    }

    fn session_job(
        &self,
        submission: Submission,
        memory: &session::Memory,
    ) -> Result<(Job, usize), String> {
        // Check the new request on its own first. An invalid request must not
        // evict useful history, nor may trimming change its grants or limits.
        let baseline = self.config.authorize_local(submission.clone())?;
        let prepare = |authority: &crate::policy::RunAuthority| -> Result<(), String> {
            let context = authority
                .session_context()
                .map(|value| value.encode())
                .transpose()?;
            let (state, _) = crate::core::initiate_with_context(
                authority.instructions().to_owned(),
                None,
                context,
                authority.prompt().to_owned(),
                !authority.workspace().tools.is_empty(),
            )
            .map_err(|_| "cannot initialize session conversation")?;
            crate::model::prepare(state.messages(), &crate::runner::options(authority))?;
            Ok(())
        };
        prepare(&baseline)?;
        let Some(mut context) = memory.context() else {
            return Ok((self.job_authorized(baseline)?, 0));
        };
        loop {
            let count = context.turns.len();
            match self
                .config
                .authorize_local_session(submission.clone(), Some(context.clone()))
            {
                Ok(authority) if prepare(&authority).is_ok() => {
                    return Ok((self.job_authorized(authority)?, count));
                }
                Ok(_) => {}
                Err(error)
                    if error == "initial conversation exceeds effective history/request budget" => {
                }
                Err(error) => return Err(error),
            }
            context.turns.remove(0);
            if context.turns.is_empty() {
                return Ok((self.job_authorized(baseline)?, 0));
            }
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum OutputLine {
    TextDelta {
        run_id: String,
        provisional: bool,
        text: String,
    },
    DisplayClosed {
        run_id: String,
        reason: String,
    },
    Run {
        index: usize,
        task_accepted: bool,
        exit_code: u8,
        #[serde(flatten)]
        record: Box<RunRecord>,
    },
    Error {
        index: Option<usize>,
        reason: String,
    },
    Inspect {
        task_accepted: bool,
        #[serde(flatten)]
        record: Box<RunRecord>,
        events: Vec<EventSummary>,
        events_complete: bool,
        /// The terminal `run_finished` counters (model turns, tool calls, and
        /// token totals when any call reported usage). The event summaries drop
        /// event bodies, so these are lifted out for the reader.
        #[serde(skip_serializing_if = "Option::is_none")]
        counters: Option<serde_json::Value>,
    },
    Exported {
        run_id: String,
        capture: String,
        capture_present: bool,
        output: PathBuf,
        event_count: usize,
    },
    Replayed {
        #[serde(flatten)]
        report: ReplayReport,
    },
    Retained {
        deleted: usize,
    },
    BackedUp {
        output: PathBuf,
    },
}

#[derive(Serialize)]
struct EventSummary {
    seq: u64,
    kind: String,
    elapsed_ms: u64,
}
fn result_line(index: usize, result: RunRecord, allow_unchecked: bool) -> (OutputLine, u8) {
    let code = exit_code(&result.phase, &result.acceptance_status, allow_unchecked);
    (
        OutputLine::Run {
            index,
            task_accepted: result.task_accepted(),
            exit_code: code,
            record: Box::new(result),
        },
        code,
    )
}

fn output_bytes(line: &OutputLine) -> Result<Vec<u8>, String> {
    let encoded = serde_json::to_string(line).map_err(|_| "cannot encode CLI result")?;
    // Preserve the decoded JSON value while avoiding terminal C1 controls.
    // Serde already escapes ASCII ESC, carriage return and other C0 controls.
    let mut output = String::with_capacity(encoded.len().min(MAX_OUTPUT_LINE_BYTES));
    for character in encoded.chars() {
        if character.is_control() {
            use std::fmt::Write as _;
            write!(&mut output, "\\u{:04x}", u32::from(character))
                .map_err(|_| "cannot encode CLI result")?;
        } else {
            output.push(character);
        }
        if output.len() >= MAX_OUTPUT_LINE_BYTES {
            return Err("CLI result exceeds output limit".into());
        }
    }
    output.push('\n');
    Ok(output.into_bytes())
}
async fn emit(line: OutputLine) -> Result<(), String> {
    // Exactly one output job is awaited at a time. Slow stdout cannot retain an
    // unbounded queue or block a Tokio worker. A started OS write is not abortable.
    tokio::task::spawn_blocking(move || {
        let bytes = output_bytes(&line)?;
        let stdout = std::io::stdout();
        let mut output = stdout.lock();
        output
            .write_all(&bytes)
            .and_then(|()| output.flush())
            .map_err(|_| "CLI output failed".to_owned())
    })
    .await
    .map_err(|_| "CLI output worker failed")?
}

pub fn execute(command: CliCommand) -> Result<u8, String> {
    if matches!(&command, CliCommand::Help) {
        let mut output = std::io::stdout().lock();
        writeln!(output, "{USAGE}")
            .and_then(|()| output.flush())
            .map_err(|_| "CLI output failed".to_owned())?;
        return Ok(0);
    }
    if let CliCommand::Serve { config } = &command {
        return crate::operator::serve(config);
    }
    if let CliCommand::Provision {
        config,
        owner,
        hours,
    } = &command
    {
        return crate::operator::provision(config, owner, *hours);
    }
    // Replay reads only its supplied snapshot. It does not load current config,
    // open the journal/workspace, construct a model client, or start Tokio.
    if let CliCommand::Replay(command) = command {
        return execute_replay(command);
    }
    // Startup filesystem work and writer readiness occur before entering Tokio.
    let path = match &command {
        CliCommand::Session { config, .. } => config,
        CliCommand::Run(command) => &command.config,
        CliCommand::Batch(command) => &command.config,
        CliCommand::Inspect(command) => &command.config,
        CliCommand::Export(command) => &command.config,
        CliCommand::Retain(command) => &command.config,
        CliCommand::Backup(command) => &command.config,
        CliCommand::Help
        | CliCommand::Replay(_)
        | CliCommand::Serve { .. }
        | CliCommand::Provision { .. } => {
            unreachable!("command handled before startup")
        }
    };
    let mut config = if let CliCommand::Session {
        explicit_config,
        json,
        options,
        ..
    } = &command
    {
        match crate::onboarding::load_session_with_options(path, *explicit_config, *json, options) {
            Ok(config) => config,
            Err(reason) if reason == crate::onboarding::SETUP_CANCELLED => return Ok(130),
            Err(reason) => return Err(reason),
        }
    } else {
        BoundedConfig::load(path)?
    };
    if !matches!(
        command,
        CliCommand::Session { .. } | CliCommand::Run(_) | CliCommand::Batch(_)
    ) {
        return execute_operator(command, config);
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| "runtime initialization failed")?;
    runtime.block_on(async move {
        let interrupted = CancellationToken::new();
        let done = CancellationToken::new();
        let mut listener = crate::signal::ctrl_c_listener()
            .map_err(|_| "cannot register cancellation before model startup")?;
        let signal = {
            let (interrupt, finished) = (interrupted.clone(), done.clone());
            tokio::spawn(async move {
                tokio::select! {
                    _ = finished.cancelled() => Ok(()),
                    result = listener.recv() => {
                        interrupt.cancel();
                        result.map_err(|_| "signal handler failed".to_owned())
                    }
                }
            })
        };
        let mut server = None;
        let outcome = async {
            if let Some(spec) = config.managed_model() {
                if matches!(&command, CliCommand::Session { json: false, .. }) {
                    eprintln!("Loading local model... Ctrl+C cancels startup.");
                }
                let owned = crate::managed_model::ManagedServer::start(
                    spec, config.model(&spec.model).ok_or("managed profile missing")?, &interrupted,
                ).await?;
                // Keep ownership even if subsequent preparation fails.
                server = Some(owned);
                config.bind_managed_model(server.as_ref().expect("owned server").profile())?;
            }
            let running = execute_runs(command, config, &interrupted);
            tokio::pin!(running);
            if let Some(owned) = &mut server {
                tokio::select! {
                    outcome = &mut running => outcome,
                    exited = owned.wait_for_exit() => {
                        interrupted.cancel();
                        let _ = running.await;
                        match exited {
                            Ok(status) => Err(format!("local model server exited ({status}); session stopped")),
                            Err(reason) => Err(reason),
                        }
                    }
                }
            } else {
                running.await
            }
        }.await;
        let cleanup = match &mut server {
            Some(owned) => owned.shutdown().await,
            None => Ok(()),
        };
        done.cancel();
        let signaled = signal.await.map_err(|_| "signal task failed")?;
        cleanup?;
        signaled?;
        // Backend death keeps its diagnostic; user interruption is exit 130.
        if interrupted.is_cancelled() && outcome.is_ok() {
            return Ok(130);
        }
        if interrupted.is_cancelled() && outcome.as_ref().is_err_and(|reason| reason.starts_with("local model startup cancelled")) {
            return Ok(130);
        }
        outcome
    })
}

async fn execute_runs(
    command: CliCommand,
    config: Config,
    interrupted: &CancellationToken,
) -> Result<u8, String> {
    if interrupted.is_cancelled() {
        return Ok(130);
    }
    let resources = RunResources::from_config(&config)?;
    let models = config
        .models()
        .iter()
        .map(|model| {
            Ok((
                model.id.clone(),
                ModelClient::http(model, config.limits().max_response_bytes)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let startup = Startup {
        config,
        models,
        resources,
    };
    // A continuation reads a prior run, so its job is built inside the runtime
    // where storage exists rather than before it opens.
    let (session, single, batch) = match command {
        CliCommand::Session { json, .. } => (Some(json), None, None),
        CliCommand::Run(command) => (
            None,
            Some((command.submission, command.allow_unchecked)),
            None,
        ),
        CliCommand::Batch(command) => (None, None, Some(BatchReader::open(&command.input)?)),
        _ => unreachable!("operator commands handled before model startup"),
    };
    let limits = startup.config.concurrency().clone();
    let storage = Storage::start_with_policy(
        startup.config.storage().path.clone(),
        QueueLimits {
            commands: limits.journal_queue_events,
            bytes: limits.journal_queue_bytes,
            max_waiters: limits
                .max_active_runs
                .checked_add(16)
                .ok_or("invalid storage waiter limit")?,
        },
        startup.config.storage().policy.clone(),
    )
    .map_err(|error| error.to_string())?;
    let (handle, controller) = match Controller::start(limits, storage.client(), false) {
        Ok(controller) => controller,
        Err(reason) => {
            storage
                .shutdown()
                .await
                .map_err(|error| error.to_string())?;
            return Err(reason);
        }
    };
    let done = CancellationToken::new();
    let cancellation = {
        let (signal_handle, signal_interrupt, signal_done) =
            (handle.clone(), interrupted.clone(), done.clone());
        tokio::spawn(async move {
            tokio::select! {
                _ = signal_done.cancelled() => {},
                _ = signal_interrupt.cancelled() => {
                    signal_handle.shutdown();
                }
            }
        })
    };
    let outcome = async {
        if let Some(json) = session {
            run_session(&handle, &startup, &storage.client(), interrupted, json).await
        } else {
            match (single, batch) {
                (Some((mut submission, allow_unchecked)), None) => {
                    let prior = resolve_continuation(&storage.client(), &mut submission).await?;
                    let job = startup.job_continued(submission, prior)?;
                    run_single(&handle, job, allow_unchecked, None).await
                }
                (None, Some(reader)) => run_batch(&handle, &startup, reader, interrupted).await,
                _ => Err("invalid prepared command".into()),
            }
        }
    }
    .await;
    // This also handles input/output failure without detaching live effects.
    handle.shutdown();
    let joined = controller.join().await;
    let mut preview_result = Ok(());
    for resources in startup.resources.values() {
        if let Err(reason) = resources.shutdown_previews().await {
            preview_result = Err(reason);
        }
    }
    done.cancel();
    let cancellation_result = cancellation
        .await
        .map_err(|_| "cancellation task failed".to_owned());
    let storage_result = storage.shutdown().await.map_err(|error| error.to_string());
    joined?;
    preview_result?;
    cancellation_result?;
    storage_result?;
    outcome
}

fn emit_sync(line: OutputLine) -> Result<(), String> {
    let bytes = output_bytes(&line)?;
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    output
        .write_all(&bytes)
        .and_then(|()| output.flush())
        .map_err(|_| "CLI output failed".into())
}

fn execute_replay(command: ReplayCommand) -> Result<u8, String> {
    let file = File::open(command.input).map_err(|_| "cannot open replay input")?;
    let metadata = file.metadata().map_err(|_| "cannot inspect replay input")?;
    if !metadata.is_file() || metadata.len() > replay::MAX_REPLAY_BYTES as u64 {
        return Err("replay input must be a regular file of at most 32 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take((replay::MAX_REPLAY_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read replay input")?;
    let snapshot = replay::decode_snapshot(&bytes).map_err(|e| e.to_string())?;
    let report = replay::replay(&snapshot.run, &snapshot.events).map_err(|e| e.to_string())?;
    emit_sync(OutputLine::Replayed { report })?;
    Ok(0)
}

fn execute_operator(command: CliCommand, config: Config) -> Result<u8, String> {
    let concurrency = config.concurrency();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|_| "runtime initialization failed")?;
    // An operator command owns the same controller lock and dedicated SQLite
    // thread. It cannot run concurrently with another local controller.
    let storage = Storage::start_with_policy(
        config.storage().path.clone(),
        QueueLimits {
            commands: concurrency.journal_queue_events,
            bytes: concurrency.journal_queue_bytes,
            max_waiters: 1,
        },
        config.storage().policy.clone(),
    )
    .map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let result = operator_action(command, &storage.client()).await;
        let shutdown = storage.shutdown().await.map_err(|e| e.to_string());
        shutdown?;
        let line = result?;
        emit(line).await?;
        Ok(0)
    })
}

async fn query(store: &StorageClient, command: Command) -> Result<Response, String> {
    store
        .execute(
            command,
            tokio::time::Instant::now() + Duration::from_secs(5),
        )
        .await
        .map_err(|e| e.to_string())
}
/// Read the cited run under this owner's scope, and accept only a finished one.
/// A run in flight has no recorded answer to quote, and citing one would depend
/// on when the read happened.
async fn resolve_continuation(
    store: &StorageClient,
    submission: &mut Submission,
) -> Result<Option<crate::policy::PriorAnswer>, String> {
    let Submission::Freeform {
        workspace,
        model,
        continues: Some(run_id),
        ..
    } = submission
    else {
        return Ok(None);
    };
    let run_id = run_id.clone();
    let record = retained_run(store, run_id.clone()).await?;
    if !matches!(
        record.phase.as_str(),
        "completed" | "stopped" | "failed" | "cancelled" | "interrupted"
    ) {
        return Err("cited run has not finished".into());
    }
    let answer = record
        .result
        .as_ref()
        .and_then(|result| result["candidate"].as_str())
        .ok_or("cited run retained no answer to continue from")?;
    // Authorization still checks these aliases against the owner's permissions:
    // inheriting them repeats an earlier decision, it does not bypass one.
    workspace.clone_from(&record.workspace_id);
    model.clone_from(&record.model_profile_id);
    Ok(Some(crate::policy::PriorAnswer {
        run_id,
        answer: answer.to_owned(),
    }))
}

async fn retained_run(store: &StorageClient, run: String) -> Result<RunRecord, String> {
    match query(
        store,
        Command::Get {
            owner_id: LOCAL_OWNER.into(),
            run_id: run,
        },
    )
    .await?
    {
        Response::Run(Some(record)) => Ok(*record),
        Response::Run(None) => Err("local run was not found".into()),
        _ => Err("unexpected run query acknowledgement".into()),
    }
}

async fn event_page(
    store: &StorageClient,
    run: &str,
    after: Option<u64>,
    limit: usize,
) -> Result<Vec<Event>, String> {
    match query(
        store,
        Command::Events {
            owner_id: LOCAL_OWNER.into(),
            run_id: run.into(),
            after,
            limit,
        },
    )
    .await?
    {
        Response::Events(events) => Ok(events),
        _ => Err("unexpected event query acknowledgement".into()),
    }
}

async fn operator_action(command: CliCommand, store: &StorageClient) -> Result<OutputLine, String> {
    match command {
        CliCommand::Inspect(command) => {
            let run = retained_run(store, command.run).await?;
            let mut summaries = Vec::new();
            let mut counters = None;
            let mut after = None;
            let complete = loop {
                let page = event_page(store, &run.run_id, after, 100).await?;
                if page.is_empty() {
                    break true;
                }
                for event in page {
                    if summaries.len() == replay::MAX_REPLAY_EVENTS {
                        break;
                    }
                    after = Some(event.seq);
                    // The terminal event carries the run's counters; lift them
                    // out before the summary drops the event body.
                    if event.kind == "run_finished" {
                        counters = event.data.get("counters").cloned();
                    }
                    summaries.push(EventSummary {
                        seq: event.seq,
                        kind: event.kind,
                        elapsed_ms: event.elapsed_ms,
                    });
                }
                if summaries.len() == replay::MAX_REPLAY_EVENTS {
                    break event_page(store, &run.run_id, after, 1).await?.is_empty();
                }
            };
            Ok(OutputLine::Inspect {
                task_accepted: run.task_accepted(),
                record: Box::new(run),
                events: summaries,
                events_complete: complete,
                counters,
            })
        }
        CliCommand::Export(command) => {
            let run = retained_run(store, command.run).await?;
            let capture_present = run.capture == "replay";
            let notice = if capture_present {
                "Private replay inputs are present. The replay command must validate compatibility, completeness and consistency; capture alone does not establish replay availability or authenticity."
            } else {
                "Metadata capture: exact decision replay and acceptance recomputation are unavailable because intermediate inputs and evidence bodies were not retained."
            };
            let mut snapshot = Snapshot {
                schema_version: 2,
                capture_notice: Some(notice.into()),
                run,
                events: Vec::new(),
            };
            let mut counted = encoded_size(&snapshot, replay::MAX_REPLAY_BYTES)?;
            let mut after = None;
            loop {
                let page = event_page(store, &snapshot.run.run_id, after, 100).await?;
                if page.is_empty() {
                    break;
                }
                if page.len() > replay::MAX_REPLAY_EVENTS - snapshot.events.len() {
                    return Err("export exceeds 256 events".into());
                }
                for event in page {
                    let bytes = encoded_size(&event, replay::MAX_REPLAY_BYTES)?;
                    counted = counted
                        .checked_add(bytes + 1)
                        .filter(|count| *count <= replay::MAX_REPLAY_BYTES)
                        .ok_or("export exceeds 32 MiB")?;
                    after = Some(event.seq);
                    snapshot.events.push(event);
                }
            }
            // All pages are bounded before retaining them; serialization writes
            // directly to the new file without a second full JSON byte buffer.
            encoded_size(&snapshot, replay::MAX_REPLAY_BYTES)?;
            let summary = OutputLine::Exported {
                run_id: snapshot.run.run_id.clone(),
                capture: snapshot.run.capture.clone(),
                capture_present,
                output: command.output.clone(),
                event_count: snapshot.events.len(),
            };
            tokio::task::spawn_blocking(move || write_snapshot_new(&command.output, &snapshot))
                .await
                .map_err(|_| "export worker failed")??;
            Ok(summary)
        }
        CliCommand::Retain(command) => {
            let now = i64::try_from(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| "invalid system clock")?
                    .as_millis(),
            )
            .map_err(|_| "system clock overflow")?;
            match query(
                store,
                Command::Retain {
                    now_unix_ms: now,
                    limit: command.limit,
                },
            )
            .await?
            {
                Response::Retained(deleted) => Ok(OutputLine::Retained { deleted }),
                _ => Err("unexpected retention acknowledgement".into()),
            }
        }
        CliCommand::Backup(command) => {
            let output =
                std::path::absolute(command.output).map_err(|_| "invalid backup destination")?;
            match query(
                store,
                Command::Backup {
                    destination: output.clone(),
                },
            )
            .await?
            {
                Response::Updated => Ok(OutputLine::BackedUp { output }),
                _ => Err("unexpected backup acknowledgement".into()),
            }
        }
        _ => Err("invalid operator command".into()),
    }
}

struct LimitedWriter<W> {
    inner: W,
    bytes: usize,
    limit: usize,
}
impl<W: Write> Write for LimitedWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit - self.bytes {
            return Err(std::io::Error::other("output exceeds limit"));
        }
        let count = self.inner.write(bytes)?;
        self.bytes += count;
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
fn encoded_size(value: &impl Serialize, limit: usize) -> Result<usize, String> {
    let mut writer = LimitedWriter {
        inner: std::io::sink(),
        bytes: 0,
        limit,
    };
    serde_json::to_writer(&mut writer, value).map_err(|_| "serialized output exceeds limit")?;
    Ok(writer.bytes)
}
fn write_snapshot_new(path: &PathBuf, snapshot: &Snapshot) -> Result<(), String> {
    // Never truncate a destination selected by the operator. Existing ACLs are
    // left alone; new Unix files are owner-only and Windows inherits the
    // descriptor of the operator-selected private folder.
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path).map_err(
        |_| "cannot create export; destination must be a new file in an existing folder",
    )?;
    let mut writer = LimitedWriter {
        inner: std::io::BufWriter::new(file),
        bytes: 0,
        limit: replay::MAX_REPLAY_BYTES,
    };
    serde_json::to_writer(&mut writer, snapshot)
        .map_err(|_| "export write failed; destination may be incomplete")?;
    writer
        .flush()
        .and_then(|()| writer.inner.get_ref().sync_all())
        .map_err(|_| "export write failed; destination may be incomplete".to_owned())
}

async fn run_single(
    handle: &ControllerHandle,
    mut job: Job,
    allow_unchecked: bool,
    human_store: Option<&StorageClient>,
) -> Result<u8, String> {
    let json = human_store.is_none();
    let mut human = presentation::HumanDisplay::default();
    let run_id = job.authority.run_id().to_owned();
    let mut display = if job.authority.model().stream {
        let (observer, receiver) = crate::model::TextObserver::bounded();
        job.display = Some(observer);
        Some(receiver)
    } else {
        None
    };
    let mut pending = handle.try_submit(job, None)?;
    pending.admitted().await?;
    if !json {
        emit_human("Working...\n".into()).await?;
    }
    let mut activity = presentation::Activity::default();
    let mut activity_enabled = human_store.is_some();
    let mut activity_tick = tokio::time::interval(Duration::from_millis(200));
    activity_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let finished = pending.finished();
    tokio::pin!(finished);
    let record = loop {
        tokio::select! {
            biased;
            result=&mut finished=>break result?,
            _=activity_tick.tick(),if activity_enabled=>{
                activity_enabled = show_activity(human_store.expect("guarded human store"), &run_id, &mut activity, &mut human).await?;
            }
            frame=async{display.as_mut().expect("guarded display").recv().await},if display.is_some()=>{
                match frame {
                    Some(text)=>{
                        if json {
                            emit(OutputLine::TextDelta{run_id:run_id.clone(),provisional:true,text}).await?;
                        } else {
                            emit_human(human.delta(&text)).await?;
                        }
                    },
                    None=>{
                        if display.as_ref().is_some_and(|receiver|receiver.is_lagged()) {
                            if json {
                                emit(OutputLine::DisplayClosed{run_id:run_id.clone(),reason:"provisional_display_lagged".into()}).await?;
                            } else {
                                emit_human("\n[Live display paused; waiting for the saved result.]\n".into()).await?;
                                human.break_stream();
                            }
                        }
                        display=None;
                    }
                }
            }
        }
    };
    if let Some(store) = human_store {
        // Finish the bounded activity page after settlement, including tools
        // too quick to be observed by a live tick, before printing the answer.
        while activity_enabled && !activity.full() {
            let before = activity.after();
            activity_enabled = show_activity(store, &run_id, &mut activity, &mut human).await?;
            if activity.after() == before {
                break;
            }
        }
    }
    let code = exit_code(&record.phase, &record.acceptance_status, allow_unchecked);
    if json {
        let (line, _) = result_line(1, record, allow_unchecked);
        emit(line).await?;
    } else {
        emit_human(human.finished(&record)).await?;
    }
    Ok(code)
}

async fn show_activity(
    store: &StorageClient,
    run_id: &str,
    activity: &mut presentation::Activity,
    human: &mut presentation::HumanDisplay,
) -> Result<bool, String> {
    if activity.full() {
        return Ok(false);
    }
    let page = tokio::time::timeout(
        Duration::from_secs(1),
        event_page(store, run_id, activity.after(), 64),
    )
    .await;
    let Ok(Ok(events)) = page else {
        emit_human(human.activity("[Live activity unavailable; waiting for the saved result.]\n"))
            .await?;
        return Ok(false);
    };
    let text = activity.observe(events);
    if !text.is_empty() {
        emit_human(human.activity(&text)).await?;
    }
    Ok(!activity.full())
}

struct Completion {
    index: usize,
    allow_unchecked: bool,
    outcome: Result<RunRecord, String>,
}
fn follow(
    pending: PendingRun,
    index: usize,
    allow_unchecked: bool,
    pending_results: &mut JoinSet<Completion>,
) {
    // A completion waiter exists only after the controller accepted this run.
    pending_results.spawn(async move {
        Completion {
            index,
            allow_unchecked,
            outcome: pending.finished().await,
        }
    });
}
async fn complete_one(
    pending: &mut JoinSet<Completion>,
    aggregate: &mut u8,
    output_failed: &mut Option<String>,
) {
    let Some(completed) = pending.join_next().await else {
        return;
    };
    let (line, code) = match completed {
        Ok(Completion {
            index,
            allow_unchecked,
            outcome: Ok(record),
        }) => result_line(index, record, allow_unchecked),
        Ok(Completion {
            index,
            outcome: Err(reason),
            ..
        }) => (
            OutputLine::Error {
                index: Some(index),
                reason,
            },
            1,
        ),
        Err(_) => (
            OutputLine::Error {
                index: None,
                reason: "completion observer failed".into(),
            },
            1,
        ),
    };
    *aggregate = aggregate_exit(*aggregate, code);
    if output_failed.is_none()
        && let Err(error) = emit(line).await
    {
        *output_failed = Some(error);
    }
}

/// Each entry receives fresh authority and freezes a bounded snapshot of recent
/// completed turns. Live memory never edits earlier immutable run records.
async fn run_session(
    handle: &ControllerHandle,
    startup: &Startup,
    store: &StorageClient,
    interrupted: &CancellationToken,
    json: bool,
) -> Result<u8, String> {
    let (workspace, model) = startup.session_defaults()?;
    if !json {
        let workspace_config = startup
            .config
            .workspace(&workspace)
            .ok_or("unknown workspace")?;
        let mut model_config = startup.config.model(&model).ok_or("unknown model")?.clone();
        if let Some(managed) = startup.config.managed_model() {
            model_config.model_id = managed
                .model_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
        }
        emit_human(presentation::session_intro(workspace_config, &model_config)).await?;
    }
    let model_config = startup.config.model(&model).ok_or("unknown model")?;
    let mut memory = session::Memory::new(
        model_config.context_size,
        startup.config.limits().max_output_tokens,
    );
    let mut last_run: Option<String> = None;
    let mut code = 0;
    loop {
        let Some(prompt) = read_entry(interrupted, json).await? else {
            return Ok(if interrupted.is_cancelled() {
                130
            } else {
                code
            });
        };
        if prompt.trim().is_empty() {
            continue;
        }
        if !json && prompt.trim().starts_with('/') {
            match prompt.trim() {
                "/exit" | "/quit" => return Ok(code),
                "/help" => emit_human(presentation::SESSION_HELP.into()).await?,
                "/clear" | "/new" => {
                    memory.clear();
                    emit_human("Session context cleared.\n\n".into()).await?;
                }
                "/context" => emit_human(memory.status()).await?,
                "/status" => {
                    let status = if let Some(run_id) = &last_run {
                        presentation::run_status(&retained_run(store, run_id.clone()).await?)
                    } else {
                        "No runs yet.\n\n".into()
                    };
                    emit_human(status).await?;
                }
                "/permissions" => {
                    let workspace_config = startup
                        .config
                        .workspace(&workspace)
                        .ok_or("unknown workspace")?;
                    emit_human(presentation::permissions(workspace_config)).await?;
                }
                _ => {
                    emit_human(
                        "Unknown session command. Type /help for available commands.\n\n".into(),
                    )
                    .await?
                }
            }
            continue;
        }
        let submission = Submission::Freeform {
            workspace: workspace.clone(),
            model: model.clone(),
            prompt: prompt.clone(),
            continues: None,
            limits: None,
            capture: None,
        };
        let (job, retained) = match startup.session_job(submission, &memory) {
            Ok(prepared) => prepared,
            Err(error) => {
                code = 1;
                session_error(error, json).await?;
                continue;
            }
        };
        let before = memory.context().map_or(0, |context| context.turns.len());
        if before > retained {
            session_notice(
                format!(
                    "[Omitted {} older turns from this request to fit its size limit.]\n\n",
                    before - retained
                ),
                json,
            )
            .await?;
        }
        let run_id = job.authority.run_id().to_owned();
        code = run_single(handle, job, true, (!json).then_some(store)).await?;
        last_run = Some(run_id.clone());
        if code == 0 {
            memory.retain_recent(retained);
            let record = retained_run(store, run_id.clone()).await?;
            if let Some(answer) = record
                .result
                .as_ref()
                .and_then(|value| value["candidate"].as_str())
                && !answer.trim().is_empty()
                && let Some(notice) = memory.remember(run_id, &prompt, answer)
            {
                session_notice(notice, json).await?;
            }
        } else {
            session_notice("[This entry was not added to memory. Earlier completed turns remain; files may have changed, so re-read them before retrying.]\n\n".into(), json).await?;
        }
        if interrupted.is_cancelled() {
            return Ok(code);
        }
    }
}

async fn session_notice(text: String, json: bool) -> Result<(), String> {
    if json {
        tokio::task::spawn_blocking(move || {
            let mut output = std::io::stderr().lock();
            output
                .write_all(text.as_bytes())
                .and_then(|()| output.flush())
                .map_err(|_| "CLI context notice output failed".to_owned())
        })
        .await
        .map_err(|_| "CLI context notice writer failed")?
    } else {
        emit_human(text).await
    }
}

/// Read one entry without tying runtime shutdown to an uninterruptible OS read.
/// At most one stdin reader exists: another is started only after it returned a
/// line. If cancelled, this read-only thread can live until process teardown;
/// the controller and journal still settle before the CLI returns its exit code.
async fn read_entry(interrupted: &CancellationToken, json: bool) -> Result<Option<String>, String> {
    if interrupted.is_cancelled() {
        return Ok(None);
    }
    let (send_line, line) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name("kinesin-console-input".into())
        .spawn(move || {
            let result = read_console_entry(json);
            let _ = send_line.send(result);
        })
        .map_err(|_| "cannot start console reader".to_owned())?;
    tokio::select! {
        biased;
        _ = interrupted.cancelled() => Ok(None),
        joined = line => joined.map_err(|_| "console reader failed".to_owned())?,
    }
}

fn read_console_entry(json: bool) -> Result<Option<String>, String> {
    use std::io::Write;
    // Human output and prompts use one flushed stream, so the next prompt
    // cannot overtake the answer. JSON keeps stdout exclusively JSONL.
    let mut out: Box<dyn Write> = if json {
        Box::new(std::io::stderr())
    } else {
        Box::new(std::io::stdout())
    };
    out.write_all(b"> ")
        .and_then(|()| out.flush())
        .map_err(|_| "CLI prompt output failed".to_owned())?;
    read_session_line(&mut std::io::stdin().lock())
}

fn read_session_line(input: &mut impl BufRead) -> Result<Option<String>, String> {
    let mut line = Vec::new();
    input
        .take((crate::config::MAX_PROMPT_BYTES + 1) as u64)
        .read_until(b'\n', &mut line)
        .map_err(|_| "cannot read session entry")?;
    if line.len() > crate::config::MAX_PROMPT_BYTES {
        return Err(
            "session entry exceeds 16 KiB; restart the session with a shorter request".into(),
        );
    }
    if line.is_empty() {
        return Ok(None);
    }
    String::from_utf8(line)
        .map(Some)
        .map_err(|_| "session entry must be UTF-8".into())
}

async fn session_error(reason: String, json: bool) -> Result<(), String> {
    if json {
        emit(OutputLine::Error {
            index: None,
            reason,
        })
        .await
    } else {
        emit_human(format!("Kinesin: {}\n\n", presentation::safe_text(&reason))).await
    }
}

async fn emit_human(text: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut output = std::io::stdout().lock();
        output
            .write_all(text.as_bytes())
            .and_then(|()| output.flush())
            .map_err(|_| "CLI output failed".to_owned())
    })
    .await
    .map_err(|_| "CLI output worker failed")?
}

async fn run_batch(
    handle: &ControllerHandle,
    startup: &Startup,
    mut reader: BatchReader,
    interrupted: &CancellationToken,
) -> Result<u8, String> {
    let mut pending = JoinSet::new();
    let mut aggregate = 0;
    let mut output_failed = None;
    let mut seen = 0;
    let capacity = startup
        .config
        .concurrency()
        .max_active_runs
        .checked_add(startup.config.concurrency().max_queued_runs)
        .ok_or("invalid batch capacity")?;
    'input: loop {
        if interrupted.is_cancelled() {
            aggregate = aggregate_exit(aggregate, 130);
            break;
        }
        if output_failed.is_some() {
            handle.shutdown();
            break;
        }
        if pending.len() >= capacity {
            complete_one(&mut pending, &mut aggregate, &mut output_failed).await;
            continue;
        }
        let read = tokio::task::spawn_blocking(move || {
            let item = reader.next();
            (reader, item)
        })
        .await;
        let (returned, item) = match read {
            Ok(value) => value,
            Err(_) => {
                output_failed = Some("batch reader worker failed".into());
                handle.shutdown();
                break;
            }
        };
        reader = returned;
        let (index, parsed) = match item {
            Ok(Some(item)) => item,
            Ok(None) => break,
            Err(reason) => {
                aggregate = aggregate_exit(aggregate, 1);
                if let Err(error) = emit(OutputLine::Error {
                    index: None,
                    reason,
                })
                .await
                {
                    output_failed = Some(error);
                }
                break;
            }
        };
        seen += 1;
        let prepared =
            parsed.and_then(|item| Ok((startup.job(item.submission)?, item.allow_unchecked)));
        let (job, allow_unchecked) = match prepared {
            Ok(prepared) => prepared,
            Err(reason) => {
                aggregate = aggregate_exit(aggregate, 1);
                if let Err(error) = emit(OutputLine::Error {
                    index: Some(index),
                    reason,
                })
                .await
                {
                    output_failed = Some(error);
                }
                continue;
            }
        };
        loop {
            if interrupted.is_cancelled() {
                aggregate = aggregate_exit(aggregate, 130);
                break 'input;
            }
            match handle.try_submit(job.clone(), None) {
                Ok(mut accepted) => {
                    match accepted.admitted().await {
                        Ok(_) => follow(accepted, index, allow_unchecked, &mut pending),
                        Err(reason) => {
                            aggregate = aggregate_exit(aggregate, 1);
                            if let Err(error) = emit(OutputLine::Error {
                                index: Some(index),
                                reason,
                            })
                            .await
                            {
                                output_failed = Some(error);
                            }
                            handle.shutdown();
                            break 'input;
                        }
                    }
                    break;
                }
                Err(reason) if reason == "controller_overloaded" => {
                    if pending.is_empty() {
                        handle.capacity_changed().await;
                    } else {
                        complete_one(&mut pending, &mut aggregate, &mut output_failed).await;
                    }
                    if output_failed.is_some() {
                        handle.shutdown();
                        break 'input;
                    }
                }
                Err(reason) => {
                    aggregate =
                        aggregate_exit(aggregate, if interrupted.is_cancelled() { 130 } else { 1 });
                    if let Err(error) = emit(OutputLine::Error {
                        index: Some(index),
                        reason,
                    })
                    .await
                    {
                        output_failed = Some(error);
                    }
                    break 'input;
                }
            }
        }
    }
    while !pending.is_empty() {
        complete_one(&mut pending, &mut aggregate, &mut output_failed).await;
        if output_failed.is_some() {
            handle.shutdown();
        }
    }
    if let Some(error) = output_failed {
        return Err(error);
    }
    if seen == 0 && !interrupted.is_cancelled() {
        return Err("batch input contains no entries".into());
    }
    Ok(aggregate)
}

pub fn exit_code(phase: &str, acceptance: &str, allow_unchecked: bool) -> u8 {
    match (phase, acceptance) {
        ("cancelled", _) => 130,
        ("completed", "passed") => 0,
        ("completed", "failed") => 2,
        ("completed", "unchecked") if allow_unchecked => 0,
        ("completed", _) => 3,
        _ => 1,
    }
}
pub fn aggregate_exit(current: u8, next: u8) -> u8 {
    if current == 130 || next == 130 {
        130
    } else if current == 1 || next == 1 {
        1
    } else if current == 2 || next == 2 {
        2
    } else if current == 3 || next == 3 {
        3
    } else {
        0
    }
}

#[cfg(test)]
mod tests {

    fn session_test_startup(fixture: &Fixture) -> Startup {
        let config = fixture.parse(crate::config::test_support::BASE).unwrap();
        Startup {
            models: BTreeMap::from([("local".into(), ModelClient::scripted([]))]),
            resources: RunResources::from_config(&config).unwrap(),
            config,
        }
    }

    fn session_test_submission(prompt: &str) -> Submission {
        Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: prompt.into(),
            continues: None,
            limits: None,
            capture: None,
        }
    }

    #[test]
    fn session_admission_fits_both_history_and_the_compiled_tool_request() {
        let fixture = Fixture::new();
        let startup = session_test_startup(&fixture);
        let mut memory = session::Memory::new(16_384, 512);
        for index in 0..3 {
            memory.remember(format!("turn-{index}"), &"p".repeat(300), &"a".repeat(400));
        }
        let original = memory.context().unwrap();
        let submission = session_test_submission("Continue the file work.");
        let full = startup
            .config
            .authorize_local_session(submission.clone(), Some(original.clone()))
            .unwrap();
        let state_for = |authority: &crate::policy::RunAuthority| {
            crate::core::initiate_with_context(
                authority.instructions().into(),
                None,
                authority
                    .session_context()
                    .map(|context| context.encode().unwrap()),
                authority.prompt().into(),
                !authority.workspace().tools.is_empty(),
            )
            .unwrap()
            .0
        };
        let state = state_for(&full);
        let history_bytes = crate::model::history_len(state.messages()).unwrap();
        let prepared =
            crate::model::prepare(state.messages(), &crate::runner::options(&full)).unwrap();
        let request_bytes = prepared.bytes().len();
        assert!(
            request_bytes > history_bytes + 1,
            "tool definitions must contribute to the request budget"
        );

        for limits in [
            crate::policy::LimitOverrides {
                max_request_bytes: Some(request_bytes - 1),
                ..Default::default()
            },
            crate::policy::LimitOverrides {
                max_history_bytes: Some(history_bytes - 1),
                ..Default::default()
            },
        ] {
            let mut bounded = submission.clone();
            let Submission::Freeform {
                limits: requested, ..
            } = &mut bounded
            else {
                unreachable!();
            };
            *requested = Some(limits);
            let (job, retained) = startup.session_job(bounded, &memory).unwrap();
            assert_eq!(retained, 2);
            assert_eq!(
                job.authority.session_context().unwrap().turns,
                original.turns[1..]
            );
            assert_eq!(job.authority.workspace().tools, full.workspace().tools);
            assert_eq!(job.authority.prompt(), full.prompt());
            let state = state_for(&job.authority);
            assert!(
                crate::model::history_len(state.messages()).unwrap()
                    <= job.authority.limits().max_history_bytes
            );
            crate::model::prepare(state.messages(), &crate::runner::options(&job.authority))
                .unwrap();
            assert_eq!(
                memory.context().unwrap(),
                original,
                "preparation must preserve history until a run succeeds"
            );
        }
    }

    #[test]
    fn rejected_session_requests_preserve_completed_context() {
        let fixture = Fixture::new();
        let startup = session_test_startup(&fixture);
        let mut memory = session::Memory::new(4_096, 512);
        memory.remember(
            "completed-run".into(),
            "Remember cobalt.",
            "I will remember cobalt.",
        );
        let original = memory.context().unwrap();
        let mut oversized =
            session_test_submission("A valid request that cannot fit its configured budget.");
        let Submission::Freeform { limits, .. } = &mut oversized else {
            unreachable!();
        };
        *limits = Some(crate::policy::LimitOverrides {
            max_request_bytes: Some(1),
            ..Default::default()
        });
        for submission in [session_test_submission("   "), oversized] {
            assert!(startup.session_job(submission, &memory).is_err());
            assert_eq!(memory.context().unwrap(), original);
        }
    }

    #[test]
    fn model_setup_flags_are_confined_to_human_sessions() {
        let args = |items: &[&str]| items.iter().map(OsString::from).collect::<Vec<_>>();
        let CliCommand::Session { options, .. } = parse(args(&[
            "--model-path",
            "other.gguf",
            "--runtime-path",
            "llama-server",
        ]))
        .unwrap() else {
            panic!("local path flags must prepare a session");
        };
        assert_eq!(options.model_path, Some(PathBuf::from("other.gguf")));
        assert!(!options.external);
        for flags in [
            vec!["--external", "--model-path", "model.gguf"],
            vec!["--model-path", "model.gguf", "--json"],
            vec!["--external", "--config", "config.toml"],
            vec![
                "--runtime-path",
                "llama-server",
                "--workspace",
                "w",
                "--model",
                "m",
                "--prompt",
                "p",
            ],
            vec!["--external", "--external"],
            vec!["--model-path"],
        ] {
            assert!(
                parse(args(&flags)).is_err(),
                "accepted conflicting setup flags"
            );
        }
    }

    #[test]
    fn session_input_stops_reading_at_the_prompt_bound() {
        let oversized = vec![b'x'; crate::config::MAX_PROMPT_BYTES * 4];
        let mut input = std::io::Cursor::new(oversized);
        assert!(
            read_session_line(&mut input)
                .unwrap_err()
                .contains("exceeds 16 KiB")
        );
        assert_eq!(
            input.position(),
            (crate::config::MAX_PROMPT_BYTES + 1) as u64
        );
        let mut input = std::io::Cursor::new(b"hello\r\nnext\n");
        assert_eq!(
            read_session_line(&mut input).unwrap().as_deref(),
            Some("hello\r\n")
        );
        assert_eq!(
            read_session_line(&mut input).unwrap().as_deref(),
            Some("next\n")
        );
        assert_eq!(read_session_line(&mut input).unwrap(), None);
        assert!(read_session_line(&mut &b"\xff\n"[..]).is_err());
    }

    #[test]
    fn the_bare_command_opens_a_session_and_continuation_needs_no_flag() {
        let args = |items: &[&str]| {
            items
                .iter()
                .map(|item| OsString::from(*item))
                .collect::<Vec<_>>()
        };
        // The common entry point takes no arguments at all.
        let CliCommand::Session {
            config,
            json,
            explicit_config,
            ..
        } = parse(args(&[])).unwrap()
        else {
            panic!("no arguments opens a session");
        };
        assert_eq!(config, PathBuf::from(DEFAULT_CONFIG));
        assert!(!json);
        assert!(!explicit_config);

        let CliCommand::Session {
            config,
            json,
            explicit_config,
            ..
        } = parse(args(&["--config", "other.toml"])).unwrap()
        else {
            panic!("a config override still opens a session");
        };
        assert_eq!(config, PathBuf::from("other.toml"));
        assert!(!json);
        assert!(explicit_config);
        assert!(matches!(
            parse(args(&["--json"])).unwrap(),
            CliCommand::Session { json: true, .. }
        ));
        assert!(parse(args(&["--json", "--json"])).is_err());
        assert!(
            parse(args(&[
                "--json",
                "--workspace",
                "w",
                "--model",
                "m",
                "--prompt",
                "p"
            ]))
            .is_err()
        );

        // There is no `run` verb — running is what bare `kinesin` does, so flags
        // alone are a one-shot run.
        assert!(
            matches!(
                parse(args(&["--workspace", "w", "--model", "m", "--prompt", "p"])).unwrap(),
                CliCommand::Run(_)
            ),
            "flags alone run the app"
        );
        assert!(
            parse(args(&[
                "run",
                "--workspace",
                "w",
                "--model",
                "m",
                "--prompt",
                "p"
            ]))
            .is_err(),
            "run is not a subcommand"
        );
        // Continuation is a property of the session, not a flag.
        assert!(
            parse(args(&[
                "--config",
                "kinesin.toml",
                "--continue",
                "11111111-1111-4111-8111-111111111111",
                "--prompt",
                "and why?",
            ]))
            .is_err(),
            "there is no continuation flag"
        );
    }

    use super::*;
    use crate::config::test_support::Fixture;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn disjoint_shapes_and_unknown_flags_fail_before_io() {
        for values in [
            vec![
                "--config", "x", "--model", "m", "--task", "t", "--prompt", "other",
            ],
            vec![
                "--config",
                "x",
                "--model",
                "m",
                "--task",
                "t",
                "--allow-unchecked",
            ],
            vec!["--config", "x", "--model", "m", "--owner", "root"],
            vec![
                "batch",
                "--config",
                "x",
                "--input",
                "jobs",
                "--allow-unchecked",
            ],
        ] {
            assert!(parse(args(&values)).is_err());
        }
        assert!(
            parse(args(&[
                "--config",
                "x",
                "--model",
                "m",
                "--workspace",
                "w",
                "--prompt",
                "p"
            ]))
            .is_ok()
        );
        assert!(matches!(
            parse(args(&["batch", "--config", "x", "--input", "jobs"])).unwrap(),
            CliCommand::Batch(_)
        ));
    }

    #[test]
    fn process_success_does_not_replace_task_acceptance() {
        assert_eq!(exit_code("completed", "unchecked", false), 3);
        assert_eq!(exit_code("completed", "unchecked", true), 0);
        assert_eq!(exit_code("completed", "failed", true), 2);
        assert_eq!(exit_code("completed", "inconclusive", true), 3);
        assert_eq!(exit_code("completed", "passed", false), 0);
        assert_eq!(exit_code("interrupted", "inconclusive", false), 1);
        for (a, b, expected) in [(0, 3, 3), (3, 2, 2), (2, 1, 1), (1, 130, 130)] {
            assert_eq!(aggregate_exit(a, b), expected);
            assert_eq!(aggregate_exit(b, a), expected);
        }
    }

    #[test]
    fn batch_rows_reject_contract_overrides_and_duplicate_fields() {
        assert!(BatchItem::parse(br#"{"submission":{"mode":"checked","task":"t","model":"m"},"allow_unchecked":true}"#).is_err());
        assert!(
            BatchItem::parse(
                br#"{"submission":{"mode":"checked","task":"t","model":"m","prompt":"other"}}"#
            )
            .is_err()
        );
        assert!(BatchItem::parse(br#"{"submission":{"mode":"freeform","workspace":"w","model":"m","prompt":"x"},"allow_unchecked":true,"allow_unchecked":false}"#).is_err());
    }

    #[test]
    fn incremental_reader_bounds_lines_entries_and_consumed_bytes() {
        let fixture = Fixture::new();
        let path = fixture.root.join("batch.jsonl");
        let row = "{\"submission\":{\"mode\":\"checked\",\"task\":\"t\",\"model\":\"m\"}}\r\n";
        std::fs::write(&path, format!("{row}{{bad json}}\n{row}")).unwrap();
        let mut reader = BatchReader::open(&path).unwrap();
        assert!(reader.next().unwrap().unwrap().1.is_ok());
        assert!(reader.next().unwrap().unwrap().1.is_err());
        assert!(reader.next().unwrap().unwrap().1.is_ok());
        assert!(reader.next().unwrap().is_none());
        std::fs::write(&path, vec![b'x'; MAX_BATCH_LINE_BYTES + 1]).unwrap();
        assert!(BatchReader::open(&path).unwrap().next().is_err());
        std::fs::write(&path, row.repeat(MAX_BATCH_ENTRIES + 1)).unwrap();
        let mut reader = BatchReader::open(&path).unwrap();
        for _ in 0..MAX_BATCH_ENTRIES {
            reader.next().unwrap().unwrap().1.unwrap();
        }
        assert!(reader.next().is_err());
        std::fs::write(&path, row).unwrap();
        let mut reader = BatchReader::open(&path).unwrap();
        reader.total_bytes = MAX_BATCH_BYTES;
        assert!(reader.next().is_err());
    }

    #[test]
    fn json_output_preserves_data_without_terminal_control_sequences() {
        let text = "line\r\n\u{001b}[31m\u{009b}31m";
        let output = output_bytes(&OutputLine::Error {
            index: Some(1),
            reason: text.into(),
        })
        .unwrap();
        assert_eq!(output.iter().filter(|byte| **byte == b'\n').count(), 1);
        assert!(!output.contains(&0x1b));
        let decoded: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(decoded["reason"], text);
        assert!(!String::from_utf8(output).unwrap().contains('\u{009b}'));
    }

    #[test]
    fn complete_cli_batch_checks_more_tasks_than_controller_capacity() {
        use crate::config::test_support::{BASE, TASK};
        use crate::storage::{Command, Response, Store};
        use serde_json::json;
        use std::io::Read;
        use std::net::TcpListener;
        use std::time::Duration;

        let fixture = Fixture::new();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let source = format!(
            "{}{TASK}\n[concurrency]\nmax_active_runs = 1\nmax_queued_runs = 1\n",
            BASE.replace("http://127.0.0.1:8080", &origin)
        );
        std::fs::write(fixture.path(), source).unwrap();
        std::fs::write(
            fixture.root.join("workspace/project.txt"),
            "language=Rust\n",
        )
        .unwrap();
        let input = fixture.root.join("batch.jsonl");
        std::fs::write(&input, "{\"submission\":{\"mode\":\"checked\",\"task\":\"practice-fields\",\"model\":\"local\"}}\n".repeat(6)).unwrap();
        // A finite synthetic provider uses only the tool observation to answer.
        // Every checked task needs a real read and a second HTTP exchange.
        // The gap between exchanges covers journal, checker, admission and batch
        // pacing for the next run, so this deadline detects a hung fixture rather
        // than asserting a rate. If it expires early the listener drops and the
        // next connect is refused, which reports as a model connection failure.
        let accept_deadline = Duration::from_secs(60);
        let provider = std::thread::spawn(move || {
            // Six checked tasks at three exchanges each.
            for _ in 0..18 {
                let deadline = std::time::Instant::now() + accept_deadline;
                let (mut connection, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(
                                std::time::Instant::now() < deadline,
                                "provider fixture request deadline"
                            );
                            std::thread::sleep(Duration::from_millis(2));
                        }
                        Err(error) => panic!("provider fixture accept failed: {error}"),
                    }
                };
                // Accepted Windows sockets can inherit the listener's mode.
                // The finite fixture reads blocking HTTP with its own timeout.
                connection.set_nonblocking(false).unwrap();
                connection
                    .set_read_timeout(Some(Duration::from_secs(30)))
                    .unwrap();
                let mut header = Vec::new();
                while !header.ends_with(b"\r\n\r\n") {
                    assert!(header.len() < 16_384);
                    let mut byte = [0];
                    connection.read_exact(&mut byte).unwrap();
                    header.push(byte[0]);
                }
                let header = String::from_utf8(header).unwrap();
                let length: usize = header
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|value| value.trim().parse().unwrap())
                    })
                    .unwrap();
                assert!(length <= 131_072);
                let mut body = vec![0; length];
                connection.read_exact(&mut body).unwrap();
                let request: serde_json::Value = serde_json::from_slice(&body).unwrap();
                let messages = request["messages"].as_array().unwrap();
                let last = messages.last().unwrap();
                // A checked run takes three turns: the tool call, a prose
                // answer, then the constrained candidate.
                let (reason, message) = if request.get("response_format").is_some() {
                    // ADR-010 end to end: a constrained request carries no tools.
                    assert!(
                        request.get("tools").is_none(),
                        "a constrained turn must withdraw tools"
                    );
                    assert_eq!(request["response_format"]["type"], "json_schema");
                    let observed: serde_json::Value = messages
                        .iter()
                        .rev()
                        .find(|message| message["role"] == "tool")
                        .map(|message| {
                            serde_json::from_str(message["content"].as_str().unwrap()).unwrap()
                        })
                        .expect("the gather turn observed a file");
                    assert_eq!(observed["status"], "ok");
                    let value = observed["body"]
                        .as_str()
                        .unwrap()
                        .lines()
                        .find_map(|line| line.strip_prefix("language="))
                        .unwrap();
                    let answer = json!({"facts":[{"id":"language","value":value,"evidence_id":observed["evidence_id"]}]}).to_string();
                    ("stop", json!({"role":"assistant","content":answer}))
                } else if last["role"] == "tool" {
                    (
                        "stop",
                        json!({"role":"assistant","content":"I read the file and found the language."}),
                    )
                } else {
                    (
                        "tool_calls",
                        json!({"role":"assistant","content":null,"tool_calls":[{"id":"read-1","type":"function","function":{"name":"read_file","arguments":"{\"path\":\"project.txt\"}"}}]}),
                    )
                };
                let reply =
                    json!({"choices":[{"index":0,"finish_reason":reason,"message":message}],
                    "usage":{"prompt_tokens":11,"completion_tokens":3,"total_tokens":14}})
                    .to_string();
                write!(&mut connection, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", reply.len(), reply).unwrap();
                connection.flush().unwrap();
            }
        });
        let result = execute(CliCommand::Batch(BatchCommand {
            config: fixture.path(),
            input,
        }));
        provider.join().unwrap();
        let code = result.unwrap();
        assert_eq!(code, 0);
        let mut store = Store::open(&fixture.root.join("state/kinesin.sqlite")).unwrap();
        let Response::Runs(records) = store
            .execute(Command::List {
                owner_id: "local".into(),
                after: None,
                limit: 100,
            })
            .unwrap()
        else {
            panic!("run page")
        };
        assert_eq!(records.len(), 6);
        assert!(records.iter().all(RunRecord::task_accepted));
        assert!(records.iter().all(|record| record.receipt.is_some()));
        drop(store);
    }

    #[test]
    fn test_cli_run_records_token_totals() {
        use crate::config::test_support::BASE;
        use crate::storage::{Command, Response, Store};
        use serde_json::json;
        use std::io::Read;
        use std::net::TcpListener;
        use std::time::Duration;

        // End to end through the real CLI surface, loopback HTTP and SQLite: a
        // freeform run whose one reply carries a `usage` object records the
        // reported prompt and completion tokens in the stored run's terminal
        // counters, read back through the run record.
        let fixture = Fixture::new();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let source = BASE.replace("http://127.0.0.1:8080", &origin);
        std::fs::write(fixture.path(), source).unwrap();

        let accept_deadline = Duration::from_secs(60);
        let provider = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + accept_deadline;
            let (mut connection, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "provider fixture request deadline"
                        );
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    Err(error) => panic!("provider fixture accept failed: {error}"),
                }
            };
            connection.set_nonblocking(false).unwrap();
            connection
                .set_read_timeout(Some(Duration::from_secs(30)))
                .unwrap();
            let mut header = Vec::new();
            while !header.ends_with(b"\r\n\r\n") {
                assert!(header.len() < 16_384);
                let mut byte = [0];
                connection.read_exact(&mut byte).unwrap();
                header.push(byte[0]);
            }
            let header = String::from_utf8(header).unwrap();
            let length: usize = header
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|value| value.trim().parse().unwrap())
                })
                .unwrap();
            let mut body = vec![0; length];
            connection.read_exact(&mut body).unwrap();
            // The model answers directly; its reply reports usage.
            let reply = json!({
                "choices":[{"index":0,"finish_reason":"stop",
                    "message":{"role":"assistant","content":"hello"}}],
                "usage":{"prompt_tokens":17,"completion_tokens":4,"total_tokens":21}})
            .to_string();
            write!(&mut connection, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", reply.len(), reply).unwrap();
            connection.flush().unwrap();
        });

        let command = parse(args(&[
            "--config",
            &fixture.path().to_string_lossy(),
            "--workspace",
            "practice",
            "--model",
            "local",
            "--prompt",
            "Say hello.",
            "--allow-unchecked",
        ]))
        .unwrap();
        let result = execute(command);
        provider.join().unwrap();
        result.unwrap();

        let mut store = Store::open(&fixture.root.join("state/kinesin.sqlite")).unwrap();
        let Response::Runs(records) = store
            .execute(Command::List {
                owner_id: "local".into(),
                after: None,
                limit: 100,
            })
            .unwrap()
        else {
            panic!("run page")
        };
        assert_eq!(records.len(), 1);
        let Response::Events(events) = store
            .execute(Command::Events {
                owner_id: "local".into(),
                run_id: records[0].run_id.clone(),
                after: None,
                limit: 100,
            })
            .unwrap()
        else {
            panic!("event page")
        };
        let counters = &events
            .iter()
            .find(|event| event.kind == "run_finished")
            .unwrap()
            .data["counters"];
        assert_eq!(counters["prompt_tokens"], 17);
        assert_eq!(counters["completion_tokens"], 4);
        drop(store);
    }
}
