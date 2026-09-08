//! Pure acceptance checks over a frozen task and runner-owned observations.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::config::{Limits, TaskCriterion, TaskProfile, validate_id};
use crate::core::AcceptanceStatus;
use crate::policy::{RunAuthority, TaskContract, sha256};
use crate::tools::{ToolResult, ToolStatus, normalized_path};

pub const MAX_EVIDENCE_BYTES: usize = 65_536;
pub const MAX_CHECKED_CANDIDATE_BYTES: usize = 8_192;
pub const MAX_RECEIPT_BYTES: usize = 8_192;
const CHECKED_SCOPE: &str = "Required fields match complete successful file observations from this run; no current-world or freshness claim.";

/// Read-only assessment data; implementing this cannot authorize a live effect.
/// Replay's private frozen context uses the same bounded pure checker.
pub(crate) trait AssessmentContext {
    fn owner(&self) -> &str;
    fn run_id(&self) -> &str;
    fn limits(&self) -> &Limits;
    fn task(&self) -> &TaskContract;
    fn task_spec_sha256(&self) -> &str;
}

impl AssessmentContext for RunAuthority {
    fn owner(&self) -> &str {
        self.owner()
    }
    fn run_id(&self) -> &str {
        self.run_id()
    }
    fn limits(&self) -> &Limits {
        self.limits()
    }
    fn task(&self) -> &TaskContract {
        self.task()
    }
    fn task_spec_sha256(&self) -> &str {
        self.task_spec_sha256()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EvidenceRecord {
    pub owner_id: String,
    pub run_id: String,
    pub effect_id: String,
    pub tool_name: String,
    pub resource: String,
    pub body: String,
    pub status: ToolStatus,
    pub completeness: bool,
    pub seq: u64,
    pub id: String,
    pub digest: String,
}

/// There is deliberately no deserializer or public record insertion operation.
#[derive(Default, Debug)]
pub struct EvidenceInventory {
    records: Vec<EvidenceRecord>,
    retained_bytes: usize,
    fault: Option<&'static str>,
}

impl EvidenceInventory {
    pub fn next_id(&self) -> String {
        format!("e{}", self.records.len())
    }

    pub fn records(&self) -> &[EvidenceRecord] {
        &self.records
    }

    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    /// Call only after an actual `read_file` observation is journaled. The runner
    /// supplies the effect/sequence and preissues the ID to the bounded tool.
    pub fn add_observation(
        &mut self,
        authority: &RunAuthority,
        effect_id: &str,
        path: &str,
        result: &ToolResult,
        seq: u64,
    ) -> Result<(), String> {
        self.add_context_observation(authority, effect_id, path, result, seq)
    }

    pub(crate) fn add_context_observation(
        &mut self,
        authority: &dyn AssessmentContext,
        effect_id: &str,
        path: &str,
        result: &ToolResult,
        seq: u64,
    ) -> Result<(), String> {
        if let Some(fault) = self.fault {
            return Err(fault.into());
        }
        if result.status != ToolStatus::Ok {
            return Ok(()); // Error/denied bodies and IDs never become evidence.
        }
        let id = result.evidence_id.as_deref().unwrap_or("");
        let valid = validate_id(id).is_ok()
            && validate_id(effect_id).is_ok()
            && normalized_path(path).is_ok()
            && seq > 0
            && result.error.is_none()
            && self.records.iter().all(|record| {
                record.owner_id == authority.owner()
                    && record.run_id == authority.run_id()
                    && record.id != id
                    && record.effect_id != effect_id
                    && record.seq != seq
            });
        if !valid {
            return self.reject("verification_evidence_invalid");
        }
        let encoded_fits = result.body.len() <= authority.limits().max_tool_result_bytes
            && result.encoded().is_ok_and(|body| {
                body.len() <= authority.limits().max_tool_result_bytes.min(8_192)
            });
        // Count every retained body, including incomplete prefixes, plus owned
        // identity strings and fixed record storage before allocating a copy.
        let bytes = std::mem::size_of::<EvidenceRecord>()
            + authority.owner().len()
            + authority.run_id().len()
            + effect_id.len()
            + "read_file".len()
            + path.len()
            + result.body.len()
            + id.len()
            + 64;
        if !encoded_fits
            || self.records.len() as u64 >= u64::from(authority.limits().max_tool_calls)
            || bytes > MAX_EVIDENCE_BYTES.saturating_sub(self.retained_bytes)
        {
            return self.reject("verification_evidence_limit");
        }
        self.records.push(EvidenceRecord {
            owner_id: authority.owner().into(),
            run_id: authority.run_id().into(),
            effect_id: effect_id.into(),
            tool_name: "read_file".into(),
            resource: path.into(),
            body: result.body.clone(),
            status: result.status,
            completeness: !result.truncated,
            seq,
            id: id.into(),
            digest: sha256(result.body.as_bytes()),
        });
        self.retained_bytes += bytes;
        Ok(())
    }

    fn reject(&mut self, code: &'static str) -> Result<(), String> {
        self.fault = Some(code);
        Err(code.into())
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReceiptContract {
    pub profile_id: String,
    pub profile_version: String,
    pub checker_id: String,
    pub checker_version: String,
    pub spec_sha256: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct EvidenceBinding {
    pub evidence_id: String,
    pub effect_id: String,
    pub seq: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CriterionResult {
    pub id: String,
    pub status: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceBinding>,
}

/// Shape of the candidate a checked run must emit, derived from the frozen
/// contract so the request and the checker cannot describe different objects.
///
/// This constrains **shape only**. llama.cpp's converter skips unsupported
/// keywords silently, so relying on one for correctness would mean believing a
/// constraint that was never applied. Which ids are acceptable, which values are
/// right, and whether the cited observation supports them stay with the checker.
pub fn candidate_schema(profile: &TaskProfile) -> serde_json::Value {
    let fields = profile.criteria.len();
    serde_json::json!({
        "type": "object",
        "properties": {
            "facts": {
                "type": "array",
                "minItems": fields,
                "maxItems": fields,
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "value": {"type": "string"},
                        "evidence_id": {"type": "string"}
                    },
                    "required": ["id", "value", "evidence_id"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["facts"],
        "additionalProperties": false
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VerifiedField {
    pub id: String,
    pub value: String,
    pub evidence_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    facts: Vec<VerifiedField>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AcceptanceReceipt {
    pub owner_id: String,
    pub run_id: String,
    pub status: String,
    pub reason: String,
    pub contract: Option<ReceiptContract>,
    pub candidate_sha256: Option<String>,
    pub criteria: Vec<CriterionResult>,
    pub scope: String,
    pub duration_ms: u64,
    #[serde(skip)]
    verified: Option<Vec<VerifiedField>>,
}

impl AcceptanceReceipt {
    pub fn verified_fields(&self) -> Option<&[VerifiedField]> {
        if self.status == "passed" {
            self.verified.as_deref()
        } else {
            None
        }
    }

    pub fn acceptance_status(&self) -> AcceptanceStatus {
        match self.status.as_str() {
            "unchecked" => AcceptanceStatus::Unchecked,
            "passed" => AcceptanceStatus::Passed,
            "failed" => AcceptanceStatus::Failed,
            _ => AcceptanceStatus::Inconclusive,
        }
    }
}

/// Cancellation and checker faults preserve every frozen required criterion.
/// Duration is set by the runner; this module does not read a clock.
pub fn inconclusive(
    authority: &RunAuthority,
    candidate: Option<&str>,
    reason: &str,
) -> AcceptanceReceipt {
    inconclusive_context(authority, candidate, reason)
}

pub(crate) fn inconclusive_context(
    authority: &dyn AssessmentContext,
    candidate: Option<&str>,
    reason: &str,
) -> AcceptanceReceipt {
    let reason = if !reason.is_empty()
        && reason.len() <= 64
        && reason
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    {
        reason
    } else {
        "verification_inconclusive"
    };
    let (contract, criteria, status, scope) = match authority.task() {
        TaskContract::Freeform => (None, Vec::new(), "unchecked", "unchecked freeform task"),
        TaskContract::FileFieldsV1(profile) => (
            Some(ReceiptContract {
                profile_id: profile.id.clone(),
                profile_version: profile.version.to_string(),
                checker_id: profile.checker.clone(),
                checker_version: profile.checker_version.to_string(),
                spec_sha256: authority.task_spec_sha256().into(),
            }),
            profile
                .criteria
                .iter()
                .map(|c| criterion_result(&c.id, "inconclusive", reason))
                .collect(),
            "inconclusive",
            CHECKED_SCOPE,
        ),
    };
    AcceptanceReceipt {
        owner_id: authority.owner().into(),
        run_id: authority.run_id().into(),
        status: status.into(),
        reason: reason.into(),
        contract,
        candidate_sha256: candidate.map(|text| sha256(text.as_bytes())),
        criteria,
        scope: scope.into(),
        duration_ms: 0,
        verified: None,
    }
}

/// The caller has already bounded the provider response and still owns its run.
/// JSON parsing is capped at 8 KiB; no file, model, or database is consulted.
pub fn assess(
    authority: &RunAuthority,
    candidate: &str,
    inventory: &EvidenceInventory,
) -> AcceptanceReceipt {
    assess_context(authority, candidate, inventory)
}

pub(crate) fn assess_context(
    authority: &dyn AssessmentContext,
    candidate: &str,
    inventory: &EvidenceInventory,
) -> AcceptanceReceipt {
    let mut receipt = inconclusive_context(authority, Some(candidate), "no_acceptance_contract");
    let TaskContract::FileFieldsV1(profile) = authority.task() else {
        return receipt;
    };
    if let Some(code) = inventory.fault {
        return inconclusive_context(authority, Some(candidate), code);
    }
    if !valid_profile(profile)
        || inventory
            .records
            .iter()
            .any(|record| sha256(record.body.as_bytes()) != record.digest)
    {
        return inconclusive_context(authority, Some(candidate), "checker_fault");
    }
    let parsed = if candidate.len() <= MAX_CHECKED_CANDIDATE_BYTES {
        serde_json::from_str::<Candidate>(candidate).ok()
    } else {
        None
    };
    let Some(parsed) = parsed.filter(|parsed| valid_facts(profile, &parsed.facts)) else {
        let code = if candidate.len() > MAX_CHECKED_CANDIDATE_BYTES {
            "candidate_too_large"
        } else {
            "invalid_output_contract"
        };
        receipt.status = "failed".into();
        receipt.reason = code.into();
        receipt.criteria = profile
            .criteria
            .iter()
            .map(|c| criterion_result(&c.id, "failed", code))
            .collect();
        return receipt;
    };

    // A required file is parsed once even when several criteria select its keys.
    let mut sources = BTreeMap::new();
    for criterion in &profile.criteria {
        sources.entry(criterion.path.as_str()).or_insert_with(|| {
            complete_source(authority, &criterion.path, inventory).and_then(parse_source)
        });
    }
    receipt.criteria = profile
        .criteria
        .iter()
        .map(|criterion| {
            let Some(fact) = parsed.facts.iter().find(|fact| fact.id == criterion.id) else {
                return criterion_result(&criterion.id, "inconclusive", "checker_fault");
            };
            let Some(source) = sources.get(criterion.path.as_str()) else {
                return criterion_result(&criterion.id, "inconclusive", "checker_fault");
            };
            check_field(authority, criterion, fact, inventory, source)
        })
        .collect();
    let status = if receipt.criteria.iter().any(|r| r.code == "checker_fault") {
        "inconclusive"
    } else if receipt.criteria.iter().any(|r| r.status == "failed") {
        "failed"
    } else if receipt.criteria.iter().any(|r| r.status != "passed") {
        "inconclusive"
    } else {
        "passed"
    };
    receipt.status = status.into();
    receipt.reason = match status {
        "passed" => "all_required_fields_match",
        "failed" => "required_check_failed",
        _ => "evidence_unresolved",
    }
    .into();
    if status == "passed" {
        receipt.verified = Some(parsed.facts);
    }
    if serde_json::to_vec(&receipt).is_ok_and(|bytes| bytes.len() <= MAX_RECEIPT_BYTES) {
        receipt
    } else {
        inconclusive_context(authority, Some(candidate), "checker_fault")
    }
}

fn valid_profile(profile: &TaskProfile) -> bool {
    profile.checker == "file_fields_v1"
        && profile.checker_version == 1
        && (1..=4).contains(&profile.criteria.len())
        && profile
            .criteria
            .iter()
            .map(|c| &c.id)
            .collect::<BTreeSet<_>>()
            .len()
            == profile.criteria.len()
        && serde_json::to_vec(profile).is_ok_and(|bytes| bytes.len() <= 8_192)
}

fn valid_facts(profile: &TaskProfile, facts: &[VerifiedField]) -> bool {
    facts.len() == profile.criteria.len()
        && facts.iter().all(|fact| {
            validate_id(&fact.id).is_ok()
                && validate_id(&fact.evidence_id).is_ok()
                && !fact.value.is_empty()
                && fact.value.len() <= 1_024
                && profile
                    .criteria
                    .iter()
                    .any(|criterion| criterion.id == fact.id)
        })
        && facts
            .iter()
            .map(|fact| &fact.id)
            .collect::<BTreeSet<_>>()
            .len()
            == facts.len()
}

fn criterion_result(id: &str, status: &str, code: &str) -> CriterionResult {
    CriterionResult {
        id: id.into(),
        status: status.into(),
        code: code.into(),
        evidence: None,
    }
}

fn complete_source<'a>(
    authority: &dyn AssessmentContext,
    path: &str,
    inventory: &'a EvidenceInventory,
) -> Result<&'a str, &'static str> {
    let mut observations = inventory.records.iter().filter(|record| {
        record.owner_id == authority.owner()
            && record.run_id == authority.run_id()
            && record.tool_name == "read_file"
            && record.resource == path
            && record.status == ToolStatus::Ok
            && record.completeness
    });
    let first = observations.next().ok_or("source_unavailable")?;
    if observations.any(|record| record.body != first.body) {
        return Err("source_conflict");
    }
    Ok(&first.body)
}

fn parse_source(body: &str) -> Result<BTreeMap<&str, &str>, &'static str> {
    let mut fields = BTreeMap::new();
    for line in body.split_inclusive('\n') {
        let line = match line.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => line,
        };
        if line.contains('\r') {
            return Err("invalid_source");
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or("invalid_source")?;
        let valid_key = !key.is_empty()
            && key.len() <= 32
            && key.as_bytes()[0].is_ascii_lowercase()
            && key
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_');
        if !valid_key
            || value.is_empty()
            || value.len() > 1_024
            || fields.insert(key, value).is_some()
        {
            return Err("invalid_source");
        }
    }
    Ok(fields)
}

fn check_field(
    authority: &dyn AssessmentContext,
    criterion: &TaskCriterion,
    fact: &VerifiedField,
    inventory: &EvidenceInventory,
    source: &Result<BTreeMap<&str, &str>, &'static str>,
) -> CriterionResult {
    let mut result = criterion_result(&criterion.id, "failed", "evidence_unknown");
    let Some(record) = inventory
        .records
        .iter()
        .find(|record| record.id == fact.evidence_id)
    else {
        return result;
    };
    if record.owner_id != authority.owner() || record.run_id != authority.run_id() {
        result.code = "evidence_wrong_run".into();
        return result;
    }
    if record.tool_name != "read_file" || record.resource != criterion.path {
        result.code = "evidence_wrong_resource".into();
        return result;
    }
    if record.status != ToolStatus::Ok {
        result.code = "evidence_not_usable".into();
        return result;
    }
    result.evidence = Some(EvidenceBinding {
        evidence_id: record.id.clone(),
        effect_id: record.effect_id.clone(),
        seq: record.seq,
        sha256: record.digest.clone(),
    });
    let (status, code) = if !record.completeness {
        ("inconclusive", "source_incomplete")
    } else if criterion
        .required_sha256
        .as_ref()
        .is_some_and(|expected| expected != &record.digest)
    {
        ("failed", "source_revision_mismatch")
    } else {
        match source {
            Err(code) => ("inconclusive", *code),
            Ok(fields) => match fields.get(criterion.key.as_str()) {
                None => ("inconclusive", "source_key_missing"),
                Some(value) if *value != fact.value => ("failed", "value_mismatch"),
                Some(_) => ("passed", "field_matches"),
            },
        }
    };
    result.status = status.into();
    result.code = code.into();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_support::{BASE, Fixture, OWNER, TASK};
    use crate::policy::Submission;

    fn checked(task: &str) -> (Fixture, RunAuthority) {
        let fixture = Fixture::new();
        let config = fixture.parse(&format!("{BASE}{task}{OWNER}")).unwrap();
        let authority = config
            .authorize(
                "alice",
                Submission::Checked {
                    task: "practice-fields".into(),
                    model: "local".into(),
                    limits: None,
                    capture: None,
                },
            )
            .unwrap();
        (fixture, authority)
    }

    fn candidate(value: &str, evidence: &str) -> String {
        serde_json::json!({"facts":[{"id":"language","value":value,"evidence_id":evidence}]})
            .to_string()
    }

    fn observe(
        inventory: &mut EvidenceInventory,
        authority: &RunAuthority,
        path: &str,
        body: &str,
        truncated: bool,
    ) {
        let id = inventory.next_id();
        inventory
            .add_observation(
                authority,
                &format!("tool-{id}"),
                path,
                &ToolResult {
                    status: ToolStatus::Ok,
                    body: body.into(),
                    truncated,
                    error: None,
                    evidence_id: Some(id.clone()),
                },
                inventory.records.len() as u64 + 1,
            )
            .unwrap();
    }

    #[test]
    fn compares_values_instead_of_trusting_a_genuine_citation() {
        let (_fixture, authority) = checked(TASK);
        let mut evidence = EvidenceInventory::default();
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "project=Kinesin\nlanguage=Rust\n",
            false,
        );
        let good = format!(" {}\n", candidate("Rust", "e0"));
        let passed = assess(&authority, &good, &evidence);
        assert_eq!(passed.status, "passed");
        assert_eq!(passed.candidate_sha256, Some(sha256(good.as_bytes())));
        assert_eq!(passed.verified_fields().unwrap()[0].value, "Rust");
        assert_eq!(passed.criteria[0].evidence.as_ref().unwrap().seq, 1);
        assert_eq!(passed.contract.as_ref().unwrap().profile_version, "1");
        let serialized = serde_json::to_string(&passed).unwrap();
        assert!(serialized.len() < MAX_RECEIPT_BYTES);
        assert!(!serialized.contains("Rust"));
        for wrong in ["Python", "rust", " Rust", "Rust "] {
            let failed = assess(&authority, &candidate(wrong, "e0"), &evidence);
            assert_eq!(failed.status, "failed");
            assert_eq!(failed.criteria[0].code, "value_mismatch");
            assert!(failed.verified_fields().is_none());
        }
    }

    #[test]
    fn output_contract_rejects_omissions_duplicates_unknown_fields_and_prose() {
        let (_fixture, authority) = checked(TASK);
        let mut evidence = EvidenceInventory::default();
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "language=Rust",
            false,
        );
        let good = candidate("Rust", "e0");
        for bad in [
            "{\"facts\":[]}".into(), "{\"facts\":[],\"facts\":[]}".into(),
            good.replace("\"Rust\"", "\"Rust\",\"value\":\"Python\""),
            good.replace("\"Rust\"", "\"Rust\",\"confidence\":1"),
            good.replace("\"facts\":", "\"extra\":true,\"facts\":"),
            format!("{good} This proves Python."), format!("```json\n{good}\n```"),
            "{\"facts\":[{\"id\":\"language\",\"value\":\"Rust\"}]}".into(),
            serde_json::json!({"facts":[{"id":"language","value":"Rust","evidence_id":"e0"},{"id":"language","value":"Rust","evidence_id":"e0"}]}).to_string(),
            " ".repeat(MAX_CHECKED_CANDIDATE_BYTES + 1),
        ] {
            let receipt = assess(&authority, &bad, &evidence);
            assert_eq!(receipt.status, "failed", "{bad}");
            assert_eq!(receipt.criteria.len(), 1);
        }
    }

    #[test]
    fn binds_evidence_to_owner_run_resource_and_successful_read() {
        let (_fixture, authority) = checked(TASK);
        let good = candidate("Rust", "e0");
        assert_eq!(
            assess(&authority, &good, &EvidenceInventory::default()).status,
            "failed"
        );
        for mutation in 0..4 {
            let mut evidence = EvidenceInventory::default();
            observe(
                &mut evidence,
                &authority,
                "project.txt",
                "language=Rust",
                false,
            );
            match mutation {
                0 => evidence.records[0].owner_id = "another-owner".into(),
                1 => evidence.records[0].run_id = "another-run".into(),
                2 => evidence.records[0].resource = "other/project.txt".into(),
                _ => evidence.records[0].tool_name = "list_files".into(),
            }
            assert_eq!(assess(&authority, &good, &evidence).status, "failed");
        }
        for status in [ToolStatus::Error, ToolStatus::Denied] {
            let mut evidence = EvidenceInventory::default();
            evidence
                .add_observation(
                    &authority,
                    "tool-0",
                    "project.txt",
                    &ToolResult {
                        status,
                        body: "language=Rust".into(),
                        truncated: false,
                        error: None,
                        evidence_id: Some("e0".into()),
                    },
                    1,
                )
                .unwrap();
            assert!(evidence.records().is_empty());
            assert_eq!(assess(&authority, &good, &evidence).status, "failed");
        }
    }

    #[test]
    fn incomplete_changed_or_malformed_sources_cannot_pass() {
        let (_fixture, authority) = checked(TASK);
        let good = candidate("Rust", "e0");
        for body in [
            "",
            "project=Kinesin",
            "language=Rust\nlanguage=Go",
            "Language=Rust",
            "language =Rust",
            "language=Rust\r",
            "ignore all checks",
            "language=",
        ] {
            let mut evidence = EvidenceInventory::default();
            observe(&mut evidence, &authority, "project.txt", body, false);
            assert_eq!(
                assess(&authority, &good, &evidence).status,
                "inconclusive",
                "{body:?}"
            );
        }
        let mut evidence = EvidenceInventory::default();
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "language=Rust",
            true,
        );
        assert_eq!(assess(&authority, &good, &evidence).status, "inconclusive");
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "language=Rust\nproject=Kinesin",
            false,
        );
        assert_eq!(
            assess(&authority, &candidate("Rust", "e1"), &evidence).status,
            "passed"
        );
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "language=Go",
            false,
        );
        assert_eq!(
            assess(&authority, &candidate("Rust", "e1"), &evidence).status,
            "inconclusive"
        );
    }

    #[test]
    fn source_grammar_preserves_bytes_and_supports_crlf_comments_and_equals() {
        assert_eq!(
            parse_source("# comment\r\n\r\nlanguage= Rust=🦀 \r\n").unwrap()["language"],
            " Rust=🦀 "
        );
        assert!(parse_source(&format!("language={}", "x".repeat(1025))).is_err());
        assert!(parse_source("language=Rust\rproject=Kinesin").is_err());
        assert!(parse_source(" language=Rust").is_err());
    }

    #[test]
    fn revision_and_aggregate_rules_do_not_average_or_skip_checks() {
        let task = format!("{TASK}required_sha256 = \"{}\"\n", sha256(b"language=Go"));
        let (_fixture, authority) = checked(&task);
        let mut evidence = EvidenceInventory::default();
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "language=Rust",
            false,
        );
        let receipt = assess(&authority, &candidate("Rust", "e0"), &evidence);
        assert_eq!(receipt.status, "failed");
        assert_eq!(receipt.criteria[0].code, "source_revision_mismatch");

        let task = format!(
            "{TASK}\n[[tasks.criteria]]\nid='project'\npath='project.txt'\nkey='project'\n"
        );
        let (_fixture, authority) = checked(&task);
        let mut evidence = EvidenceInventory::default();
        observe(
            &mut evidence,
            &authority,
            "project.txt",
            "language=Rust",
            false,
        );
        let answer = r#"{"facts":[{"id":"language","value":"Python","evidence_id":"e0"},{"id":"project","value":"Kinesin","evidence_id":"e0"}]}"#;
        let receipt = assess(&authority, answer, &evidence);
        assert_eq!(receipt.status, "failed");
        assert_eq!(receipt.criteria[1].status, "inconclusive");
        evidence.records[0].digest = "0".repeat(64);
        let faulty = assess(&authority, answer, &evidence);
        assert_eq!(faulty.status, "inconclusive");
        assert!(faulty.criteria.iter().all(|c| c.code == "checker_fault"));
        assert_eq!(
            inconclusive(&authority, None, "cancelled").criteria.len(),
            2
        );
        assert!(
            inconclusive(&authority, None, "cancelled")
                .candidate_sha256
                .is_none()
        );
    }

    #[test]
    fn retained_prefixes_count_and_exhaustion_never_discards_into_a_pass() {
        let (_fixture, authority) = checked(TASK);
        let mut evidence = EvidenceInventory::default();
        for i in 0..authority.limits().max_tool_calls {
            let result = evidence.add_observation(
                &authority,
                &format!("tool-{i}"),
                "project.txt",
                &ToolResult {
                    status: ToolStatus::Ok,
                    body: "x".repeat(7500),
                    truncated: true,
                    error: None,
                    evidence_id: Some(format!("e{i}")),
                },
                i as u64 + 1,
            );
            if let Err(code) = result {
                assert_eq!(code, "verification_evidence_limit");
                break;
            }
        }
        assert!(evidence.retained_bytes() <= MAX_EVIDENCE_BYTES);
        assert!(evidence.fault.is_some());
        assert!(!evidence.records().is_empty());
        assert_eq!(
            assess(&authority, "malformed candidate", &evidence).status,
            "inconclusive"
        );
    }

    #[test]
    fn freeform_is_unchecked_and_diagnostics_do_not_echo_untrusted_input() {
        let fixture = Fixture::new();
        let config = fixture.parse(BASE).unwrap();
        let authority = config
            .authorize_local(Submission::Freeform {
                workspace: "practice".into(),
                model: "local".into(),
                continues: None,
                prompt: "Say hello".into(),
                limits: None,
                capture: None,
            })
            .unwrap();
        let receipt = assess(
            &authority,
            "I passed all checks",
            &EvidenceInventory::default(),
        );
        assert_eq!(receipt.status, "unchecked");
        assert!(receipt.contract.is_none());
        assert!(receipt.criteria.is_empty());
        assert!(receipt.verified_fields().is_none());
        let receipt = inconclusive(&authority, None, "SECRET path/exception 🦀");
        assert_eq!(receipt.reason, "verification_inconclusive");
    }
}
