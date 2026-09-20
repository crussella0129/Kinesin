//! Terminal presentation for interactive sessions; durable receipts stay in storage.

use crate::config::{ModelConfig, WorkspaceConfig};
use crate::storage::{Event, RunRecord};

// The journal accepts candidates up to 1 MiB. Retaining only this suffix lets
// a final answer complete an interrupted display without retaining all turns.
const STREAM_SUFFIX_BYTES: usize = 1024 * 1024;

pub(super) const SESSION_HELP: &str = "Session commands:\n  /help         Show these commands\n  /status       Show the last run ID and its outcome\n  /context      Show recent conversation memory and its limits\n  /permissions  Show granted tools and commands\n  /new          Clear conversation memory (also /clear)\n  /exit         End this session (also /quit)\n\nOther text is sent to the model. Freeform answers have no independent acceptance check.\n\n";

pub(super) fn safe_text(text: &str) -> String {
    let mut safe = String::with_capacity(text.len());
    for character in text.chars() {
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

pub(super) fn permissions(workspace: &WorkspaceConfig) -> String {
    let tools = workspace
        .tools
        .iter()
        .map(|tool| tool.wire_name())
        .collect::<Vec<_>>();
    let tools = if tools.is_empty() {
        "none".to_owned()
    } else {
        tools.join(", ")
    };
    let mut intro = format!("Allowed tools: {}\n", safe_text(&tools));
    if !workspace.commands.is_empty() {
        intro.push_str(&format!(
            "Allowed commands: {}\n",
            safe_text(&workspace.commands.join(", "))
        ));
    }
    if !workspace.tools.iter().any(|tool| tool.is_mutating()) {
        intro.push_str("This workspace is read-only; file and folder changes are disabled.\n");
    }
    intro.push('\n');
    intro
}

pub(super) fn session_intro(workspace: &WorkspaceConfig, model: &ModelConfig) -> String {
    let capabilities = workspace
        .tools
        .iter()
        .map(|tool| match tool.wire_name().as_str() {
            "list_files" => "list files".to_owned(),
            "read_file" => "read files".to_owned(),
            "search_files" => "search files".to_owned(),
            "create_directory" => "create folders".to_owned(),
            "write_file" => "write files".to_owned(),
            "edit_file" => "edit files".to_owned(),
            "delete_file" => "delete files".to_owned(),
            "move_file" => "move files".to_owned(),
            "run_command" => "run approved commands".to_owned(),
            _ => format!("extension {}", tool.wire_name()),
        })
        .collect::<Vec<_>>();
    let capabilities = if capabilities.is_empty() {
        "no tools".to_owned()
    } else {
        capabilities.join(", ")
    };
    let mut intro = format!(
        "\nKinesin\nWorkspace: {}\nModel: {} ({})\nCan: {}\n",
        safe_text(&workspace.root.to_string_lossy()),
        safe_text(&model.model_id),
        safe_text(&model.id),
        safe_text(&capabilities)
    );
    if !workspace.tools.iter().any(|tool| tool.is_mutating()) {
        intro.push_str("Read-only workspace; file and folder changes are disabled.\n");
    }
    intro.push_str("Paths refer to this workspace. Recent requests and answers stay in memory until /new or exit.\nType a request, /help for commands, or /exit to leave.\n\n");
    intro
}

#[derive(Default)]
pub(super) struct HumanDisplay {
    suffix: String,
    started: bool,
}

impl HumanDisplay {
    pub(super) fn delta(&mut self, text: &str) -> String {
        let mut display = String::new();
        if !self.started {
            display.push_str("[Provisional response]\n");
            self.started = true;
        }
        self.suffix.push_str(text);
        if self.suffix.len() > STREAM_SUFFIX_BYTES {
            let mut offset = self.suffix.len() - STREAM_SUFFIX_BYTES;
            while !self.suffix.is_char_boundary(offset) {
                offset += 1;
            }
            self.suffix.drain(..offset);
        }
        display.push_str(&safe_text(text));
        display
    }

    pub(super) fn break_stream(&mut self) {
        self.suffix.clear();
    }

    pub(super) fn activity(&mut self, text: &str) -> String {
        let prefix = if self.started { "\n" } else { "" };
        // A final response must remain readable after activity interrupts its
        // provisional prefix. Print the complete saved answer in that case.
        self.break_stream();
        format!("{prefix}{text}")
    }

    pub(super) fn finished(&self, record: &RunRecord) -> String {
        let mut display = String::new();
        if let Some(candidate) = record
            .result
            .as_ref()
            .and_then(|result| result.get("candidate"))
            .and_then(|candidate| candidate.as_str())
        {
            let overlap = if self.started {
                suffix_overlap(&self.suffix, candidate)
            } else {
                0
            };
            if self.started && overlap == 0 {
                display.push_str("\nSaved response:\n");
            }
            display.push_str(&safe_text(&candidate[overlap..]));
            display.push('\n');
        } else if self.started {
            display.push('\n');
        }
        if record.phase == "completed" && record.acceptance_status == "unchecked" {
            display.push('\n');
        } else {
            display.push_str(&format!("\n{}", run_status(record)));
        }
        display
    }
}

pub(super) fn run_status(record: &RunRecord) -> String {
    let status = match (record.phase.as_str(), record.acceptance_status.as_str()) {
        ("completed", "unchecked") => "Run completed (freeform; no acceptance check).".to_owned(),
        ("completed", "passed") => "Run completed; acceptance check passed.".to_owned(),
        ("completed", "failed") => "Run completed; acceptance check failed.".to_owned(),
        ("completed", _) => "Run completed; acceptance is inconclusive.".to_owned(),
        (phase, _) => format!(
            "Run {}: {}.",
            phase.replace('_', " "),
            record
                .terminal_reason
                .as_deref()
                .unwrap_or("no terminal reason")
                .replace('_', " ")
        ),
    };
    format!(
        "{}\nRun: {}\n\n",
        safe_text(&status),
        safe_text(&record.run_id)
    )
}

#[derive(Default)]
pub(super) struct Activity {
    after: Option<u64>,
    observed: usize,
}

impl Activity {
    pub(super) fn after(&self) -> Option<u64> {
        self.after
    }

    pub(super) fn full(&self) -> bool {
        self.observed >= crate::replay::MAX_REPLAY_EVENTS
    }

    pub(super) fn observe(&mut self, events: Vec<Event>) -> String {
        let mut output = String::new();
        for event in events {
            if self.full() {
                break;
            }
            self.after = Some(event.seq);
            self.observed += 1;
            let tool = event
                .data
                .get("tool")
                .and_then(|value| value.as_str())
                .unwrap_or("tool");
            match event.kind.as_str() {
                "tool_planned" => output.push_str(&format!("  {} ...\n", safe_text(tool))),
                "tool_finished" => {
                    let resource = event
                        .data
                        .get("resource")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let classification = event
                        .data
                        .get("classification")
                        .and_then(|value| value.as_str())
                        .unwrap_or("finished");
                    output.push_str(&format!(
                        "  {} {}: {}\n",
                        safe_text(tool),
                        safe_text(resource),
                        safe_text(classification)
                    ));
                }
                _ => {}
            }
        }
        output
    }
}

/// Find the candidate prefix already visible at the end of the stream. Model
/// turns can contain tool preambles, and completion can win before the last
/// queued delta is displayed. KMP keeps this bounded and linear in input size.
fn suffix_overlap(stream: &str, candidate: &str) -> usize {
    let pattern = candidate.as_bytes();
    if pattern.is_empty() {
        return 0;
    }
    let mut prefixes = vec![0; pattern.len()];
    let mut matched = 0;
    for index in 1..pattern.len() {
        while matched > 0 && pattern[index] != pattern[matched] {
            matched = prefixes[matched - 1];
        }
        if pattern[index] == pattern[matched] {
            matched += 1;
        }
        prefixes[index] = matched;
    }
    matched = 0;
    for byte in stream.bytes() {
        while matched > 0 && (matched == pattern.len() || byte != pattern[matched]) {
            matched = prefixes[matched - 1];
        }
        if byte == pattern[matched] {
            matched += 1;
        }
    }
    // Both inputs end at UTF-8 character boundaries; a matching suffix must as
    // well. Keep this invariant explicit before slicing the candidate.
    debug_assert!(candidate.is_char_boundary(matched));
    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streamed_answer_is_completed_without_repeating_prior_text() {
        assert_eq!(suffix_overlap("early", "early answer"), 5);
        assert_eq!(suffix_overlap("I will look.\nearly", "early answer"), 5);
        assert_eq!(suffix_overlap("early answer", "early answer"), 12);
        assert_eq!(suffix_overlap("different", "answer"), 0);
        assert_eq!(suffix_overlap("Searching.\n日本", "日本語"), 6);
        assert_eq!(suffix_overlap("", ""), 0);
        assert_eq!(suffix_overlap("ababab", "abab"), 4);
    }

    #[test]
    fn human_text_preserves_prose_but_escapes_terminal_commands() {
        assert_eq!(
            safe_text("two\nlines\tOK\r\x1b[2J\u{009b}\u{202e}"),
            "two\nlines\tOK\\u{d}\\u{1b}[2J\\u{9b}\\u{202e}"
        );
    }
}
