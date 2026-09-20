//! Bounded historical operation descriptors, never current state or acceptance.
//!
//! Only journal metadata is inspected. A recorded success does not prove that
//! bytes changed or behavior works; an error does not exclude partial effects.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::config::{ToolName, validate_id, validate_relative_path};
use crate::storage::Event;

pub const MAX_EFFECT_EVENTS: usize = 256;
pub const MAX_EFFECT_RECORDS: usize = 12;
pub const MAX_EFFECT_SUMMARY_BYTES: usize = 4_096;
const MAX_RESOURCE_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectDispatch {
    Unknown,
    Denied,
    Executed,
    Unsent,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectOutcome {
    Unknown,
    Ok,
    Error,
    Denied,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairStatus {
    Unverified,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EffectRecord {
    pub effect_id: String,
    /// None means the tool is unsupported. Its name and payload are not copied.
    pub tool: Option<ToolName>,
    pub resource: Option<String>,
    pub dispatch: EffectDispatch,
    pub outcome: EffectOutcome,
    pub result_complete: Option<bool>,
    /// Sticky even when a conflicting later observation makes the result unknown.
    pub repair_status: Option<RepairStatus>,
    pub conflicted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EffectSummary {
    pub version: u32,
    pub run_id: String,
    pub events_scanned: usize,
    pub tool_events: usize,
    pub records: Vec<EffectRecord>,
    /// Known identified records removed, including atomic session-memory trimming.
    pub omitted_records: usize,
    /// Tool events whose missing/oversized identity could not be represented.
    pub omitted_events: usize,
    /// Distinct identified unsupported tools, whether retained or omitted.
    pub unsupported_records: usize,
    /// A contiguous scan beginning at zero reached the terminal journal event.
    pub history_complete: bool,
    /// No terminal event was reached, or input exceeded the scan bound.
    pub unscanned_tail: bool,
    /// Missing, conflicting, unsupported, omitted or incomplete observations.
    pub incomplete: bool,
}

impl EffectSummary {
    pub fn has_activity(&self) -> bool {
        self.tool_events > 0
    }

    pub fn encoded_len(&self) -> Result<usize, String> {
        serde_json::to_vec(self)
            .map(|bytes| bytes.len())
            .map_err(|_| "cannot encode effect summary".into())
    }

    /// Drop complete facts, never truncate an identity into a different path.
    pub fn drop_oldest_record(&mut self) -> bool {
        if self.records.is_empty() {
            return false;
        }
        self.records.remove(0);
        self.omitted_records = self.omitted_records.saturating_add(1);
        self.incomplete = true;
        true
    }

    /// Validate frozen reference data without reading files or granting authority.
    pub fn validate(&self) -> Result<(), String> {
        validate_id(&self.run_id)?;
        let invalid = || "invalid bounded effect summary".to_owned();
        if self.version != 1
            || self.events_scanned > MAX_EFFECT_EVENTS
            || self.tool_events > self.events_scanned
            || self.records.len() > MAX_EFFECT_RECORDS
            || self.omitted_records > self.tool_events
            || self.omitted_events > self.tool_events
            || self.records.len() + self.omitted_records + self.omitted_events > self.tool_events
            || self.unsupported_records > self.records.len() + self.omitted_records
            || (self.history_complete && (self.unscanned_tail || self.events_scanned == 0))
        {
            return Err(invalid());
        }
        let mut ids = BTreeSet::new();
        let mut incomplete = !self.history_complete
            || self.unscanned_tail
            || self.omitted_records > 0
            || self.omitted_events > 0
            || self.unsupported_records > 0;
        let mut unsupported = 0;
        for record in &self.records {
            validate_id(&record.effect_id)?;
            if !ids.insert(&record.effect_id) {
                return Err(invalid());
            }
            if let Some(resource) = &record.resource {
                validate_resource(resource)?;
            }
            if record.tool.is_none() {
                unsupported += 1;
                if record.resource.is_some()
                    || record.dispatch != EffectDispatch::Unknown
                    || record.outcome != EffectOutcome::Unknown
                    || record.result_complete.is_some()
                {
                    return Err(invalid());
                }
            }
            if record.conflicted
                && (record.dispatch != EffectDispatch::Unknown
                    || record.outcome != EffectOutcome::Unknown
                    || record.result_complete.is_some())
            {
                return Err(invalid());
            }
            if record.outcome == EffectOutcome::Unknown {
                if record.result_complete.is_some()
                    || matches!(
                        record.dispatch,
                        EffectDispatch::Executed | EffectDispatch::Unsent
                    )
                    || (record.repair_status.is_some() && !record.conflicted)
                {
                    return Err(invalid());
                }
            } else if record.result_complete.is_none()
                || record.dispatch == EffectDispatch::Unknown
                || (record.dispatch == EffectDispatch::Unsent
                    && record.outcome != EffectOutcome::Error)
                || (record.dispatch == EffectDispatch::Denied
                    && record.outcome != EffectOutcome::Denied)
                || ((record.outcome == EffectOutcome::Error) != record.repair_status.is_some())
            {
                return Err(invalid());
            }
            incomplete |= record.tool.is_none()
                || record.conflicted
                || record.outcome == EffectOutcome::Unknown
                || record.result_complete == Some(false);
        }
        if unsupported > self.unsupported_records
            || (incomplete && !self.incomplete)
            || self.encoded_len()? > MAX_EFFECT_SUMMARY_BYTES
        {
            return Err(invalid());
        }
        Ok(())
    }
}

struct PendingRecord {
    record: EffectRecord,
    planned: bool,
    finished: bool,
    omitted: bool,
}

/// Feed owner-scoped events in sequence, including non-tool events. Short pages
/// do not end the history; only a contiguous scan through run_finished does.
pub struct EffectSummaryBuilder {
    summary: EffectSummary,
    pending: Vec<PendingRecord>,
    next_seq: u64,
    contiguous: bool,
    terminal_seen: bool,
}

impl EffectSummaryBuilder {
    pub fn new(run_id: impl Into<String>) -> Result<Self, String> {
        let run_id = run_id.into();
        validate_id(&run_id)?;
        Ok(Self {
            summary: EffectSummary {
                version: 1,
                run_id,
                events_scanned: 0,
                tool_events: 0,
                records: Vec::new(),
                omitted_records: 0,
                omitted_events: 0,
                unsupported_records: 0,
                history_complete: false,
                unscanned_tail: true,
                incomplete: false,
            },
            pending: Vec::new(),
            next_seq: 0,
            contiguous: true,
            terminal_seen: false,
        })
    }

    pub fn remaining_events(&self) -> usize {
        MAX_EFFECT_EVENTS.saturating_sub(self.summary.events_scanned)
    }

    pub fn is_complete(&self) -> bool {
        self.contiguous && self.terminal_seen && !self.summary.unscanned_tail
    }

    pub fn observe(&mut self, event: &Event) {
        if self.remaining_events() == 0 {
            self.summary.unscanned_tail = true;
            self.summary.incomplete = true;
            return;
        }
        self.summary.events_scanned += 1;
        let ordered = !self.terminal_seen && event.seq == self.next_seq;
        if !ordered {
            self.contiguous = false;
            self.summary.incomplete = true;
            for pending in &mut self.pending {
                if !pending.finished {
                    conflict(&mut pending.record);
                }
            }
        }
        self.next_seq = event.seq.saturating_add(1);
        if event.kind == "run_finished" {
            self.terminal_seen = true;
            self.summary.unscanned_tail = false;
            return;
        }
        if !matches!(event.kind.as_str(), "tool_planned" | "tool_finished") {
            return;
        }
        self.summary.tool_events += 1;
        let Some(id) = event.data["effect_id"]
            .as_str()
            .filter(|id| validate_id(id).is_ok())
        else {
            self.summary.omitted_events += 1;
            self.summary.incomplete = true;
            return;
        };
        let tool = event.data["tool"].as_str().and_then(ToolName::from_wire);
        let index = match self
            .pending
            .iter()
            .position(|pending| pending.record.effect_id == id)
        {
            Some(index) => index,
            None => {
                self.pending.push(PendingRecord {
                    record: EffectRecord {
                        effect_id: id.to_owned(),
                        tool,
                        resource: None,
                        dispatch: EffectDispatch::Unknown,
                        outcome: EffectOutcome::Unknown,
                        result_complete: None,
                        repair_status: None,
                        conflicted: false,
                    },
                    planned: false,
                    finished: false,
                    omitted: false,
                });
                self.pending.len() - 1
            }
        };
        let pending = &mut self.pending[index];
        if !ordered || pending.record.tool != tool {
            conflict(&mut pending.record);
        }
        // Unsupported names and payloads are never copied, even when they look
        // like instructions, paths or familiar tool output.
        if pending.record.tool.is_none() || tool.is_none() {
            pending.record.tool = None;
            pending.record.resource = None;
            pending.record.dispatch = EffectDispatch::Unknown;
            pending.record.outcome = EffectOutcome::Unknown;
            pending.record.result_complete = None;
        }
        if event.kind == "tool_planned" {
            if pending.planned || pending.finished {
                conflict(&mut pending.record);
            }
            pending.planned = true;
            match event.data["dispatch"].as_str() {
                Some("denied") if pending.record.tool.is_some() && !pending.record.conflicted => {
                    pending.record.dispatch = EffectDispatch::Denied;
                }
                Some("pending" | "denied") => {}
                _ => conflict(&mut pending.record),
            }
            return;
        }
        if pending.finished || !pending.planned {
            conflict(&mut pending.record);
        }
        pending.finished = true;
        if pending.record.tool.is_none() {
            return;
        }
        let outcome = match event.data["classification"].as_str() {
            Some("ok") => EffectOutcome::Ok,
            Some("error") => {
                pending.record.repair_status = Some(RepairStatus::Unverified);
                EffectOutcome::Error
            }
            Some("denied") => EffectOutcome::Denied,
            _ => EffectOutcome::Unknown,
        };
        let dispatch = match event.data["dispatch"].as_str() {
            Some("executed") => EffectDispatch::Executed,
            Some("denied") => EffectDispatch::Denied,
            Some("unsent") => EffectDispatch::Unsent,
            _ => EffectDispatch::Unknown,
        };
        let complete = event.data["complete"].as_bool();
        if outcome == EffectOutcome::Unknown
            || dispatch == EffectDispatch::Unknown
            || complete.is_none()
            || (dispatch == EffectDispatch::Unsent && outcome != EffectOutcome::Error)
            || (dispatch == EffectDispatch::Denied && outcome != EffectOutcome::Denied)
            || (pending.record.dispatch == EffectDispatch::Denied
                && dispatch != EffectDispatch::Denied)
        {
            conflict(&mut pending.record);
        }
        match event.data.get("resource") {
            None | Some(serde_json::Value::Null) => {}
            // The runner records TypedToolArgs.path for every compiled tool;
            // run_command deliberately has no path and encodes that as "".
            Some(serde_json::Value::String(resource))
                if resource.is_empty() && pending.record.tool == Some(ToolName::RunCommand) => {}
            Some(serde_json::Value::String(resource)) if validate_resource(resource).is_ok() => {
                if pending
                    .record
                    .resource
                    .as_ref()
                    .is_some_and(|prior| prior != resource)
                {
                    conflict(&mut pending.record);
                } else if !pending.record.conflicted {
                    pending.record.resource = Some(resource.clone());
                }
            }
            _ => {
                pending.omitted = true;
                self.summary.incomplete = true;
            }
        }
        if !pending.record.conflicted {
            pending.record.dispatch = dispatch;
            pending.record.outcome = outcome;
            pending.record.result_complete = complete;
        }
    }

    pub fn finished(mut self) -> EffectSummary {
        self.summary.history_complete = self.is_complete();
        self.summary.incomplete |= !self.summary.history_complete;
        for pending in self.pending {
            if pending.record.tool.is_none() {
                self.summary.unsupported_records += 1;
            }
            if pending.omitted {
                self.summary.omitted_records += 1;
                self.summary.incomplete = true;
                continue;
            }
            self.summary.incomplete |= pending.record.tool.is_none()
                || pending.record.conflicted
                || pending.record.outcome == EffectOutcome::Unknown
                || pending.record.result_complete == Some(false);
            self.summary.records.push(pending.record);
        }
        while self.summary.records.len() > MAX_EFFECT_RECORDS
            || self
                .summary
                .encoded_len()
                .is_ok_and(|bytes| bytes > MAX_EFFECT_SUMMARY_BYTES)
        {
            if !self.summary.drop_oldest_record() {
                break;
            }
        }
        self.summary
    }
}

fn conflict(record: &mut EffectRecord) {
    record.conflicted = true;
    record.resource = None;
    record.dispatch = EffectDispatch::Unknown;
    record.outcome = EffectOutcome::Unknown;
    record.result_complete = None;
}

fn validate_resource(resource: &str) -> Result<(), String> {
    if resource.len() > MAX_RESOURCE_BYTES {
        return Err("effect resource exceeds its byte limit".into());
    }
    if resource == "." {
        Ok(())
    } else {
        validate_relative_path(resource)
    }
}
