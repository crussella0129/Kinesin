//! A single SQLite owner with bounded, commit-acknowledged commands.
//!
//! Startup and the synchronous `Store` API perform blocking I/O. Production
//! async callers use `StorageClient`; reserve before terminal arbitration, then
//! transfer the final command synchronously. Dropping its acknowledgement never
//! recalls a command or releases its outstanding-work reservations early.

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc, oneshot, watch};
use tokio::time::{Instant, timeout_at};

const SCHEMA: &str = include_str!("storage_schema.sql");
const MAX_EVENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_COMMAND_BYTES: usize = 8 * 1024 * 1024;
const MAX_RESULT_BYTES: usize = 2 * 1024 * 1024;
const MAX_CANDIDATE_BYTES: usize = 1024 * 1024;
const MAX_RECEIPT_BYTES: usize = 8192;
const MAX_PAGE_BYTES: usize = 4 * 1024 * 1024;
const MAX_PAGE_SIZE: usize = 100;
const RUN_COLUMNS: &str = "run_id,owner_id,workspace_id,model_profile_id,task_mode,task_profile_id,task_profile_version,task_spec_sha256,checker_id,checker_version,capture,phase,acceptance_status,created_unix_ms,policy_version,submission_sha256,idempotency_key,terminal_reason,result_json,result_sha256,acceptance_json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageError {
    pub code: &'static str,
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code)
    }
}
impl std::error::Error for StorageError {}
impl From<rusqlite::Error> for StorageError {
    fn from(_: rusqlite::Error) -> Self {
        error("storage_database")
    }
}
impl From<serde_json::Error> for StorageError {
    fn from(_: serde_json::Error) -> Self {
        error("storage_invalid_json")
    }
}
fn error(code: &'static str) -> StorageError {
    StorageError { code }
}
type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskIdentity {
    pub profile_id: String,
    pub profile_version: String,
    pub spec_sha256: String,
    pub checker_id: String,
    pub checker_version: String,
    pub criterion_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Admission {
    pub run_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub model_profile_id: String,
    pub task: Option<TaskIdentity>,
    pub capture: String,
    pub created_unix_ms: i64,
    pub policy_version: String,
    pub submission_sha256: String,
    pub idempotency_key: Option<String>,
    /// Safe descriptors; exact inputs may appear only in a `replay` member
    /// when capture is explicitly `replay`.
    pub data: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub seq: u64,
    pub kind: String,
    pub elapsed_ms: u64,
    pub data: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Terminal {
    pub phase: String,
    pub reason: String,
    pub result: Option<Value>,
    pub result_sha256: Option<String>,
    pub acceptance_status: String,
    pub receipt: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RunRecord {
    pub run_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub model_profile_id: String,
    pub task_mode: String,
    pub task_profile_id: Option<String>,
    pub task_profile_version: Option<String>,
    pub task_spec_sha256: Option<String>,
    pub checker_id: Option<String>,
    pub checker_version: Option<String>,
    pub capture: String,
    pub phase: String,
    pub acceptance_status: String,
    pub created_unix_ms: i64,
    pub policy_version: String,
    pub submission_sha256: String,
    pub idempotency_key: Option<String>,
    pub terminal_reason: Option<String>,
    pub result: Option<Value>,
    pub result_sha256: Option<String>,
    pub receipt: Option<Value>,
}

impl RunRecord {
    pub fn task_accepted(&self) -> bool {
        self.phase == "completed" && self.acceptance_status == "passed"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCursor {
    pub created_unix_ms: i64,
    pub run_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Admit(Admission),
    Append {
        owner_id: String,
        run_id: String,
        event: Event,
        phase: Option<String>,
    },
    Finish {
        owner_id: String,
        run_id: String,
        event: Event,
        terminal: Terminal,
    },
    Get {
        owner_id: String,
        run_id: String,
    },
    Lookup {
        owner_id: String,
        key: String,
        submission_sha256: String,
    },
    List {
        owner_id: String,
        after: Option<RunCursor>,
        limit: usize,
    },
    Events {
        owner_id: String,
        run_id: String,
        after: Option<u64>,
        limit: usize,
    },
    /// Operator commands, never submitted by model or ordinary service clients.
    Retain {
        now_unix_ms: i64,
        limit: usize,
    },
    Backup {
        destination: PathBuf,
    },
    Health,
}

impl Command {
    /// Exact serialized command bytes used for capacity accounting.
    pub fn encoded_len(&self) -> Result<usize> {
        serialized_len(self, MAX_COMMAND_BYTES)?;
        serialized_len(&self.clone().prepare()?, MAX_COMMAND_BYTES)
    }

    fn prepare(mut self) -> Result<Self> {
        match &mut self {
            Self::Admit(admission) => {
                validate_admission(admission)?;
                let object = admission
                    .data
                    .as_object_mut()
                    .ok_or_else(|| error("storage_invalid_event"))?;
                object.insert("owner_id".into(), json!(admission.owner_id));
                object.insert("task".into(), json!(admission.task));
                object.insert(
                    "required_criterion_ids".into(),
                    json!(
                        admission
                            .task
                            .as_ref()
                            .map(|t| t.criterion_ids.clone())
                            .unwrap_or_default()
                    ),
                );
                serialized_len(
                    &Event {
                        seq: 0,
                        kind: "run_accepted".into(),
                        elapsed_ms: 0,
                        data: admission.data.clone(),
                    },
                    MAX_EVENT_BYTES,
                )?;
            }
            Self::Append { event, .. } => {
                serialized_len(event, MAX_EVENT_BYTES)?;
            }
            Self::Finish {
                event, terminal, ..
            } => {
                serialized_len(&terminal.receipt, MAX_RECEIPT_BYTES)?;
                if let Some(result) = &terminal.result {
                    serialized_len(result, MAX_RESULT_BYTES)?;
                }
                let object = event
                    .data
                    .as_object_mut()
                    .ok_or_else(|| error("storage_invalid_event"))?;
                object.insert("phase".into(), json!(terminal.phase));
                object.insert(
                    "acceptance_status".into(),
                    json!(terminal.acceptance_status),
                );
                object.insert("receipt".into(), terminal.receipt.clone());
                object.insert("candidate_sha256".into(), json!(terminal.result_sha256));
                serialized_len(event, MAX_EVENT_BYTES)?;
            }
            Self::List { limit, .. } | Self::Events { limit, .. } => page_limit(*limit)?,
            Self::Get { .. }
            | Self::Lookup { .. }
            | Self::Retain { .. }
            | Self::Backup { .. }
            | Self::Health => {}
        }
        Ok(self)
    }
}

fn serialized_len(value: &impl Serialize, limit: usize) -> Result<usize> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit - self.bytes {
                return Err(std::io::Error::other("serialized value exceeds its bound"));
            }
            self.bytes += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, value).map_err(|_| error("storage_serialized_limit"))?;
    Ok(counter.bytes)
}

fn retained_run_bound(admission: &Admission) -> Result<u64> {
    let limits = &admission.data["limits"];
    // Metadata reserves bounded descriptors and the retained final candidate.
    // Replay additionally reserves every permitted observation/argument body.
    let turns = limits["max_model_turns"].as_u64().unwrap_or(12);
    let calls = limits["max_tool_calls"].as_u64().unwrap_or(24);
    let (model_bytes, tool_bytes) = if admission.capture == "replay" {
        (2 * 1024 * 1024, 80 * 1024)
    } else {
        (4096, 4096)
    };
    turns
        .checked_mul(model_bytes)
        .and_then(|models| {
            calls
                .checked_mul(tool_bytes)
                .and_then(|tools| models.checked_add(tools))
        })
        .and_then(|bytes| bytes.checked_add(4 * 1024 * 1024))
        .ok_or_else(|| error("storage_admission_capacity"))
}

#[derive(Clone, Debug)]
pub enum Response {
    Admitted {
        record: Box<RunRecord>,
        created: bool,
    },
    Updated,
    Run(Option<Box<RunRecord>>),
    Runs(Vec<RunRecord>),
    Events(Vec<Event>),
    Retained(usize),
    Health {
        database_bytes: u64,
        wal_bytes: u64,
        reserved_bytes: u64,
        runs: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct RetentionPolicy {
    pub max_bytes: u64,
    pub max_runs: u64,
    pub minimum_hours: u64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_bytes: 4 * 1024 * 1024 * 1024,
            max_runs: 100_000,
            minimum_hours: 24,
        }
    }
}

impl RetentionPolicy {
    pub fn validate(&self) -> Result<()> {
        if self.max_bytes < 4 * 1024 * 1024
            || self.max_bytes > i64::MAX as u64
            || self.max_runs == 0
            || self.max_runs > 1_000_000
            || !(24..=u64::from(u32::MAX)).contains(&self.minimum_hours)
        {
            return Err(error("storage_invalid_retention_policy"));
        }
        Ok(())
    }
}

struct ControllerLock(File);

impl Drop for ControllerLock {
    fn drop(&mut self) {
        // A concurrently forked child can retain the same Unix open file
        // description until exec. Closing only our descriptor would leave
        // its flock held, even after this controller has fully shut down.
        let _ = self.0.unlock();
    }
}

pub struct Store {
    connection: Connection,
    // Field drop order closes SQLite before explicitly releasing ownership.
    _controller_lock: ControllerLock,
    path: PathBuf,
    policy: RetentionPolicy,
    reservations: BTreeMap<String, u64>,
}

impl Store {
    /// The caller provisions private permissions on this local state directory.
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_policy(path, RetentionPolicy::default())
    }

    pub fn open_with_policy(path: &Path, policy: RetentionPolicy) -> Result<Self> {
        policy.validate()?;
        if rusqlite::version_number() < 3_051_003 {
            return Err(error("storage_sqlite_too_old"));
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| error("storage_invalid_path"))?;
        std::fs::create_dir_all(parent).map_err(|_| error("storage_directory"))?;
        let mut lock_options = OpenOptions::new();
        lock_options
            .read(true)
            .write(true)
            .create(true)
            .truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // A private directory alone does not satisfy the retained-tree
            // policy: its newly created files must also be owner-only.
            lock_options.mode(0o600);
        }
        let controller_lock = lock_options
            .open(parent.join("controller.lock"))
            .map_err(|_| error("storage_lock_open"))?;
        controller_lock
            .try_lock()
            .map_err(|_| error("storage_controller_locked"))?;
        let controller_lock = ControllerLock(controller_lock);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // SQLite defaults a new database to 0644. Precreate it owner-only
            // so SQLite's WAL/SHM files inherit that mode. create_new preserves
            // every existing database and its administrator-provided mode.
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
            {
                Ok(file) => drop(file),
                Err(problem) if problem.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(error("storage_database_create")),
            }
        }
        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        let version: i64 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version == 0 {
            let tables: i64 = connection.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r| r.get(0))?;
            if tables != 0 {
                return Err(error("storage_unknown_schema"));
            }
            connection.execute_batch(&format!("BEGIN IMMEDIATE;\n{SCHEMA}\nCOMMIT;"))?;
        } else if version != 2 {
            return Err(error("storage_unknown_schema"));
        }
        let wal: String = connection.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
        let sync: i64 = connection.pragma_query_value(None, "synchronous", |r| r.get(0))?;
        let foreign: i64 = connection.pragma_query_value(None, "foreign_keys", |r| r.get(0))?;
        if wal != "wal" || sync != 2 || foreign != 1 {
            return Err(error("storage_configuration"));
        }
        let mut store = Self {
            connection,
            _controller_lock: controller_lock,
            path: path.to_owned(),
            policy,
            reservations: BTreeMap::new(),
        };
        store.recover()?;
        Ok(store)
    }

    pub fn execute(&mut self, command: Command) -> Result<Response> {
        match command {
            Command::Admit(admission) => self.admit(admission),
            Command::Append {
                owner_id,
                run_id,
                event,
                phase,
            } => {
                self.append(&owner_id, &run_id, event, phase.as_deref())?;
                Ok(Response::Updated)
            }
            Command::Finish {
                owner_id,
                run_id,
                event,
                terminal,
            } => {
                self.finish(&owner_id, &run_id, event, terminal)?;
                self.reservations.remove(&run_id);
                Ok(Response::Updated)
            }
            Command::Get { owner_id, run_id } => Ok(Response::Run(
                get_run(&self.connection, &owner_id, &run_id)?.map(Box::new),
            )),
            Command::Lookup {
                owner_id,
                key,
                submission_sha256,
            } => Ok(Response::Run(
                lookup(&self.connection, &owner_id, &key, &submission_sha256)?.map(Box::new),
            )),
            Command::List {
                owner_id,
                after,
                limit,
            } => Ok(Response::Runs(self.list(
                &owner_id,
                after.as_ref(),
                limit,
            )?)),
            Command::Events {
                owner_id,
                run_id,
                after,
                limit,
            } => Ok(Response::Events(read_events(
                &self.connection,
                &owner_id,
                &run_id,
                after,
                limit,
            )?)),
            Command::Retain { now_unix_ms, limit } => {
                if now_unix_ms < 0 || limit == 0 || limit > 100 {
                    return Err(error("storage_invalid_retention_request"));
                }
                let cutoff = now_unix_ms.saturating_sub(
                    i64::try_from(self.policy.minimum_hours * 3_600_000)
                        .map_err(|_| error("storage_invalid_retention_policy"))?,
                );
                let transaction = self.connection.transaction()?;
                let deleted = transaction.execute("DELETE FROM runs WHERE run_id IN (SELECT run_id FROM runs WHERE phase IN ('completed','stopped','failed','cancelled','interrupted') AND created_unix_ms <= ?1 ORDER BY created_unix_ms,run_id LIMIT ?2)", params![cutoff,limit as i64])?;
                transaction.commit()?;
                self.connection
                    .execute_batch("PRAGMA wal_checkpoint(PASSIVE)")?;
                Ok(Response::Retained(deleted))
            }
            Command::Backup { destination } => {
                // Only create a new operator-selected file in a separately
                // provisioned private directory; never overwrite a live database.
                if !destination.is_absolute()
                    || destination.parent() == self.path.parent()
                    || destination.exists()
                {
                    return Err(error("storage_invalid_backup_destination"));
                }
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                options
                    .open(&destination)
                    .map_err(|_| error("storage_backup_create"))?;
                let mut target = Connection::open(&destination)?;
                let backup = rusqlite::backup::Backup::new(&self.connection, &mut target)?;
                backup.run_to_completion(128, Duration::from_millis(1), None)?;
                drop(backup);
                let integrity: String =
                    target.pragma_query_value(None, "integrity_check", |row| row.get(0))?;
                if integrity != "ok" {
                    return Err(error("storage_backup_integrity"));
                }
                Ok(Response::Updated)
            }
            Command::Health => self.health(),
        }
    }

    fn admit(&mut self, admission: Admission) -> Result<Response> {
        validate_admission(&admission)?;
        // Retried keys bypass new-run headroom checks, but remain scoped to
        // their original owner and request fingerprint.
        if let Some(key) = &admission.idempotency_key
            && let Some(record) = lookup(
                &self.connection,
                &admission.owner_id,
                key,
                &admission.submission_sha256,
            )?
        {
            return Ok(Response::Admitted {
                record: Box::new(record),
                created: false,
            });
        }
        let reservation = retained_run_bound(&admission)?;
        let Response::Health {
            database_bytes,
            wal_bytes,
            reserved_bytes,
            runs,
        } = self.health()?
        else {
            unreachable!()
        };
        if runs >= self.policy.max_runs
            || database_bytes
                .saturating_add(wal_bytes)
                .saturating_add(reserved_bytes)
                .saturating_add(reservation)
                > self.policy.max_bytes
        {
            return Err(error("storage_admission_capacity"));
        }
        let transaction = self.connection.transaction()?;
        if let Some(key) = &admission.idempotency_key
            && let Some(record) = lookup(
                &transaction,
                &admission.owner_id,
                key,
                &admission.submission_sha256,
            )?
        {
            return Ok(Response::Admitted {
                record: Box::new(record),
                created: false,
            });
        }
        let task = admission.task.as_ref();
        transaction.execute(
            "INSERT INTO runs (run_id,owner_id,workspace_id,model_profile_id,task_mode,task_profile_id,task_profile_version,task_spec_sha256,checker_id,checker_version,capture,phase,acceptance_status,created_unix_ms,policy_version,submission_sha256,idempotency_key) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'queued',?12,?13,?14,?15,?16)",
            params![admission.run_id, admission.owner_id, admission.workspace_id, admission.model_profile_id,
                if task.is_some() { "checked" } else { "freeform" }, task.map(|t| &t.profile_id), task.map(|t| &t.profile_version), task.map(|t| &t.spec_sha256), task.map(|t| &t.checker_id), task.map(|t| &t.checker_version), admission.capture,
                if task.is_some() { "pending" } else { "unchecked" }, admission.created_unix_ms, admission.policy_version, admission.submission_sha256, admission.idempotency_key],
        )?;
        let mut data = admission.data;
        let object = data
            .as_object_mut()
            .ok_or_else(|| error("storage_invalid_event"))?;
        object.insert("owner_id".into(), json!(admission.owner_id));
        object.insert("task".into(), json!(task));
        object.insert(
            "required_criterion_ids".into(),
            json!(task.map(|t| t.criterion_ids.clone()).unwrap_or_default()),
        );
        let event = Event {
            seq: 0,
            kind: "run_accepted".into(),
            elapsed_ms: 0,
            data,
        };
        insert_event(&transaction, &admission.run_id, &event)?;
        let record = get_run(&transaction, &admission.owner_id, &admission.run_id)?
            .ok_or_else(|| error("storage_invariant"))?;
        transaction.commit()?;
        self.reservations.insert(record.run_id.clone(), reservation);
        Ok(Response::Admitted {
            record: Box::new(record),
            created: true,
        })
    }

    fn health(&self) -> Result<Response> {
        let pages: u64 = self
            .connection
            .pragma_query_value(None, "page_count", |row| row.get::<_, i64>(0))
            .and_then(|value| u64::try_from(value).map_err(|_| rusqlite::Error::InvalidQuery))?;
        let free: u64 = self
            .connection
            .pragma_query_value(None, "freelist_count", |row| row.get::<_, i64>(0))
            .and_then(|value| u64::try_from(value).map_err(|_| rusqlite::Error::InvalidQuery))?;
        let size: u64 = self
            .connection
            .pragma_query_value(None, "page_size", |row| row.get::<_, i64>(0))
            .and_then(|value| u64::try_from(value).map_err(|_| rusqlite::Error::InvalidQuery))?;
        let runs: i64 = self
            .connection
            .query_row("SELECT count(*) FROM runs", [], |row| row.get(0))?;
        let mut wal = self.path.as_os_str().to_os_string();
        wal.push("-wal");
        let wal_bytes = match std::fs::metadata(PathBuf::from(wal)) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(_) => return Err(error("storage_metadata")),
        };
        Ok(Response::Health {
            database_bytes: pages.saturating_sub(free).saturating_mul(size),
            wal_bytes,
            reserved_bytes: self.reservations.values().sum(),
            runs: runs as u64,
        })
    }

    fn append(&mut self, owner: &str, run: &str, event: Event, phase: Option<&str>) -> Result<()> {
        let transaction = self.connection.transaction()?;
        let record =
            get_run(&transaction, owner, run)?.ok_or_else(|| error("storage_not_found"))?;
        if terminal_phase(&record.phase) {
            return Err(error("storage_terminal_immutable"));
        }
        if !matches!(
            event.kind.as_str(),
            "run_started"
                | "model_planned"
                | "model_finished"
                | "tool_planned"
                | "tool_finished"
                | "cancel_requested"
        ) {
            return Err(error("storage_invalid_event"));
        }
        if matches!(event.kind.as_str(), "model_planned" | "tool_planned")
            && record.phase != "running"
        {
            return Err(error("storage_invalid_transition"));
        }
        if matches!(event.kind.as_str(), "model_finished" | "tool_finished")
            && !matches!(record.phase.as_str(), "running" | "cancelling")
        {
            return Err(error("storage_invalid_transition"));
        }
        if let Some(next) = phase {
            if !matches!(
                (record.phase.as_str(), next),
                ("queued", "running") | ("queued" | "running", "cancelling")
            ) {
                return Err(error("storage_invalid_transition"));
            }
            if (next == "running" && event.kind != "run_started")
                || (next == "cancelling" && event.kind != "cancel_requested")
            {
                return Err(error("storage_invalid_transition"));
            }
        } else if event.kind == "run_started" {
            return Err(error("storage_invalid_transition"));
        }
        validate_capture(&record.capture, &event.data)?;
        insert_event(&transaction, run, &event)?;
        if let Some(next) = phase {
            transaction.execute(
                "UPDATE runs SET phase=?1 WHERE owner_id=?2 AND run_id=?3",
                params![next, owner, run],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn finish(
        &mut self,
        owner: &str,
        run: &str,
        mut event: Event,
        terminal: Terminal,
    ) -> Result<()> {
        let transaction = self.connection.transaction()?;
        let record =
            get_run(&transaction, owner, run)?.ok_or_else(|| error("storage_not_found"))?;
        if terminal_phase(&record.phase) {
            return Err(error("storage_terminal_immutable"));
        }
        if event.kind != "run_finished" {
            return Err(error("storage_invalid_event"));
        }
        validate_terminal(&transaction, &record, &terminal)?;
        validate_capture(&record.capture, &event.data)?;
        let object = event
            .data
            .as_object_mut()
            .ok_or_else(|| error("storage_invalid_event"))?;
        object.insert("phase".into(), json!(terminal.phase));
        object.insert(
            "acceptance_status".into(),
            json!(terminal.acceptance_status),
        );
        object.insert("receipt".into(), terminal.receipt.clone());
        object.insert("candidate_sha256".into(), json!(terminal.result_sha256));
        insert_event(&transaction, run, &event)?;
        transaction.execute("UPDATE runs SET phase=?1,terminal_reason=?2,result_json=?3,result_sha256=?4,acceptance_status=?5,acceptance_json=?6 WHERE owner_id=?7 AND run_id=?8",
            params![terminal.phase, terminal.reason, terminal.result.as_ref().map(serde_json::to_string).transpose()?, terminal.result_sha256, terminal.acceptance_status, serde_json::to_string(&terminal.receipt)?, owner, run])?;
        transaction.commit()?;
        Ok(())
    }

    fn list(&self, owner: &str, after: Option<&RunCursor>, limit: usize) -> Result<Vec<RunRecord>> {
        page_limit(limit)?;
        let mut statement = self.connection.prepare(&format!("SELECT {RUN_COLUMNS} FROM runs WHERE owner_id=?1 AND (?2 IS NULL OR created_unix_ms>?2 OR (created_unix_ms=?2 AND run_id>?3)) ORDER BY created_unix_ms,run_id LIMIT ?4"))?;
        let rows = statement.query_map(
            params![
                owner,
                after.map(|c| c.created_unix_ms),
                after.map(|c| &c.run_id),
                limit as i64
            ],
            record_row,
        )?;
        let mut result = Vec::new();
        let mut bytes = 0;
        for row in rows {
            let row = row?;
            let size = serde_json::to_vec(&row)?.len();
            if bytes + size > MAX_PAGE_BYTES {
                break;
            }
            bytes += size;
            result.push(row);
        }
        Ok(result)
    }

    fn recover(&mut self) -> Result<()> {
        let transaction = self.connection.transaction()?;
        // Process one bounded run at a time, retaining a single transaction so
        // recovery is atomic without loading every retained run into memory.
        loop {
            let record = transaction.query_row(&format!("SELECT {RUN_COLUMNS} FROM runs WHERE phase IN ('queued','running','cancelling') ORDER BY run_id LIMIT 1"), [], record_row).optional()?;
            let Some(record) = record else {
                break;
            };
            let ids = criterion_ids(&transaction, &record.run_id)?;
            let status = if record.task_mode == "checked" {
                "inconclusive"
            } else {
                "unchecked"
            };
            let receipt = json!({"owner_id":record.owner_id,"run_id":record.run_id,"status":status,"reason":"controller_interrupted","contract":contract_value(&record),"candidate_sha256":null,"criteria":ids.iter().map(|id| json!({"id":id,"status":"inconclusive","code":"controller_interrupted"})).collect::<Vec<_>>(),"scope":if record.task_mode == "checked" {"frozen contract; assessment interrupted"} else {"unchecked freeform task"},"duration_ms":0});
            if serde_json::to_vec(&receipt)?.len() > MAX_RECEIPT_BYTES {
                return Err(error("storage_receipt_too_large"));
            }
            let seq = next_sequence(&transaction, &record.run_id)?;
            let elapsed: i64 = transaction.query_row(
                "SELECT COALESCE(MAX(elapsed_ms),0) FROM events WHERE run_id=?1",
                [&record.run_id],
                |r| r.get(0),
            )?;
            let event = Event {
                seq,
                kind: "recovery_interrupted".into(),
                elapsed_ms: elapsed as u64,
                data: json!({"phase":"interrupted","acceptance_status":status,"receipt":receipt,"pending_effect_outcome":"unknown"}),
            };
            insert_event(&transaction, &record.run_id, &event)?;
            transaction.execute("UPDATE runs SET phase='interrupted',terminal_reason='controller_interrupted',acceptance_status=?1,acceptance_json=?2 WHERE run_id=?3", params![status,serde_json::to_string(&receipt)?,record.run_id])?;
        }
        transaction.commit()?;
        Ok(())
    }
}

fn record_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunRecord> {
    fn value(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<Value>> {
        let text: Option<String> = row.get(index)?;
        text.map(|s| {
            serde_json::from_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    index,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })
        .transpose()
    }
    Ok(RunRecord {
        run_id: row.get(0)?,
        owner_id: row.get(1)?,
        workspace_id: row.get(2)?,
        model_profile_id: row.get(3)?,
        task_mode: row.get(4)?,
        task_profile_id: row.get(5)?,
        task_profile_version: row.get(6)?,
        task_spec_sha256: row.get(7)?,
        checker_id: row.get(8)?,
        checker_version: row.get(9)?,
        capture: row.get(10)?,
        phase: row.get(11)?,
        acceptance_status: row.get(12)?,
        created_unix_ms: row.get(13)?,
        policy_version: row.get(14)?,
        submission_sha256: row.get(15)?,
        idempotency_key: row.get(16)?,
        terminal_reason: row.get(17)?,
        result: value(row, 18)?,
        result_sha256: row.get(19)?,
        receipt: value(row, 20)?,
    })
}

fn get_run(connection: &Connection, owner: &str, run: &str) -> Result<Option<RunRecord>> {
    Ok(connection
        .query_row(
            &format!("SELECT {RUN_COLUMNS} FROM runs WHERE owner_id=?1 AND run_id=?2"),
            params![owner, run],
            record_row,
        )
        .optional()?)
}

fn lookup(
    connection: &Connection,
    owner: &str,
    key: &str,
    digest: &str,
) -> Result<Option<RunRecord>> {
    if key.is_empty() || key.len() > 128 || !valid_digest(digest) {
        return Err(error("storage_invalid_lookup"));
    }
    let record = connection
        .query_row(
            &format!("SELECT {RUN_COLUMNS} FROM runs WHERE owner_id=?1 AND idempotency_key=?2"),
            params![owner, key],
            record_row,
        )
        .optional()?;
    if record
        .as_ref()
        .is_some_and(|r| r.submission_sha256 != digest)
    {
        return Err(error("storage_idempotency_conflict"));
    }
    Ok(record)
}

fn next_sequence(connection: &Connection, run: &str) -> Result<u64> {
    let value: i64 = connection.query_row(
        "SELECT COALESCE(MAX(seq),-1)+1 FROM events WHERE run_id=?1",
        [run],
        |r| r.get(0),
    )?;
    u64::try_from(value).map_err(|_| error("storage_sequence_overflow"))
}

fn insert_event(transaction: &Transaction<'_>, run: &str, event: &Event) -> Result<()> {
    if event.seq > i64::MAX as u64
        || event.elapsed_ms > i64::MAX as u64
        || event.seq != next_sequence(transaction, run)?
    {
        return Err(error("storage_event_sequence"));
    }
    if serde_json::to_vec(event)?.len() > MAX_EVENT_BYTES || !event.data.is_object() {
        return Err(error("storage_event_too_large"));
    }
    let last: Option<i64> = transaction
        .query_row(
            "SELECT elapsed_ms FROM events WHERE run_id=?1 ORDER BY seq DESC LIMIT 1",
            [run],
            |r| r.get(0),
        )
        .optional()?;
    if last.is_some_and(|v| event.elapsed_ms < v as u64) {
        return Err(error("storage_event_time"));
    }
    transaction.execute("INSERT INTO events (run_id,seq,schema_version,kind,elapsed_ms,data_json) VALUES (?1,?2,2,?3,?4,?5)", params![run,event.seq as i64,event.kind,event.elapsed_ms as i64,serde_json::to_string(&event.data)?])?;
    Ok(())
}

fn read_events(
    connection: &Connection,
    owner: &str,
    run: &str,
    after: Option<u64>,
    limit: usize,
) -> Result<Vec<Event>> {
    page_limit(limit)?;
    if after.is_some_and(|s| s > i64::MAX as u64) {
        return Err(error("storage_invalid_cursor"));
    }
    let mut statement = connection.prepare("SELECT e.seq,e.kind,e.elapsed_ms,e.data_json FROM events e JOIN runs r ON r.run_id=e.run_id WHERE r.owner_id=?1 AND r.run_id=?2 AND (?3 IS NULL OR e.seq>?3) ORDER BY e.seq LIMIT ?4")?;
    let rows = statement.query_map(
        params![owner, run, after.map(|v| v as i64), limit as i64],
        |row| {
            let data: String = row.get(3)?;
            let value = serde_json::from_str(&data).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Event {
                seq: u64::try_from(row.get::<_, i64>(0)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Integer,
                        Box::new(e),
                    )
                })?,
                kind: row.get(1)?,
                elapsed_ms: u64::try_from(row.get::<_, i64>(2)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Integer,
                        Box::new(e),
                    )
                })?,
                data: value,
            })
        },
    )?;
    let mut result = Vec::new();
    let mut bytes = 0;
    for row in rows {
        let event = row?;
        let size = serde_json::to_vec(&event)?.len();
        if bytes + size > MAX_PAGE_BYTES {
            break;
        }
        bytes += size;
        result.push(event);
    }
    Ok(result)
}

fn page_limit(limit: usize) -> Result<()> {
    if limit == 0 || limit > MAX_PAGE_SIZE {
        return Err(error("storage_invalid_page_limit"));
    }
    Ok(())
}
fn terminal_phase(phase: &str) -> bool {
    matches!(
        phase,
        "completed" | "stopped" | "failed" | "cancelled" | "interrupted"
    )
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn candidate_digest(candidate: &str) -> String {
    Sha256::digest(candidate.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_admission(admission: &Admission) -> Result<()> {
    if [
        &admission.run_id,
        &admission.owner_id,
        &admission.workspace_id,
        &admission.model_profile_id,
        &admission.policy_version,
    ]
    .iter()
    .any(|s| !valid_id(s))
        || admission.created_unix_ms < 0
        || !valid_digest(&admission.submission_sha256)
        || admission.idempotency_key.as_ref().is_some_and(|k| {
            k.is_empty()
                || k.len() > 128
                || !k.is_ascii()
                || k.bytes().any(|b| b.is_ascii_control())
        })
    {
        return Err(error("storage_invalid_admission"));
    }
    validate_capture(&admission.capture, &admission.data)?;
    if !admission.data.is_object() {
        return Err(error("storage_invalid_admission"));
    }
    if let Some(task) = &admission.task
        && (!valid_id(&task.profile_id)
            || task.checker_id != "file_fields_v1"
            || task.checker_version != "1"
            || task
                .profile_version
                .parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .is_none()
            || !valid_digest(&task.spec_sha256)
            || !(1..=4).contains(&task.criterion_ids.len())
            || task.criterion_ids.iter().any(|id| !valid_id(id))
            || task
                .criterion_ids
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != task.criterion_ids.len())
    {
        return Err(error("storage_invalid_task"));
    }
    Ok(())
}

fn validate_capture(capture: &str, data: &Value) -> Result<()> {
    if !matches!(capture, "metadata" | "replay") {
        return Err(error("storage_invalid_capture"));
    }
    if capture == "metadata" && sensitive_metadata(data) {
        return Err(error("storage_private_payload_in_metadata"));
    }
    Ok(())
}
fn sensitive_metadata(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            matches!(
                key.as_str(),
                "replay"
                    | "prompt"
                    | "instructions"
                    | "messages"
                    | "content"
                    | "body"
                    | "arguments"
                    | "raw_response"
                    | "observation"
            ) || sensitive_metadata(value)
        }),
        Value::Array(values) => values.iter().any(sensitive_metadata),
        _ => false,
    }
}

fn criterion_ids(connection: &Connection, run: &str) -> Result<Vec<String>> {
    let data: String = connection.query_row(
        "SELECT data_json FROM events WHERE run_id=?1 AND seq=0 AND kind='run_accepted'",
        [run],
        |r| r.get(0),
    )?;
    let value: Value = serde_json::from_str(&data)?;
    let ids: Vec<String> = serde_json::from_value(
        value
            .get("required_criterion_ids")
            .cloned()
            .ok_or_else(|| error("storage_missing_contract"))?,
    )?;
    if ids.len() > 4 || ids.iter().any(|id| !valid_id(id)) {
        return Err(error("storage_missing_contract"));
    }
    Ok(ids)
}
fn contract_value(record: &RunRecord) -> Value {
    if record.task_mode == "freeform" {
        return Value::Null;
    }
    json!({"profile_id":record.task_profile_id,"profile_version":record.task_profile_version,"checker_id":record.checker_id,"checker_version":record.checker_version,"spec_sha256":record.task_spec_sha256})
}

fn validate_terminal(
    connection: &Connection,
    record: &RunRecord,
    terminal: &Terminal,
) -> Result<()> {
    if terminal.phase == "completed" && record.phase != "running" {
        return Err(error("storage_invalid_transition"));
    }
    if !terminal_phase(&terminal.phase)
        || terminal.reason.is_empty()
        || terminal.reason.len() > 256
        || serde_json::to_vec(&terminal.receipt)?.len() > MAX_RECEIPT_BYTES
    {
        return Err(error("storage_invalid_terminal"));
    }
    let valid_status = if record.task_mode == "freeform" {
        terminal.acceptance_status == "unchecked"
    } else if terminal.phase == "completed" {
        matches!(
            terminal.acceptance_status.as_str(),
            "passed" | "failed" | "inconclusive"
        )
    } else {
        terminal.acceptance_status == "inconclusive"
    };
    if !valid_status {
        return Err(error("storage_invalid_terminal"));
    }
    match (&terminal.result, &terminal.result_sha256) {
        (Some(result), Some(digest)) => {
            let candidate = result
                .get("candidate")
                .and_then(Value::as_str)
                .ok_or_else(|| error("storage_missing_candidate"))?;
            if candidate.len() > MAX_CANDIDATE_BYTES
                || serde_json::to_vec(result)?.len() > MAX_RESULT_BYTES
                || candidate_digest(candidate) != *digest
                || (terminal.phase == "completed" && candidate.trim().is_empty())
            {
                return Err(error("storage_invalid_candidate"));
            }
        }
        (None, None) if terminal.phase != "completed" => {}
        _ => return Err(error("storage_missing_candidate")),
    }
    let receipt = &terminal.receipt;
    if receipt.get("owner_id") != Some(&json!(record.owner_id))
        || receipt.get("run_id") != Some(&json!(record.run_id))
        || receipt.get("status") != Some(&json!(terminal.acceptance_status))
        || receipt.get("candidate_sha256") != Some(&json!(terminal.result_sha256))
        || receipt.get("contract") != Some(&contract_value(record))
    {
        return Err(error("storage_receipt_mismatch"));
    }
    let required = criterion_ids(connection, &record.run_id)?;
    let actual = receipt
        .get("criteria")
        .and_then(Value::as_array)
        .ok_or_else(|| error("storage_receipt_mismatch"))?;
    let ids: Option<Vec<&str>> = actual
        .iter()
        .map(|c| c.get("id").and_then(Value::as_str))
        .collect();
    let Some(ids) = ids else {
        return Err(error("storage_receipt_mismatch"));
    };
    if ids.len() != required.len()
        || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
        || required.iter().any(|id| !ids.contains(&id.as_str()))
    {
        return Err(error("storage_receipt_mismatch"));
    }
    if terminal.acceptance_status == "passed"
        && actual
            .iter()
            .any(|c| c.get("status").and_then(Value::as_str) != Some("passed"))
    {
        return Err(error("storage_receipt_mismatch"));
    }
    Ok(())
}

pub fn freeform_receipt(owner: &str, run: &str, reason: &str, digest: Option<&str>) -> Value {
    json!({"owner_id":owner,"run_id":run,"status":"unchecked","reason":reason,"contract":null,"candidate_sha256":digest,"criteria":[],"scope":"unchecked freeform task","duration_ms":0})
}

#[derive(Clone, Copy, Debug)]
pub struct QueueLimits {
    pub commands: usize,
    pub bytes: usize,
    pub max_waiters: usize,
}
impl Default for QueueLimits {
    fn default() -> Self {
        Self {
            commands: 64,
            bytes: 8 * 1024 * 1024,
            max_waiters: 16,
        }
    }
}
struct Shared {
    accepting: AtomicBool,
    count: Arc<Semaphore>,
    bytes: Arc<Semaphore>,
    waiters: Arc<Semaphore>,
    limits: QueueLimits,
    changes: watch::Sender<u64>,
}
struct Envelope {
    command: Command,
    reply: oneshot::Sender<Result<Response>>,
    _count: OwnedSemaphorePermit,
    _bytes: OwnedSemaphorePermit,
}
enum Inbox {
    Work(Box<Envelope>),
    Shutdown,
}

#[derive(Clone)]
pub struct StorageClient {
    sender: mpsc::Sender<Inbox>,
    shared: Arc<Shared>,
}
pub struct Reservation {
    client: StorageClient,
    bytes: usize,
    count: OwnedSemaphorePermit,
    byte_permit: OwnedSemaphorePermit,
}
pub struct PendingAck {
    receiver: oneshot::Receiver<Result<Response>>,
}
pub struct Storage {
    client: StorageClient,
    thread: Option<JoinHandle<()>>,
}

impl Storage {
    /// Blocking startup handshake; use outside an async runtime worker.
    pub fn start(path: PathBuf, limits: QueueLimits) -> Result<Self> {
        Self::start_with_policy(path, limits, RetentionPolicy::default())
    }

    pub fn start_with_policy(
        path: PathBuf,
        limits: QueueLimits,
        policy: RetentionPolicy,
    ) -> Result<Self> {
        policy.validate()?;
        if limits.commands == 0
            || limits.bytes == 0
            || limits.bytes > u32::MAX as usize
            || limits.max_waiters == 0
            || limits.commands > Semaphore::MAX_PERMITS
            || limits.max_waiters > Semaphore::MAX_PERMITS
        {
            return Err(error("storage_invalid_queue_limits"));
        }
        let (sender, mut receiver) = mpsc::channel(limits.commands);
        let shared = Arc::new(Shared {
            accepting: AtomicBool::new(true),
            count: Arc::new(Semaphore::new(limits.commands)),
            bytes: Arc::new(Semaphore::new(limits.bytes)),
            waiters: Arc::new(Semaphore::new(limits.max_waiters)),
            limits,
            changes: watch::channel(0).0,
        });
        let worker_shared = Arc::clone(&shared);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("kinesin-sqlite".into())
            .spawn(move || {
                let mut store = match Store::open_with_policy(&path, policy) {
                    Ok(store) => {
                        let _ = ready_tx.send(Ok(()));
                        store
                    }
                    Err(problem) => {
                        worker_shared.accepting.store(false, Ordering::Release);
                        let _ = ready_tx.send(Err(problem));
                        return;
                    }
                };
                while let Some(message) = receiver.blocking_recv() {
                    match message {
                        Inbox::Shutdown => {
                            worker_shared.accepting.store(false, Ordering::Release);
                            receiver.close();
                        }
                        Inbox::Work(envelope) => {
                            let Envelope {
                                command,
                                reply,
                                _count,
                                _bytes,
                            } = *envelope;
                            let changes_projection = matches!(
                                &command,
                                Command::Admit(_) | Command::Append { .. } | Command::Finish { .. }
                            );
                            let result = store.execute(command);
                            if changes_projection && result.is_ok() {
                                worker_shared
                                    .changes
                                    .send_modify(|revision| *revision = revision.wrapping_add(1));
                            }
                            if result.as_ref().err().is_some_and(|e| {
                                matches!(
                                    e.code,
                                    "storage_database"
                                        | "storage_invariant"
                                        | "storage_missing_contract"
                                )
                            }) {
                                worker_shared.accepting.store(false, Ordering::Release);
                                receiver.close();
                                let _ = reply.send(result);
                                drop((_count, _bytes));
                                while let Some(message) = receiver.blocking_recv() {
                                    if let Inbox::Work(envelope) = message {
                                        let _ = envelope.reply.send(Err(error("storage_closed")));
                                    }
                                }
                                break;
                            }
                            // The permits remain owned across the complete transaction,
                            // even if the original waiter already discarded its receiver.
                            let _ = reply.send(result);
                            drop((_count, _bytes));
                        }
                    }
                }
                worker_shared.accepting.store(false, Ordering::Release);
            })
            .map_err(|_| error("storage_thread_start"))?;
        match ready_rx.recv().map_err(|_| error("storage_thread_start"))? {
            Ok(()) => Ok(Self {
                client: StorageClient { sender, shared },
                thread: Some(thread),
            }),
            Err(problem) => {
                let _ = thread.join();
                Err(problem)
            }
        }
    }
    pub fn client(&self) -> StorageClient {
        self.client.clone()
    }
    pub async fn shutdown(mut self) -> Result<()> {
        self.client.shared.accepting.store(false, Ordering::Release);
        let _ = self.client.sender.send(Inbox::Shutdown).await;
        if let Some(thread) = self.thread.take() {
            tokio::task::spawn_blocking(move || thread.join())
                .await
                .map_err(|_| error("storage_join"))?
                .map_err(|_| error("storage_thread_panicked"))?;
        }
        Ok(())
    }
}

impl StorageClient {
    /// Subscribe before durable catch-up. Notifications are coalesced wakeups,
    /// contain no owner data, and never replace owner-scoped event queries.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.shared.changes.subscribe()
    }
    pub fn is_accepting(&self) -> bool {
        self.shared.accepting.load(Ordering::Acquire) && !self.sender.is_closed()
    }
    pub fn outstanding(&self) -> (usize, usize) {
        (
            self.shared.limits.commands - self.shared.count.available_permits(),
            self.shared.limits.bytes - self.shared.bytes.available_permits(),
        )
    }
    pub async fn reserve(&self, bytes: usize, deadline: Instant) -> Result<Reservation> {
        if !self.is_accepting() {
            return Err(error("storage_closed"));
        }
        if bytes == 0 || bytes > self.shared.limits.bytes {
            return Err(error("storage_command_too_large"));
        }
        let _waiter = self
            .shared
            .waiters
            .clone()
            .try_acquire_owned()
            .map_err(|_| error("storage_waiters_full"))?;
        let count = timeout_at(deadline, self.shared.count.clone().acquire_owned())
            .await
            .map_err(|_| error("storage_admission_timeout"))?
            .map_err(|_| error("storage_closed"))?;
        let byte_permit = timeout_at(
            deadline,
            self.shared.bytes.clone().acquire_many_owned(bytes as u32),
        )
        .await
        .map_err(|_| error("storage_admission_timeout"))?
        .map_err(|_| error("storage_closed"))?;
        if !self.is_accepting() || Instant::now() >= deadline {
            return Err(error("storage_closed_or_expired"));
        }
        Ok(Reservation {
            client: self.clone(),
            bytes,
            count,
            byte_permit,
        })
    }
    pub async fn execute(&self, command: Command, deadline: Instant) -> Result<Response> {
        let size = command.encoded_len()?;
        self.reserve(size, deadline)
            .await?
            .submit(command)?
            .wait()
            .await
    }

    /// An already-admitted run retains this wait through stop and settlement.
    /// Its controller bounds the number of owners before this future exists.
    /// Unlike a new request, it must not abandon its terminal record because
    /// the inbox or its waiter slots are temporarily occupied.
    pub(crate) async fn reserve_owned(&self, bytes: usize) -> Result<Reservation> {
        if !self.is_accepting() {
            return Err(error("storage_closed"));
        }
        if bytes == 0 || bytes > self.shared.limits.bytes {
            return Err(error("storage_command_too_large"));
        }
        let _waiter = self
            .shared
            .waiters
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| error("storage_closed"))?;
        let count = self
            .shared
            .count
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| error("storage_closed"))?;
        let byte_permit = self
            .shared
            .bytes
            .clone()
            .acquire_many_owned(bytes as u32)
            .await
            .map_err(|_| error("storage_closed"))?;
        if !self.is_accepting() {
            return Err(error("storage_closed"));
        }
        Ok(Reservation {
            client: self.clone(),
            bytes,
            count,
            byte_permit,
        })
    }
}
impl Reservation {
    /// Synchronous transfer is the terminal cancellation arbitration boundary.
    pub fn submit(self, command: Command) -> Result<PendingAck> {
        if !self.client.is_accepting() {
            return Err(error("storage_closed"));
        }
        if command.encoded_len()? > self.bytes {
            return Err(error("storage_reservation_too_small"));
        }
        let command = command.prepare()?;
        let (reply, receiver) = oneshot::channel();
        let envelope = Envelope {
            command,
            reply,
            _count: self.count,
            _bytes: self.byte_permit,
        };
        self.client
            .sender
            .try_send(Inbox::Work(Box::new(envelope)))
            .map_err(|_| error("storage_closed"))?;
        Ok(PendingAck { receiver })
    }
}
impl PendingAck {
    /// No implicit timeout/retry: the controller must retain this operation's
    /// ownership when its ordinary response or settlement deadline expires.
    pub async fn wait(self) -> Result<Response> {
        self.receiver.await.map_err(|_| error("storage_ack_lost"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        directory: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let directory =
                std::env::temp_dir().join(format!("kinesin-storage-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&directory).unwrap();
            Self { directory }
        }
        fn path(&self) -> PathBuf {
            self.directory.join("kinesin.sqlite")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
    fn admission(run: &str, owner: &str) -> Admission {
        Admission {
            run_id: run.into(),
            owner_id: owner.into(),
            workspace_id: "practice".into(),
            model_profile_id: "local".into(),
            task: None,
            capture: "metadata".into(),
            created_unix_ms: 1,
            policy_version: "1".into(),
            submission_sha256: candidate_digest("submission"),
            idempotency_key: None,
            data: json!({"request_bytes":10}),
        }
    }
    fn event(seq: u64, kind: &str) -> Event {
        Event {
            seq,
            kind: kind.into(),
            elapsed_ms: seq,
            data: json!({}),
        }
    }
    fn terminal(run: &str, owner: &str) -> Terminal {
        let digest = candidate_digest(" answer\n");
        Terminal {
            phase: "completed".into(),
            reason: "answer".into(),
            result: Some(json!({"candidate":" answer\n"})),
            result_sha256: Some(digest.clone()),
            acceptance_status: "unchecked".into(),
            receipt: freeform_receipt(owner, run, "answer", Some(&digest)),
        }
    }
    fn start_run(store: &mut Store, run: &str, owner: &str) {
        store
            .execute(Command::Admit(admission(run, owner)))
            .unwrap();
        store
            .execute(Command::Append {
                owner_id: owner.into(),
                run_id: run.into(),
                event: event(1, "run_started"),
                phase: Some("running".into()),
            })
            .unwrap();
    }

    #[test]
    fn exact_schema_and_engine_settings_are_active() {
        let fixture = Fixture::new();
        let store = Store::open(&fixture.path()).unwrap();
        assert!(rusqlite::version_number() >= 3_051_003);
        let version: i64 = store
            .connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        let sync: i64 = store
            .connection
            .pragma_query_value(None, "synchronous", |r| r.get(0))
            .unwrap();
        let wal: String = store
            .connection
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        assert_eq!((version, sync, wal.as_str()), (2, 2, "wal"));
    }

    #[test]
    fn retention_keeps_active_and_young_runs_and_ends_deduplication_only_after_its_window() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        let mut old = admission("old", "alice");
        old.created_unix_ms = 1;
        old.idempotency_key = Some("old-key".into());
        let fingerprint = old.submission_sha256.clone();
        store.execute(Command::Admit(old)).unwrap();
        store
            .execute(Command::Append {
                owner_id: "alice".into(),
                run_id: "old".into(),
                event: event(1, "run_started"),
                phase: Some("running".into()),
            })
            .unwrap();
        store
            .execute(Command::Finish {
                owner_id: "alice".into(),
                run_id: "old".into(),
                event: event(2, "run_finished"),
                terminal: terminal("old", "alice"),
            })
            .unwrap();
        start_run(&mut store, "active", "alice");
        assert!(matches!(
            store
                .execute(Command::Retain {
                    now_unix_ms: 86_400_000,
                    limit: 10
                })
                .unwrap(),
            Response::Retained(0)
        ));
        assert!(matches!(
            store
                .execute(Command::Lookup {
                    owner_id: "alice".into(),
                    key: "old-key".into(),
                    submission_sha256: fingerprint.clone()
                })
                .unwrap(),
            Response::Run(Some(_))
        ));
        assert!(matches!(
            store
                .execute(Command::Retain {
                    now_unix_ms: 86_400_001,
                    limit: 10
                })
                .unwrap(),
            Response::Retained(1)
        ));
        assert!(matches!(
            store
                .execute(Command::Get {
                    owner_id: "alice".into(),
                    run_id: "active".into()
                })
                .unwrap(),
            Response::Run(Some(_))
        ));
        assert!(matches!(
            store
                .execute(Command::Lookup {
                    owner_id: "alice".into(),
                    key: "old-key".into(),
                    submission_sha256: fingerprint
                })
                .unwrap(),
            Response::Run(None)
        ));
        assert!(
            read_events(&store.connection, "alice", "old", None, 100)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn count_and_byte_admission_caps_preserve_original_idempotent_retries() {
        let fixture = Fixture::new();
        let policy = RetentionPolicy {
            max_runs: 1,
            ..RetentionPolicy::default()
        };
        let mut store = Store::open_with_policy(&fixture.path(), policy).unwrap();
        let mut first = admission("first", "alice");
        first.idempotency_key = Some("key".into());
        store.execute(Command::Admit(first.clone())).unwrap();
        assert_eq!(
            store
                .execute(Command::Admit(admission("other", "alice")))
                .unwrap_err()
                .code,
            "storage_admission_capacity"
        );
        first.run_id = "retry-proposed-id".into();
        assert!(matches!(
            store.execute(Command::Admit(first)).unwrap(),
            Response::Admitted { created: false, .. }
        ));
        drop(store);
        let mut store = Store::open_with_policy(
            &fixture.path(),
            RetentionPolicy {
                max_bytes: 4 * 1024 * 1024,
                ..RetentionPolicy::default()
            },
        )
        .unwrap();
        assert_eq!(
            store
                .execute(Command::Admit(admission("large", "alice")))
                .unwrap_err()
                .code,
            "storage_admission_capacity"
        );
    }

    #[test]
    fn sqlite_backup_restores_terminal_receipts_and_interrupts_unfinished_runs() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        start_run(&mut store, "done", "alice");
        store
            .execute(Command::Finish {
                owner_id: "alice".into(),
                run_id: "done".into(),
                event: event(2, "run_finished"),
                terminal: terminal("done", "alice"),
            })
            .unwrap();
        start_run(&mut store, "active", "alice");
        let directory = fixture.path().parent().unwrap().join("restored");
        std::fs::create_dir(&directory).unwrap();
        let destination = directory.join("backup.sqlite");
        store
            .execute(Command::Backup {
                destination: destination.clone(),
            })
            .unwrap();
        let mut restored = Store::open(&destination).unwrap();
        let Response::Run(Some(done)) = restored
            .execute(Command::Get {
                owner_id: "alice".into(),
                run_id: "done".into(),
            })
            .unwrap()
        else {
            panic!("missing backed up result")
        };
        assert_eq!(done.result, terminal("done", "alice").result);
        assert_eq!(done.receipt, Some(terminal("done", "alice").receipt));
        let Response::Run(Some(active)) = restored
            .execute(Command::Get {
                owner_id: "alice".into(),
                run_id: "active".into(),
            })
            .unwrap()
        else {
            panic!("missing active backup")
        };
        assert_eq!(active.phase, "interrupted");
        assert_eq!(
            store
                .execute(Command::Backup { destination })
                .unwrap_err()
                .code,
            "storage_invalid_backup_destination"
        );
    }

    #[test]
    fn accounting_includes_generated_event_and_projection_bytes() {
        let command = Command::Finish {
            owner_id: "alice".into(),
            run_id: "run".into(),
            event: event(2, "run_finished"),
            terminal: terminal("run", "alice"),
        };
        let original = serde_json::to_vec(&command).unwrap().len();
        let actual = command.encoded_len().unwrap();
        let prepared = command.prepare().unwrap();
        assert!(actual > original);
        assert_eq!(actual, serde_json::to_vec(&prepared).unwrap().len());
        let mut too_large = event(2, "model_finished");
        too_large.data = json!({"replay":{"observation":"x".repeat(MAX_EVENT_BYTES)}});
        assert_eq!(
            Command::Append {
                owner_id: "alice".into(),
                run_id: "run".into(),
                event: too_large,
                phase: None
            }
            .encoded_len()
            .unwrap_err()
            .code,
            "storage_serialized_limit"
        );
    }

    #[test]
    fn queued_runs_cannot_record_model_dispatch_or_completed_answers() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        store
            .execute(Command::Admit(admission("run", "alice")))
            .unwrap();
        assert_eq!(
            store
                .execute(Command::Append {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(1, "model_planned"),
                    phase: None
                })
                .unwrap_err()
                .code,
            "storage_invalid_transition"
        );
        assert_eq!(
            store
                .execute(Command::Finish {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(1, "run_finished"),
                    terminal: terminal("run", "alice")
                })
                .unwrap_err()
                .code,
            "storage_invalid_transition"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn waiter_population_is_bounded_before_waiting() {
        let fixture = Fixture::new();
        let storage = Storage::start(
            fixture.path(),
            QueueLimits {
                commands: 1,
                bytes: 4096,
                max_waiters: 1,
            },
        )
        .unwrap();
        let client = storage.client();
        let held = client
            .reserve(1, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        let waiting_client = client.clone();
        let waiting = tokio::spawn(async move {
            waiting_client
                .reserve(1, Instant::now() + Duration::from_secs(2))
                .await
        });
        let deadline = Instant::now() + Duration::from_secs(1);
        while client.shared.waiters.available_permits() != 0 {
            assert!(Instant::now() < deadline, "waiter did not begin waiting");
            tokio::task::yield_now().await;
        }
        assert_eq!(
            client
                .reserve(1, Instant::now() + Duration::from_secs(1))
                .await
                .err()
                .unwrap()
                .code,
            "storage_waiters_full"
        );
        drop(held);
        drop(waiting.await.unwrap().unwrap());
        storage.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn database_failure_closes_worker_admission() {
        let fixture = Fixture::new();
        let storage = Storage::start(fixture.path(), QueueLimits::default()).unwrap();
        let client = storage.client();
        let external = Connection::open(fixture.path()).unwrap();
        external.execute_batch("CREATE TRIGGER reject_admission BEFORE INSERT ON runs BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
        drop(external);
        let problem = client
            .execute(
                Command::Admit(admission("run", "alice")),
                Instant::now() + Duration::from_secs(1),
            )
            .await
            .unwrap_err();
        assert_eq!(problem.code, "storage_database");
        assert!(!client.is_accepting());
        assert_eq!(
            client
                .reserve(1, Instant::now() + Duration::from_secs(1))
                .await
                .err()
                .unwrap()
                .code,
            "storage_closed"
        );
        storage.shutdown().await.unwrap();
    }

    #[test]
    fn sqlite_full_rolls_back_admission_and_preserves_a_committed_receipt() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        start_run(&mut store, "committed", "alice");
        store
            .execute(Command::Finish {
                owner_id: "alice".into(),
                run_id: "committed".into(),
                event: event(2, "run_finished"),
                terminal: terminal("committed", "alice"),
            })
            .unwrap();
        let before = get_run(&store.connection, "alice", "committed")
            .unwrap()
            .unwrap();
        let before_events =
            read_events(&store.connection, "alice", "committed", None, 100).unwrap();
        let pages: i64 = store
            .connection
            .pragma_query_value(None, "page_count", |row| row.get(0))
            .unwrap();
        store
            .connection
            .pragma_update(None, "max_page_count", pages)
            .unwrap();
        let actual_limit: i64 = store
            .connection
            .pragma_query_value(None, "max_page_count", |row| row.get(0))
            .unwrap();
        assert_eq!(actual_limit, pages);
        let payload = "x".repeat(512 * 1024);
        // Observe the actual engine result rather than assuming a generic
        // storage error means SQLITE_FULL. This bounded probe also rolls back.
        {
            let transaction = store.connection.transaction().unwrap();
            let failed = transaction.execute(
                "INSERT INTO events (run_id,seq,schema_version,kind,elapsed_ms,data_json) VALUES ('committed',3,2,'fault_probe',3,?1)",
                [&payload],
            ).unwrap_err();
            assert_eq!(
                failed.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DiskFull)
            );
        }
        assert!(store.connection.is_autocommit());
        let mut oversized = admission("uncommitted", "alice");
        oversized.data = json!({"bounded_test_padding":payload});
        let command = Command::Admit(oversized);
        assert!(command.encoded_len().unwrap() < MAX_COMMAND_BYTES);
        assert_eq!(store.execute(command).unwrap_err().code, "storage_database");
        assert!(
            get_run(&store.connection, "alice", "uncommitted")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            get_run(&store.connection, "alice", "committed")
                .unwrap()
                .unwrap(),
            before
        );
        assert_eq!(
            read_events(&store.connection, "alice", "committed", None, 100).unwrap(),
            before_events
        );
        assert!(store.reservations.is_empty());
        drop(store);
        let reopened = Store::open(&fixture.path()).unwrap();
        assert_eq!(
            get_run(&reopened.connection, "alice", "committed")
                .unwrap()
                .unwrap(),
            before
        );
        assert!(
            get_run(&reopened.connection, "alice", "uncommitted")
                .unwrap()
                .is_none()
        );
        // max_page_count simulates SQLite capacity exhaustion. The host volume
        // was never filled; this does not claim physical disk/power-loss proof.
    }

    #[cfg(windows)]
    #[test]
    fn fresh_windows_write_denial_fails_storage_startup_before_model_dispatch() {
        use std::os::windows::ffi::OsStrExt;
        use std::ptr::null_mut;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
        use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
        use windows_sys::Win32::Storage::FileSystem::CreateDirectoryW;

        let fixture = Fixture::new();
        let path = fixture.directory.join("deny-new-files");
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Only this newly created empty test directory gets a DACL. Deny
        // file/subdirectory creation (0x2|0x4), while retaining cleanup rights.
        let sddl: Vec<u16> = "D:P(D;OICI;0x6;;;WD)(A;OICI;FA;;;WD)"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut descriptor = null_mut();
        // SAFETY: terminated synthetic SDDL and valid owned output pointer.
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    1,
                    &mut descriptor,
                    null_mut(),
                )
            },
            0
        );
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        // SAFETY: a fresh UUID child path and a live valid descriptor.
        let created = unsafe { CreateDirectoryW(wide.as_ptr(), &attributes) };
        // SAFETY: the conversion API returned this unique LocalAlloc allocation.
        unsafe { LocalFree(descriptor) };
        assert_ne!(created, 0);
        let denied = std::fs::write(path.join("probe"), b"synthetic").unwrap_err();
        assert_eq!(denied.kind(), std::io::ErrorKind::PermissionDenied);
        let model =
            crate::model::ModelClient::scripted([
                crate::core::ModelReply::Answer("unused".into()).into()
            ]);
        let result = Storage::start(path.join("kinesin.sqlite"), QueueLimits::default());
        let problem = match result {
            Err(problem) => problem,
            Ok(storage) => {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                runtime.block_on(storage.shutdown()).unwrap();
                panic!("storage started in a write-denied directory");
            }
        };
        assert_eq!(problem.code, "storage_lock_open");
        assert!(model.captured_requests().unwrap().is_empty());
        assert!(!path.join("kinesin.sqlite").exists());
        assert!(!path.join("controller.lock").exists());
        std::fs::remove_dir(path).unwrap();
    }

    #[test]
    fn result_receipt_and_terminal_event_survive_reopen_together() {
        let fixture = Fixture::new();
        let expected = terminal("run", "alice");
        {
            let mut store = Store::open(&fixture.path()).unwrap();
            start_run(&mut store, "run", "alice");
            store
                .execute(Command::Finish {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(2, "run_finished"),
                    terminal: expected.clone(),
                })
                .unwrap();
        }
        let store = Store::open(&fixture.path()).unwrap();
        let record = get_run(&store.connection, "alice", "run").unwrap().unwrap();
        assert_eq!(record.phase, "completed");
        assert_eq!(record.result, expected.result);
        assert_eq!(record.receipt, Some(expected.receipt));
        assert!(!record.task_accepted());
        let events = read_events(&store.connection, "alice", "run", None, 100).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[2].data["receipt"], record.receipt.unwrap());
    }

    #[test]
    fn failure_between_event_and_projection_rolls_back_both() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        start_run(&mut store, "run", "alice");
        store.connection.execute_batch("CREATE TEMP TRIGGER deny_terminal BEFORE UPDATE ON runs WHEN NEW.phase='completed' BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
        let result = store.execute(Command::Finish {
            owner_id: "alice".into(),
            run_id: "run".into(),
            event: event(2, "run_finished"),
            terminal: terminal("run", "alice"),
        });
        assert_eq!(result.unwrap_err().code, "storage_database");
        let record = get_run(&store.connection, "alice", "run").unwrap().unwrap();
        assert_eq!(record.phase, "running");
        assert!(record.result.is_none());
        assert!(record.receipt.is_none());
        assert_eq!(
            read_events(&store.connection, "alice", "run", None, 100)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn owner_filtering_covers_reads_mutations_and_idempotency() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        let mut accepted = admission("alice-run", "alice");
        accepted.idempotency_key = Some("key".into());
        store.execute(Command::Admit(accepted.clone())).unwrap();
        assert!(
            get_run(&store.connection, "bob", "alice-run")
                .unwrap()
                .is_none()
        );
        assert!(store.list("bob", None, 10).unwrap().is_empty());
        assert!(
            read_events(&store.connection, "bob", "alice-run", None, 10)
                .unwrap()
                .is_empty()
        );
        assert!(
            lookup(&store.connection, "bob", "key", &accepted.submission_sha256)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .execute(Command::Append {
                    owner_id: "bob".into(),
                    run_id: "alice-run".into(),
                    event: event(1, "run_started"),
                    phase: Some("running".into())
                })
                .unwrap_err()
                .code,
            "storage_not_found"
        );
        assert_eq!(
            store
                .execute(Command::Finish {
                    owner_id: "bob".into(),
                    run_id: "alice-run".into(),
                    event: event(1, "run_finished"),
                    terminal: terminal("alice-run", "bob")
                })
                .unwrap_err()
                .code,
            "storage_not_found"
        );
        let mut duplicate = accepted.clone();
        duplicate.run_id = "replacement".into();
        match store.execute(Command::Admit(duplicate)).unwrap() {
            Response::Admitted { record, created } => {
                assert!(!created);
                assert_eq!(record.run_id, "alice-run");
            }
            _ => panic!("expected admission response"),
        }
        let mut conflict = accepted;
        conflict.submission_sha256 = candidate_digest("different");
        assert_eq!(
            store.execute(Command::Admit(conflict)).unwrap_err().code,
            "storage_idempotency_conflict"
        );
    }

    #[test]
    fn metadata_rejects_private_observations_but_replay_retains_them_once() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        let mut input = admission("private", "alice");
        input.data =
            json!({"replay":{"instructions":"secret instructions","prompt":"private prompt"}});
        assert_eq!(
            store
                .execute(Command::Admit(input.clone()))
                .unwrap_err()
                .code,
            "storage_private_payload_in_metadata"
        );
        assert!(
            get_run(&store.connection, "alice", "private")
                .unwrap()
                .is_none()
        );
        input.capture = "replay".into();
        store.execute(Command::Admit(input)).unwrap();
        let events = read_events(&store.connection, "alice", "private", None, 100).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["replay"]["prompt"], "private prompt");
    }

    #[test]
    fn invalid_sequences_and_terminal_rewrites_cannot_change_history() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        start_run(&mut store, "run", "alice");
        assert_eq!(
            store
                .execute(Command::Append {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(9, "model_planned"),
                    phase: None
                })
                .unwrap_err()
                .code,
            "storage_event_sequence"
        );
        store
            .execute(Command::Finish {
                owner_id: "alice".into(),
                run_id: "run".into(),
                event: event(2, "run_finished"),
                terminal: terminal("run", "alice"),
            })
            .unwrap();
        assert_eq!(
            store
                .execute(Command::Finish {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(3, "run_finished"),
                    terminal: terminal("run", "alice")
                })
                .unwrap_err()
                .code,
            "storage_terminal_immutable"
        );
        assert_eq!(
            read_events(&store.connection, "alice", "run", None, 100)
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn receipt_binding_and_exact_candidate_hash_are_enforced() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        start_run(&mut store, "run", "alice");
        let mut bad = terminal("run", "alice");
        bad.receipt["status"] = json!("passed");
        assert_eq!(
            store
                .execute(Command::Finish {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(2, "run_finished"),
                    terminal: bad
                })
                .unwrap_err()
                .code,
            "storage_receipt_mismatch"
        );
        let mut bad = terminal("run", "alice");
        bad.result_sha256 = Some(candidate_digest("answer"));
        assert_eq!(
            store
                .execute(Command::Finish {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                    event: event(2, "run_finished"),
                    terminal: bad
                })
                .unwrap_err()
                .code,
            "storage_invalid_candidate"
        );
        assert_eq!(
            read_events(&store.connection, "alice", "run", None, 100)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn recovery_uses_frozen_criteria_and_never_invents_a_checked_pass() {
        let fixture = Fixture::new();
        {
            let mut store = Store::open(&fixture.path()).unwrap();
            let mut input = admission("checked", "alice");
            input.task = Some(TaskIdentity {
                profile_id: "fields".into(),
                profile_version: "7".into(),
                spec_sha256: candidate_digest("frozen"),
                checker_id: "file_fields_v1".into(),
                checker_version: "1".into(),
                criterion_ids: vec!["language".into()],
            });
            store.execute(Command::Admit(input)).unwrap();
            store
                .execute(Command::Append {
                    owner_id: "alice".into(),
                    run_id: "checked".into(),
                    event: event(1, "run_started"),
                    phase: Some("running".into()),
                })
                .unwrap();
            store
                .execute(Command::Append {
                    owner_id: "alice".into(),
                    run_id: "checked".into(),
                    event: event(2, "model_planned"),
                    phase: None,
                })
                .unwrap();
        }
        let store = Store::open(&fixture.path()).unwrap();
        let record = get_run(&store.connection, "alice", "checked")
            .unwrap()
            .unwrap();
        assert_eq!(record.phase, "interrupted");
        assert_eq!(record.acceptance_status, "inconclusive");
        assert!(!record.task_accepted());
        let receipt = record.receipt.unwrap();
        assert_eq!(receipt["criteria"][0]["id"], "language");
        assert_eq!(receipt["contract"]["profile_version"], "7");
        let events = read_events(&store.connection, "alice", "checked", None, 100).unwrap();
        assert_eq!(events.len(), 4);
        assert_eq!(events[3].kind, "recovery_interrupted");
        assert_eq!(events[3].data["pending_effect_outcome"], "unknown");
    }

    #[test]
    fn second_controller_fails_before_recovery_and_unknown_schema_fails_closed() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        start_run(&mut store, "run", "alice");
        let second = Store::open(&fixture.path());
        assert_eq!(second.err().unwrap().code, "storage_controller_locked");
        assert_eq!(
            get_run(&store.connection, "alice", "run")
                .unwrap()
                .unwrap()
                .phase,
            "running"
        );
        store
            .connection
            .pragma_update(None, "user_version", 99)
            .unwrap();
        drop(store);
        assert_eq!(
            Store::open(&fixture.path()).err().unwrap().code,
            "storage_unknown_schema"
        );
    }

    #[cfg(unix)]
    #[test]
    fn shutdown_releases_controller_lock_with_inherited_descriptor() {
        let fixture = Fixture::new();
        let store = Store::open(&fixture.path()).unwrap();
        // Like fork inheritance, try_clone shares the open file description.
        // Keep it alive across shutdown to make the parallel-spawn race exact.
        let inherited = store._controller_lock.0.try_clone().unwrap();
        drop(store);
        let reopened = Store::open(&fixture.path()).unwrap();
        assert_eq!(
            Store::open(&fixture.path()).err().unwrap().code,
            "storage_controller_locked"
        );
        drop(inherited);
        drop(reopened);
    }

    #[test]
    fn sql_constraints_reject_terminal_pending_and_passed_running() {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.path()).unwrap();
        let mut input = admission("checked", "alice");
        input.task = Some(TaskIdentity {
            profile_id: "fields".into(),
            profile_version: "1".into(),
            spec_sha256: candidate_digest("spec"),
            checker_id: "file_fields_v1".into(),
            checker_version: "1".into(),
            criterion_ids: vec!["language".into()],
        });
        store.execute(Command::Admit(input)).unwrap();
        assert!(store.connection.execute("UPDATE runs SET phase='completed',result_json='{}',result_sha256=?1,acceptance_json='{}' WHERE run_id='checked'",[candidate_digest("")]).is_err());
        assert!(
            store
                .connection
                .execute(
                    "UPDATE runs SET acceptance_status='passed' WHERE run_id='checked'",
                    []
                )
                .is_err()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reservations_reject_growth_after_arbitration_and_bound_waiters() {
        let fixture = Fixture::new();
        let storage = Storage::start(
            fixture.path(),
            QueueLimits {
                commands: 1,
                bytes: 4096,
                max_waiters: 1,
            },
        )
        .unwrap();
        let client = storage.client();
        let reservation = client
            .reserve(1, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(client.outstanding(), (1, 1));
        let command = Command::Admit(admission("run", "alice"));
        assert_eq!(
            reservation.submit(command.clone()).err().unwrap().code,
            "storage_reservation_too_small"
        );
        assert_eq!(client.outstanding(), (0, 0));
        let reservation = client
            .reserve(
                command.encoded_len().unwrap(),
                Instant::now() + Duration::from_secs(1),
            )
            .await
            .unwrap();
        let problem = client
            .reserve(1, Instant::now() + Duration::from_millis(10))
            .await
            .err()
            .unwrap();
        assert_eq!(problem.code, "storage_admission_timeout");
        reservation.submit(command).unwrap().wait().await.unwrap();
        storage.shutdown().await.unwrap();
        assert!(!client.is_accepting());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropped_ack_and_dequeue_keep_permits_until_actual_transaction_finishes() {
        let fixture = Fixture::new();
        let storage = Storage::start(
            fixture.path(),
            QueueLimits {
                commands: 1,
                bytes: 4096,
                max_waiters: 2,
            },
        )
        .unwrap();
        let client = storage.client();
        let blocker = Connection::open(fixture.path()).unwrap();
        blocker.execute_batch("BEGIN IMMEDIATE;").unwrap();
        let command = Command::Admit(admission("run", "alice"));
        let size = command.encoded_len().unwrap();
        let ack = client
            .reserve(size, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap()
            .submit(command)
            .unwrap();
        let until = Instant::now() + Duration::from_secs(1);
        while client.sender.capacity() != 1 {
            assert!(Instant::now() < until, "worker did not dequeue command");
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        drop(ack);
        assert_eq!(client.outstanding(), (1, size));
        assert_eq!(
            client
                .reserve(1, Instant::now() + Duration::from_millis(10))
                .await
                .err()
                .unwrap()
                .code,
            "storage_admission_timeout"
        );
        blocker.execute_batch("COMMIT;").unwrap();
        drop(blocker);
        let response = client
            .execute(
                Command::Get {
                    owner_id: "alice".into(),
                    run_id: "run".into(),
                },
                Instant::now() + Duration::from_secs(2),
            )
            .await
            .unwrap();
        assert!(matches!(response, Response::Run(Some(_))));
        storage.shutdown().await.unwrap();
        assert_eq!(client.outstanding(), (0, 0));
    }
}
