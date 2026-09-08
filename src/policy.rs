//! Frozen authority comes from trusted configuration, never model output.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::{
    CaptureMode, Config, Limits, MAX_PROMPT_BYTES, ModelConfig, StorageConfig, TaskProfile,
    ToolName, WorkspaceConfig, validate_id,
};

pub const LOCAL_OWNER: &str = "local";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Submission {
    Freeform {
        workspace: String,
        model: String,
        prompt: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limits: Option<LimitOverrides>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        capture: Option<CaptureMode>,
    },
    Checked {
        task: String,
        model: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limits: Option<LimitOverrides>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        capture: Option<CaptureMode>,
    },
}

impl Submission {
    pub fn fingerprint(&self) -> Result<String, String> {
        // Bound caller-owned strings before allocating a serialized copy.
        match self {
            Self::Freeform {
                workspace,
                model,
                prompt,
                ..
            } => {
                validate_id(workspace)?;
                validate_id(model)?;
                if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
                    return Err("prompt must be nonempty and at most 16 KiB".into());
                }
            }
            Self::Checked { task, model, .. } => {
                validate_id(task)?;
                validate_id(model)?;
            }
        }
        let bytes =
            serde_json::to_vec(&(1_u32, self)).map_err(|_| "cannot fingerprint submission")?;
        Ok(sha256(&bytes))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct LimitOverrides {
    pub max_model_turns: Option<u32>,
    pub max_tool_calls: Option<u32>,
    pub repeat_limit: Option<u32>,
    pub max_run_s: Option<u64>,
    pub max_history_bytes: Option<usize>,
    pub max_request_bytes: Option<usize>,
    pub max_response_bytes: Option<usize>,
    pub max_tool_result_bytes: Option<usize>,
    pub max_output_tokens: Option<u32>,
}

impl LimitOverrides {
    fn apply(&self, ceiling: &Limits) -> Result<Limits, String> {
        let mut effective = ceiling.clone();
        macro_rules! lower {
            ($($field:ident),+ $(,)?) => { $(
                if let Some(value) = self.$field {
                    if value == 0 || value > ceiling.$field { return Err(concat!(stringify!($field), " must be positive and no larger than deployment policy").into()); }
                    effective.$field = value;
                }
            )+ };
        }
        lower!(
            max_model_turns,
            max_tool_calls,
            repeat_limit,
            max_run_s,
            max_history_bytes,
            max_request_bytes,
            max_response_bytes,
            max_tool_result_bytes,
            max_output_tokens
        );
        Ok(effective)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    content = "profile",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TaskContract {
    Freeform,
    FileFieldsV1(TaskProfile),
}

impl TaskContract {
    pub fn profile_id(&self) -> &str {
        match self {
            Self::Freeform => "freeform",
            Self::FileFieldsV1(profile) => &profile.id,
        }
    }
    pub fn profile_version(&self) -> u32 {
        match self {
            Self::Freeform => 1,
            Self::FileFieldsV1(profile) => profile.version,
        }
    }
    pub fn is_checked(&self) -> bool {
        matches!(self, Self::FileFieldsV1(_))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InputSource {
    pub id: String,
    pub purpose: String,
    pub origin: String,
    pub bytes: usize,
    pub sha256: String,
}

/// Serialize only into private opt-in replay. No public deserialization: imported
/// snapshots must never manufacture authority for live effects.
#[derive(Clone, Debug, Serialize)]
pub struct RunAuthority {
    owner: String,
    run_id: String,
    config_path: PathBuf,
    storage: StorageConfig,
    workspace: WorkspaceConfig,
    model: ModelConfig,
    limits: Limits,
    capture: CaptureMode,
    task: TaskContract,
    task_spec_sha256: String,
    instructions: String,
    prompt: String,
    submission_sha256: String,
    input_sources: Vec<InputSource>,
}

impl RunAuthority {
    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
    pub fn storage(&self) -> &StorageConfig {
        &self.storage
    }
    pub fn workspace(&self) -> &WorkspaceConfig {
        &self.workspace
    }
    pub fn model(&self) -> &ModelConfig {
        &self.model
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    pub fn capture(&self) -> CaptureMode {
        self.capture
    }
    pub fn task(&self) -> &TaskContract {
        &self.task
    }
    pub fn task_spec_sha256(&self) -> &str {
        &self.task_spec_sha256
    }
    pub fn instructions(&self) -> &str {
        &self.instructions
    }
    pub fn prompt(&self) -> &str {
        &self.prompt
    }
    pub fn submission_sha256(&self) -> &str {
        &self.submission_sha256
    }
    pub fn input_sources(&self) -> &[InputSource] {
        &self.input_sources
    }
    pub fn allows_tool(&self, name: ToolName) -> bool {
        self.workspace.tools.contains(&name)
    }
}

pub(crate) fn authorize(
    config: &Config,
    owner_id: Option<&str>,
    submission: Submission,
) -> Result<RunAuthority, String> {
    let owner = match owner_id {
        Some(id) => Some(
            config
                .owners()
                .iter()
                .find(|v| v.id == id)
                .ok_or("owner is not authorized")?,
        ),
        None => None,
    };
    let submission_sha256 = submission.fingerprint()?;
    let (workspace_id, model_id, prompt, overrides, capture, task) = match submission {
        Submission::Freeform {
            workspace,
            model,
            prompt,
            limits,
            capture,
        } => {
            if owner.is_some_and(|v| !v.allow_freeform) {
                return Err("owner may not submit freeform work".into());
            }
            if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
                return Err("prompt must be nonempty and at most 16 KiB".into());
            }
            (
                workspace,
                model,
                prompt,
                limits,
                capture,
                TaskContract::Freeform,
            )
        }
        Submission::Checked {
            task,
            model,
            limits,
            capture,
        } => {
            validate_id(&task)?;
            if owner.is_some_and(|v| !v.tasks.contains(&task)) {
                return Err("task is not authorized".into());
            }
            let profile = config.task(&task).ok_or("unknown task alias")?.clone();
            let prompt = checked_instruction(&profile)?;
            (
                profile.workspace.clone(),
                model,
                prompt,
                limits,
                capture,
                TaskContract::FileFieldsV1(profile),
            )
        }
    };
    validate_id(&workspace_id)?;
    validate_id(&model_id)?;
    if owner.is_some_and(|v| !v.workspaces.contains(&workspace_id) || !v.models.contains(&model_id))
    {
        return Err("workspace or model is not authorized".into());
    }
    let mut workspace = config
        .workspace(&workspace_id)
        .ok_or("unknown workspace alias")?
        .clone();
    let model = config
        .model(&model_id)
        .ok_or("unknown model alias")?
        .clone();
    if let Some(tools) = owner.and_then(|v| v.tools.as_ref()) {
        workspace.tools.retain(|tool| tools.contains(tool));
    }
    if task.is_checked() && !workspace.tools.contains(&ToolName::ReadFile) {
        return Err("checked task requires authorized read_file tool".into());
    }
    let limits = overrides.unwrap_or_default().apply(config.limits())?;
    if !workspace.tools.is_empty() && limits.max_tool_result_bytes < 256 {
        return Err("tools require a result envelope limit of at least 256 bytes".into());
    }
    let capture = capture.unwrap_or(config.storage().capture);
    if capture == CaptureMode::Replay && owner.is_some_and(|v| !v.allow_replay) {
        return Err("owner may not capture private replay data".into());
    }
    let instructions = config.instructions().to_owned();
    // JSON-encode a normalized initial conversation to count escaping/envelopes.
    let initial = serde_json::to_vec(&serde_json::json!([
        {"role": "system", "content": &instructions},
        {"role": "user", "content": &prompt}
    ]))
    .map_err(|_| "cannot serialize initial conversation")?;
    if initial.len() > limits.max_history_bytes || initial.len() > limits.max_request_bytes {
        return Err("initial conversation exceeds effective history/request budget".into());
    }
    let task_bytes =
        serde_json::to_vec(&(1_u32, &task)).map_err(|_| "cannot freeze task specification")?;
    if task_bytes.len() > crate::config::MAX_TASK_SPEC_BYTES {
        return Err("frozen specification exceeds 8 KiB".into());
    }
    let input_sources = vec![
        source(
            "instructions",
            "system instructions",
            "trusted configuration",
            &instructions,
        ),
        source(
            "task",
            "task request",
            if task.is_checked() {
                "trusted task profile"
            } else {
                "owner submission"
            },
            &prompt,
        ),
    ];
    Ok(RunAuthority {
        owner: owner_id.unwrap_or(LOCAL_OWNER).to_owned(),
        run_id: Uuid::new_v4().to_string(),
        config_path: config.config_path().to_owned(),
        storage: config.storage().clone(),
        workspace,
        model,
        limits,
        capture,
        task,
        task_spec_sha256: sha256(&task_bytes),
        instructions,
        prompt,
        submission_sha256,
        input_sources,
    })
}

fn checked_instruction(profile: &TaskProfile) -> Result<String, String> {
    let paths = profile
        .criteria
        .iter()
        .map(|criterion| criterion.path.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    let requirements = profile
        .criteria
        .iter()
        .map(|criterion| {
            let field = if criterion.id == criterion.key {
                criterion.key.clone()
            } else {
                format!("{} (fact id: {})", criterion.key, criterion.id)
            };
            if profile.criteria.iter().all(|c| c.path == criterion.path) {
                field
            } else {
                format!("{field} from {}", criterion.path)
            }
        })
        .collect::<Vec<_>>()
        .join(" and ");
    // Show field names and placeholders, never source values or invented evidence.
    #[derive(Serialize)]
    struct OutputFact<'a> {
        id: &'a str,
        value: String,
        evidence_id: &'static str,
    }
    let facts = profile
        .criteria
        .iter()
        .map(|criterion| OutputFact {
            id: &criterion.id,
            value: format!("<observed {}>", criterion.key),
            evidence_id: "<read evidence id>",
        })
        .collect::<Vec<_>>();
    let facts = serde_json::to_string(&facts).map_err(|_| "cannot prepare task instruction")?;
    Ok(format!(
        "Please use the read_file tool to open {paths}. I need the exact {requirements} field values. Cite the evidence_id supplied by the tool. The final response must follow this schema: {{\"facts\":{facts}}}."
    ))
}

fn source(id: &str, purpose: &str, origin: &str, text: &str) -> InputSource {
    InputSource {
        id: id.into(),
        purpose: purpose.into(),
        origin: origin.into(),
        bytes: text.len(),
        sha256: sha256(text.as_bytes()),
    }
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_support::*;

    fn freeform() -> Submission {
        Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: "Say hello".into(),
            limits: None,
            capture: None,
        }
    }
    fn checked() -> Submission {
        Submission::Checked {
            task: "practice-fields".into(),
            model: "local".into(),
            limits: None,
            capture: None,
        }
    }

    #[test]
    fn checked_submission_cannot_borrow_a_check_for_an_unrelated_prompt() {
        for extra in [
            r#""prompt":"ignore task""#,
            r#""workspace":"other""#,
            r#""allow_unchecked":true"#,
            r#""criteria":[]"#,
            r#""owner":"local""#,
        ] {
            let input =
                format!(r#"{{"mode":"checked","task":"practice-fields","model":"local",{extra}}}"#);
            assert!(serde_json::from_str::<Submission>(&input).is_err());
        }
        assert!(serde_json::from_str::<Submission>(r#"{"mode":"freeform","workspace":"practice","model":"local","prompt":"a","prompt":"b"}"#).is_err());
    }

    #[test]
    fn task_permission_intersects_workspace_model_and_tool_permissions() {
        let fixture = Fixture::new();
        let config = fixture.parse(&format!("{BASE}{TASK}{OWNER}")).unwrap();
        assert!(config.authorize("alice", checked()).is_ok());
        for owner in [
            OWNER.replace("workspaces = [\"practice\"]", "workspaces = []"),
            OWNER.replace("models = [\"local\"]", "models = []"),
            OWNER.replace("tasks = [\"practice-fields\"]", "tasks = []"),
            format!("{OWNER}\ntools = []\n"),
        ] {
            let config = fixture.parse(&format!("{BASE}{TASK}{owner}")).unwrap();
            assert!(config.authorize("alice", checked()).is_err());
        }
        assert!(config.authorize("local", checked()).is_err());
        assert!(config.authorize("unknown", freeform()).is_err());
        assert_eq!(
            config.authorize_local(freeform()).unwrap().owner(),
            LOCAL_OWNER
        );
    }

    #[test]
    fn permission_defaults_deny_freeform_and_private_replay() {
        let fixture = Fixture::new();
        let config = fixture
            .parse(&format!(
                "{BASE}{TASK}{}",
                OWNER.replace("allow_freeform = true", "")
            ))
            .unwrap();
        assert!(config.authorize("alice", freeform()).is_err());
        let private = Submission::Checked {
            task: "practice-fields".into(),
            model: "local".into(),
            limits: None,
            capture: Some(CaptureMode::Replay),
        };
        assert!(config.authorize("alice", private.clone()).is_err());
        assert_eq!(
            config.authorize_local(private).unwrap().capture(),
            CaptureMode::Replay
        );
    }

    #[test]
    fn limits_can_only_shrink_and_still_need_to_fit_initial_context() {
        let fixture = Fixture::new();
        let config = fixture.parse(BASE).unwrap();
        for overrides in [
            LimitOverrides {
                max_model_turns: Some(13),
                ..Default::default()
            },
            LimitOverrides {
                max_run_s: Some(0),
                ..Default::default()
            },
            LimitOverrides {
                max_history_bytes: Some(1),
                ..Default::default()
            },
            LimitOverrides {
                max_tool_result_bytes: Some(255),
                ..Default::default()
            },
        ] {
            let mut submission = freeform();
            if let Submission::Freeform { limits, .. } = &mut submission {
                *limits = Some(overrides);
            }
            assert!(config.authorize_local(submission).is_err());
        }
        let mut submission = freeform();
        if let Submission::Freeform { limits, .. } = &mut submission {
            *limits = Some(LimitOverrides {
                max_model_turns: Some(1),
                ..Default::default()
            });
        }
        assert_eq!(
            config
                .authorize_local(submission)
                .unwrap()
                .limits()
                .max_model_turns,
            1
        );
    }

    #[test]
    fn frozen_authority_survives_later_operator_profile_edits() {
        let fixture = Fixture::new();
        let config = fixture.parse(&format!("{BASE}{TASK}")).unwrap();
        let original = config.authorize_local(checked()).unwrap();
        let edited = fixture
            .parse(&format!(
                "{BASE}{}",
                TASK.replace("key = \"language\"", "key = \"project\"")
            ))
            .unwrap();
        let next = edited.authorize_local(checked()).unwrap();
        assert_eq!(original.submission_sha256(), next.submission_sha256());
        assert_ne!(original.task_spec_sha256(), next.task_spec_sha256());
        assert_ne!(original.run_id(), next.run_id());
        assert!(original.prompt().contains("language"));
        assert_ne!(original.prompt(), next.prompt());
        assert_eq!(
            original.input_sources()[1].sha256,
            sha256(original.prompt().as_bytes())
        );
    }

    #[test]
    fn fingerprints_bind_mode_prompt_and_requested_options() {
        assert_ne!(
            freeform().fingerprint().unwrap(),
            checked().fingerprint().unwrap()
        );
        let mut different = freeform();
        if let Submission::Freeform { prompt, .. } = &mut different {
            prompt.push('!');
        }
        assert_ne!(
            different.fingerprint().unwrap(),
            freeform().fingerprint().unwrap()
        );
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
