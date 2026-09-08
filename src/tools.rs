//! Three bounded read operations behind a private filesystem capability.

use std::io::{ErrorKind, Read};
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::{ToolName, validate_id, validate_relative_path};

pub const MAX_TOOL_BYTES: usize = 8_192;
pub const MAX_VISITED_ENTRIES: usize = 256;
pub const MAX_ARGUMENT_BYTES: usize = 65_536;
/// A literal search term. Not a pattern language: an expression engine would be
/// a new dependency and an unbounded matching cost on model-selected input.
pub const MAX_QUERY_BYTES: usize = 128;
/// Directory levels a search may descend below its requested path.
pub const MAX_SEARCH_DEPTH: usize = 4;
/// Bytes read from any single file while searching.
pub const MAX_SEARCH_FILE_BYTES: usize = 65_536;
/// Bytes a single write may place. Content arrives inside the tool arguments,
/// so it is already under MAX_ARGUMENT_BYTES; this states the write's own bound.
pub const MAX_WRITE_BYTES: usize = 32_768;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Ok,
    Error,
    Denied,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolResult {
    pub status: ToolStatus,
    pub body: String,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
}

#[cfg(test)]
impl ToolResult {
    fn assert_ok(&self) {
        assert_eq!(self.status, ToolStatus::Ok, "{:?}", self.error);
    }
}

impl ToolResult {
    pub fn failure(status: ToolStatus, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            body: String::new(),
            truncated: false,
            error: Some(ToolError {
                code: code.into(),
                message: message.into(),
            }),
            evidence_id: None,
        }
    }

    fn success(evidence_id: Option<&str>) -> Self {
        Self {
            status: ToolStatus::Ok,
            body: String::new(),
            truncated: false,
            error: None,
            evidence_id: evidence_id.map(str::to_owned),
        }
    }

    pub fn encoded(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|_| "tool result serialization failed".into())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TypedToolArgs {
    pub path: String,
    /// Present only for `search_files`. Absent for the single-path tools, so an
    /// unexpected term on `read_file` still fails the typed contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Optional for `search_files`; absent means an insensitive match. Lexical
    /// search fails when the caller guesses the wrong casing, and a smaller
    /// model refines a failed query least reliably, so the forgiving mode is
    /// the default and exactness is the deliberate request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_sensitive: Option<bool>,
    /// Present only for `write_file`. Absent for the read tools, so unexpected
    /// content on a read still fails the typed contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The two present only for `edit_file`: the exact text to find, and its
    /// replacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub find: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace: Option<String>,
    /// The destination, present only for `move_file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

impl TypedToolArgs {
    /// One place that decides which fields each tool accepts, so a field can
    /// never be silently ignored on a tool that does not use it.
    pub fn shape_for(&self, name: ToolName) -> Result<(), (&'static str, &'static str)> {
        // Each tool accepts a fixed set of optional fields. A required field
        // that is missing, or any field a tool does not use, fails the contract,
        // so a value can never be silently ignored on the wrong tool.
        let has_query = self.query.is_some() || self.case_sensitive.is_some();
        let has_content = self.content.is_some();
        let has_edit = self.find.is_some() || self.replace.is_some();
        let has_to = self.to.is_some();

        let missing = match name {
            ToolName::SearchFiles if self.query.is_none() => Some((
                "missing_query",
                "search_files requires a literal query term.",
            )),
            ToolName::WriteFile if self.content.is_none() => {
                Some(("missing_content", "write_file requires content to write."))
            }
            ToolName::EditFile if self.find.is_none() || self.replace.is_none() => Some((
                "missing_edit",
                "edit_file requires both find and replace text.",
            )),
            ToolName::MoveFile if self.to.is_none() => {
                Some(("missing_destination", "move_file requires a destination."))
            }
            _ => None,
        };
        if let Some(error) = missing {
            return Err(error);
        }

        if has_query && name != ToolName::SearchFiles {
            return Err((
                "unexpected_query",
                "Only search_files accepts a query term.",
            ));
        }
        if has_content && name != ToolName::WriteFile {
            return Err(("unexpected_content", "Only write_file accepts content."));
        }
        if has_edit && name != ToolName::EditFile {
            return Err((
                "unexpected_edit",
                "Only edit_file accepts find and replace.",
            ));
        }
        if has_to && name != ToolName::MoveFile {
            return Err((
                "unexpected_destination",
                "Only move_file accepts a destination.",
            ));
        }
        Ok(())
    }
}

impl TypedToolArgs {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_ARGUMENT_BYTES {
            return Err("tool arguments exceed 64 KiB".into());
        }
        let mut args: Self =
            serde_json::from_str(raw).map_err(|_| "invalid tool arguments or fields")?;
        args.path = normalized_path(&args.path)?;
        if let Some(to) = &args.to {
            args.to = Some(normalized_path(to)?);
        }
        if let Some(query) = &args.query {
            args.query = Some(validated_query(query)?);
        }
        for field in [&args.content, &args.find, &args.replace] {
            if field
                .as_ref()
                .is_some_and(|value| value.len() > MAX_WRITE_BYTES)
            {
                return Err("write content exceeds its byte limit".into());
            }
        }
        Ok(args)
    }
}

/// A search term is literal text. Reject empty terms, oversized terms, and
/// control characters, which cannot appear in a matched line the caller reads.
pub fn validated_query(query: &str) -> Result<String, String> {
    if query.is_empty() {
        return Err("search term cannot be empty".into());
    }
    if query.len() > MAX_QUERY_BYTES {
        return Err("search term exceeds its byte limit".into());
    }
    if query.chars().any(char::is_control) {
        return Err("search term cannot contain control characters".into());
    }
    Ok(query.to_owned())
}

pub fn normalized_path(path: &str) -> Result<String, String> {
    if path != "." {
        validate_relative_path(path)?;
    }
    if path
        .chars()
        .any(|c| matches!(c, '<' | '>' | '"' | '|' | '?' | '*'))
        || path.split('/').any(|part| {
            matches!(
                part.split('.')
                    .next()
                    .unwrap_or("")
                    .to_ascii_uppercase()
                    .as_str(),
                "COM¹" | "COM²" | "COM³" | "LPT¹" | "LPT²" | "LPT³"
            )
        })
    {
        return Err("resource path uses unsupported Windows filename syntax".into());
    }
    Ok(path.to_owned())
}

pub struct WorkspaceReader {
    root: Dir,
}

impl WorkspaceReader {
    /// Trusted startup only; never call with a path from a model or submission.
    pub fn open(root: &Path) -> Result<Self, String> {
        Dir::open_ambient_dir(root, ambient_authority())
            .map(|root| Self { root })
            .map_err(|_| "cannot open approved workspace directory".into())
    }

    /// Synchronous bounded work. The caller owns its blocking-worker permit.
    /// Config validation guarantees max_bytes >= 256. A lower direct-call limit
    /// is an invalid API use: even its diagnostic cannot fit an arbitrary cap.
    pub fn execute(
        &self,
        name: ToolName,
        path: &str,
        max_bytes: usize,
        evidence_id: Option<&str>,
    ) -> ToolResult {
        self.execute_with_query(name, path, None, false, max_bytes, evidence_id)
    }

    /// `query` is required by `search_files` and rejected by the other tools,
    /// so a term can never be silently ignored on a single-path operation.
    pub fn execute_with_query(
        &self,
        name: ToolName,
        path: &str,
        query: Option<&str>,
        case_sensitive: bool,
        max_bytes: usize,
        evidence_id: Option<&str>,
    ) -> ToolResult {
        if max_bytes < 256 {
            return ToolResult::failure(
                ToolStatus::Error,
                "invalid_limit",
                "Tool result budget must be at least 256 bytes.",
            );
        }
        let limit = max_bytes.min(MAX_TOOL_BYTES);
        if normalized_path(path).is_err() {
            return ToolResult::failure(
                ToolStatus::Denied,
                "invalid_path",
                "Use a supported relative workspace path.",
            );
        }
        let query = match (name, query) {
            (ToolName::SearchFiles, Some(query)) => match validated_query(query) {
                Ok(query) => Some(query),
                Err(_) => {
                    return ToolResult::failure(
                        ToolStatus::Denied,
                        "invalid_query",
                        "Provide a literal search term within its byte limit.",
                    );
                }
            },
            (ToolName::SearchFiles, None) => {
                return ToolResult::failure(
                    ToolStatus::Denied,
                    "missing_query",
                    "search_files requires a literal query term.",
                );
            }
            (_, Some(_)) => {
                return ToolResult::failure(
                    ToolStatus::Denied,
                    "unexpected_query",
                    "Only search_files accepts a query term.",
                );
            }
            (_, None) => None,
        };
        match name {
            ToolName::ReadFile => self.read_file(path, limit, evidence_id),
            ToolName::ListFiles => self.list_files(path, limit),
            ToolName::SearchFiles => self.search_files(
                path,
                &query.expect("search query checked above"),
                case_sensitive,
                limit,
            ),
            // A read capability cannot write. The runner routes a write to the
            // separate WorkspaceWriter; reaching here is a routing fault.
            ToolName::WriteFile
            | ToolName::EditFile
            | ToolName::DeleteFile
            | ToolName::MoveFile => ToolResult::failure(
                ToolStatus::Denied,
                "not_a_write_capability",
                "This capability only reads.",
            ),
        }
    }

    fn read_file(&self, path: &str, limit: usize, evidence_id: Option<&str>) -> ToolResult {
        if evidence_id.is_some_and(|id| validate_id(id).is_err()) {
            return ToolResult::failure(
                ToolStatus::Error,
                "invalid_evidence_id",
                "Invalid runner evidence reference.",
            );
        }
        // Early rejection helps curated trees avoid opening known special files.
        // The opened-handle check below is still required; this is not a sandbox
        // for a hostile process racing replacements with devices or FIFOs.
        match self.root.metadata(path) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => return unsupported_type(),
            Err(error) => return io_failure(error.kind()),
        }
        let file = match self.root.open(path) {
            Ok(file) => file,
            Err(error) => return io_failure(error.kind()),
        };
        match file.metadata() {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => return unsupported_type(),
            Err(error) => return io_failure(error.kind()),
        }
        let read_limit = limit + 1;
        let mut bytes = Vec::with_capacity(read_limit);
        if let Err(error) = file.take(read_limit as u64).read_to_end(&mut bytes) {
            return io_failure(error.kind());
        }
        let text = match std::str::from_utf8(&bytes) {
            Ok(text) => text,
            Err(error) if error.error_len().is_none() && bytes.len() == read_limit => {
                // The read limit may cut a final multibyte character. Never use
                // replacement characters or pretend that its unseen bytes exist.
                std::str::from_utf8(&bytes[..error.valid_up_to()]).expect("validated UTF-8 prefix")
            }
            Err(_) => {
                return ToolResult::failure(
                    ToolStatus::Error,
                    "invalid_utf8",
                    "The observed file bytes are not valid UTF-8 text.",
                );
            }
        };
        let mut result = ToolResult::success(evidence_id);
        let overhead = result.encoded().expect("fixed result shape").len();
        let prefix = escaped_prefix(text, limit - overhead);
        result.body = prefix.to_owned();
        result.truncated = prefix.len() < bytes.len();
        debug_assert!(result.encoded().expect("fixed result shape").len() <= limit);
        result
    }

    fn list_files(&self, path: &str, limit: usize) -> ToolResult {
        let dir = match self.root.open_dir(path) {
            Ok(dir) => dir,
            Err(error) => return io_failure(error.kind()),
        };
        let entries = match dir.entries() {
            Ok(entries) => entries,
            Err(error) => return io_failure(error.kind()),
        };
        let mut result = ToolResult::success(None);
        let overhead = result.encoded().expect("fixed result shape").len();
        let body_budget = limit - overhead;
        let mut collected = Vec::new();
        let mut body_cost = 2; // The array's brackets, before escaping into body.
        let mut name_bytes = 0;
        let mut visited = 0;
        for item in entries.take(MAX_VISITED_ENTRIES) {
            visited += 1;
            let entry = match item {
                Ok(entry) => entry,
                Err(error) => return io_failure(error.kind()),
            };
            let raw_name = entry.file_name();
            let Some(name) = raw_name.to_str() else {
                return ToolResult::failure(
                    ToolStatus::Error,
                    "unsupported_name",
                    "A directory name cannot be represented as a UTF-8 access path.",
                );
            };
            let name = if path == "." {
                name.to_owned()
            } else {
                format!("{path}/{name}")
            };
            if normalized_path(&name).is_err() {
                return ToolResult::failure(
                    ToolStatus::Error,
                    "unsupported_name",
                    "A directory name cannot be represented as a supported access path.",
                );
            }
            let file_type = match entry.file_type() {
                Ok(kind) => kind,
                Err(error) => return io_failure(error.kind()),
            };
            let kind = if file_type.is_file() {
                "file"
            } else if file_type.is_dir() {
                "directory"
            } else if file_type.is_symlink() {
                "symlink"
            } else {
                "unsupported"
            };
            let entry = ListedEntry {
                name,
                kind: kind.to_owned(),
            };
            let encoded_entry = serde_json::to_string(&entry).expect("fixed listing shape");
            let cost = escaped_len(&encoded_entry) + usize::from(!collected.is_empty());
            if name_bytes + entry.name.len() > limit || body_cost + cost > body_budget {
                result.truncated = true;
                break;
            }
            name_bytes += entry.name.len();
            body_cost += cost;
            collected.push(entry);
        }
        // Without fetching entry 257 we cannot establish EOF at this boundary.
        result.truncated |= visited == MAX_VISITED_ENTRIES;
        collected.sort_by(|a, b| a.name.cmp(&b.name));
        result.body = serde_json::to_string(&collected).expect("fixed listing shape");
        debug_assert!(result.encoded().expect("fixed result shape").len() <= limit);
        result
    }

    /// Bounded literal search below `path`. Returns matched lines, never the
    /// whole file, and never an evidence reference: a partial view cannot
    /// certify that a field equals a file's value. Read the file to cite it.
    fn search_files(
        &self,
        path: &str,
        query: &str,
        case_sensitive: bool,
        limit: usize,
    ) -> ToolResult {
        // Fold the term once. Each line is folded only while it is examined.
        let needle = if case_sensitive {
            query.to_owned()
        } else {
            query.to_lowercase()
        };
        let mut result = ToolResult::success(None);
        let overhead = result.encoded().expect("fixed result shape").len();
        let body_budget = limit - overhead;
        let mut matches: Vec<SearchMatch> = Vec::new();
        let mut body_cost = 2; // The array's brackets, before escaping into body.
        let mut visited = 0usize;
        let mut incomplete = false;
        // Explicit stack instead of recursion: the depth bound is then a value
        // this function owns rather than a property of the call stack.
        let mut pending = vec![(path.to_owned(), 0usize)];
        while let Some((current, depth)) = pending.pop() {
            let dir = match self.root.open_dir(&current) {
                Ok(dir) => dir,
                // A directory that disappears mid-search is incompleteness, not
                // a failure of the whole search.
                Err(_) if current != path => {
                    incomplete = true;
                    continue;
                }
                Err(error) => return io_failure(error.kind()),
            };
            let entries = match dir.entries() {
                Ok(entries) => entries,
                Err(_) if current != path => {
                    incomplete = true;
                    continue;
                }
                Err(error) => return io_failure(error.kind()),
            };
            for item in entries {
                if visited >= MAX_VISITED_ENTRIES {
                    incomplete = true;
                    break;
                }
                visited += 1;
                let Ok(entry) = item else {
                    incomplete = true;
                    continue;
                };
                let raw_name = entry.file_name();
                let Some(name) = raw_name.to_str() else {
                    incomplete = true;
                    continue;
                };
                let joined = if current == "." {
                    name.to_owned()
                } else {
                    format!("{current}/{name}")
                };
                if normalized_path(&joined).is_err() {
                    incomplete = true;
                    continue;
                }
                let Ok(file_type) = entry.file_type() else {
                    incomplete = true;
                    continue;
                };
                if file_type.is_dir() {
                    if depth >= MAX_SEARCH_DEPTH {
                        incomplete = true;
                    } else {
                        pending.push((joined, depth + 1));
                    }
                    continue;
                }
                // Symlinks and devices are never followed or opened here. The
                // opened-handle check below still decides what was actually read.
                if !file_type.is_file() {
                    continue;
                }
                match self.scan_file(
                    &joined,
                    &needle,
                    case_sensitive,
                    &mut matches,
                    &mut body_cost,
                    body_budget,
                ) {
                    ScanOutcome::Continued => {}
                    ScanOutcome::Skipped => incomplete = true,
                    ScanOutcome::BudgetReached => {
                        incomplete = true;
                        pending.clear();
                        break;
                    }
                }
            }
        }
        matches.sort_by(|a, b| a.name.cmp(&b.name).then(a.line.cmp(&b.line)));
        result.truncated = incomplete;
        result.body = serde_json::to_string(&matches).expect("fixed match shape");
        debug_assert!(result.encoded().expect("fixed result shape").len() <= limit);
        result
    }

    fn scan_file(
        &self,
        name: &str,
        needle: &str,
        case_sensitive: bool,
        matches: &mut Vec<SearchMatch>,
        body_cost: &mut usize,
        body_budget: usize,
    ) -> ScanOutcome {
        let Ok(file) = self.root.open(name) else {
            return ScanOutcome::Skipped;
        };
        match file.metadata() {
            Ok(metadata) if metadata.is_file() => {}
            _ => return ScanOutcome::Skipped,
        }
        let mut bytes = Vec::new();
        if file
            .take(MAX_SEARCH_FILE_BYTES as u64)
            .read_to_end(&mut bytes)
            .is_err()
        {
            return ScanOutcome::Skipped;
        }
        // Binary or non-UTF-8 content is reported as incompleteness rather than
        // searched with replacement characters that were never in the file.
        let Ok(text) = std::str::from_utf8(&bytes) else {
            return ScanOutcome::Skipped;
        };
        for (index, line) in text.lines().enumerate() {
            // Case folding can change byte length, so it decides the match only.
            // The reported text is always the line as the file actually holds it.
            let matched = if case_sensitive {
                line.contains(needle)
            } else {
                line.to_lowercase().contains(needle)
            };
            if !matched {
                continue;
            }
            let candidate = SearchMatch {
                name: name.to_owned(),
                line: index + 1,
                text: escaped_prefix(line, MAX_QUERY_BYTES * 4).to_owned(),
            };
            let encoded = serde_json::to_string(&candidate).expect("fixed match shape");
            let cost = escaped_len(&encoded) + usize::from(!matches.is_empty());
            if *body_cost + cost > body_budget {
                return ScanOutcome::BudgetReached;
            }
            *body_cost += cost;
            matches.push(candidate);
        }
        ScanOutcome::Continued
    }
}

enum ScanOutcome {
    Continued,
    Skipped,
    BudgetReached,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SearchMatch {
    pub name: String,
    /// One-based, matching how a person reads a file.
    pub line: usize,
    pub text: String,
}

/// A write capability, distinct from WorkspaceReader by construction: the read
/// tools have no method that can change a file. One is built only for a
/// workspace whose operator listed a write tool, so a run that was never granted
/// writes has no writer at all.
pub struct WorkspaceWriter {
    root: Dir,
}

impl WorkspaceWriter {
    /// Trusted startup only; never call with a path from a model or submission.
    pub fn open(root: &Path) -> Result<Self, String> {
        Dir::open_ambient_dir(root, ambient_authority())
            .map(|root| Self { root })
            .map_err(|_| "cannot open approved workspace directory".into())
    }

    /// Run one authorized mutating tool. The runner routes here only for a tool
    /// whose arguments its shape check already validated for this variant.
    pub fn execute(&self, name: ToolName, args: &TypedToolArgs) -> ToolResult {
        match name {
            ToolName::WriteFile => {
                self.write_file(&args.path, args.content.as_deref().unwrap_or(""))
            }
            ToolName::EditFile => self.edit_file(
                &args.path,
                args.find.as_deref().unwrap_or(""),
                args.replace.as_deref().unwrap_or(""),
            ),
            ToolName::DeleteFile => self.delete_file(&args.path),
            ToolName::MoveFile => self.move_file(&args.path, args.to.as_deref().unwrap_or("")),
            // The reader owns the read tools; a non-mutating name here is a fault.
            _ => ToolResult::failure(
                ToolStatus::Denied,
                "not_a_write_tool",
                "This capability only writes.",
            ),
        }
    }

    /// Replace or create one regular file inside the capability.
    pub fn write_file(&self, path: &str, content: &str) -> ToolResult {
        if content.len() > MAX_WRITE_BYTES {
            return ToolResult::failure(
                ToolStatus::Denied,
                "content_too_large",
                "Write content exceeds its byte limit.",
            );
        }
        match self.target_state(path, false) {
            Ok(_) => {}
            Err(denial) => return denial,
        }
        self.atomic_replace(path, content, "wrote")
    }

    /// Replace exactly one occurrence of `find` with `replace` in an existing
    /// file. The match must be unique: an absent match cannot edit, and an
    /// ambiguous one is refused rather than guessed, so the change is exact.
    pub fn edit_file(&self, path: &str, find: &str, replace: &str) -> ToolResult {
        if find.is_empty() {
            return ToolResult::failure(
                ToolStatus::Denied,
                "empty_find",
                "The text to find cannot be empty.",
            );
        }
        if find.len() > MAX_WRITE_BYTES || replace.len() > MAX_WRITE_BYTES {
            return ToolResult::failure(
                ToolStatus::Denied,
                "content_too_large",
                "The find or replace text exceeds its byte limit.",
            );
        }
        match self.target_state(path, true) {
            Ok(true) => {}
            Ok(false) => return io_failure(ErrorKind::NotFound),
            Err(denial) => return denial,
        }
        // Read a bounded prefix; a file larger than the write bound cannot be
        // edited into a bounded result, so refuse it rather than truncate.
        let mut bytes = Vec::new();
        let file = match self.root.open(path) {
            Ok(file) => file,
            Err(error) => return io_failure(error.kind()),
        };
        if let Err(error) = file
            .take(MAX_WRITE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
        {
            return io_failure(error.kind());
        }
        if bytes.len() > MAX_WRITE_BYTES {
            return ToolResult::failure(
                ToolStatus::Denied,
                "file_too_large",
                "The file is larger than the editable byte limit.",
            );
        }
        let text = match std::str::from_utf8(&bytes) {
            Ok(text) => text,
            Err(_) => {
                return ToolResult::failure(
                    ToolStatus::Error,
                    "invalid_utf8",
                    "The file is not valid UTF-8 text.",
                );
            }
        };
        let occurrences = text.matches(find).count();
        if occurrences == 0 {
            return ToolResult::failure(
                ToolStatus::Error,
                "match_not_found",
                "The text to find does not appear in the file.",
            );
        }
        if occurrences > 1 {
            return ToolResult::failure(
                ToolStatus::Denied,
                "ambiguous_match",
                "The text to find appears more than once; make it unique.",
            );
        }
        let updated = text.replacen(find, replace, 1);
        if updated.len() > MAX_WRITE_BYTES {
            return ToolResult::failure(
                ToolStatus::Denied,
                "content_too_large",
                "The edited file would exceed its byte limit.",
            );
        }
        self.atomic_replace(path, &updated, "edited")
    }

    /// Remove one regular file. Only a regular file: a symbolic link or a
    /// directory is refused, so a delete never removes curated structure or
    /// reaches outside the root, and a missing file is an error, not a success.
    pub fn delete_file(&self, path: &str) -> ToolResult {
        match self.target_state(path, true) {
            Ok(true) => {}
            Ok(false) => return io_failure(ErrorKind::NotFound),
            Err(denial) => return denial,
        }
        if let Err(error) = self.root.remove_file(path) {
            return io_failure(error.kind());
        }
        let mut result = ToolResult::success(None);
        result.body =
            serde_json::to_string(&json!({ "deleted": path })).expect("fixed delete result shape");
        result
    }

    /// Rename one regular file inside the capability. The source must be a
    /// regular file; the destination must not already exist, so a move never
    /// silently overwrites another file. The rename is atomic.
    pub fn move_file(&self, from: &str, to: &str) -> ToolResult {
        match self.target_state(from, true) {
            Ok(true) => {}
            Ok(false) => return io_failure(ErrorKind::NotFound),
            Err(denial) => return denial,
        }
        match self.target_state(to, false) {
            Ok(false) => {}
            Ok(true) => {
                return ToolResult::failure(
                    ToolStatus::Denied,
                    "destination_exists",
                    "The destination already exists; move never overwrites.",
                );
            }
            Err(denial) => return denial,
        }
        if let Err(error) = self.root.rename(from, &self.root, to) {
            return io_failure(error.kind());
        }
        let mut result = ToolResult::success(None);
        result.body = serde_json::to_string(&json!({ "moved": from, "to": to }))
            .expect("fixed move result shape");
        result
    }

    /// Refuse to write through a symlink or over a directory. Returns whether
    /// the target already exists as a regular file. When `must_exist`, a missing
    /// target is the caller's error; otherwise it is a create, and the parent
    /// must already be a directory.
    fn target_state(&self, path: &str, must_exist: bool) -> Result<bool, ToolResult> {
        if normalized_path(path).is_err() {
            return Err(ToolResult::failure(
                ToolStatus::Denied,
                "invalid_path",
                "Use a supported relative workspace path.",
            ));
        }
        match self.root.symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => return Ok(true),
            Ok(metadata) if metadata.is_symlink() => {
                return Err(ToolResult::failure(
                    ToolStatus::Denied,
                    "symlink_target",
                    "Refusing to write through a symbolic link.",
                ));
            }
            Ok(_) => {
                return Err(ToolResult::failure(
                    ToolStatus::Denied,
                    "not_a_file",
                    "The path is not a regular file.",
                ));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if must_exist {
                    return Ok(false);
                }
            }
            Err(error) => return Err(io_failure(error.kind())),
        }
        // Creating: the parent must already exist. Making directories is a
        // separate effect a later tool can own explicitly.
        let (parent, _) = path.rsplit_once('/').unwrap_or((".", path));
        if parent != "." {
            match self.root.metadata(parent) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(ToolResult::failure(
                        ToolStatus::Denied,
                        "parent_not_a_directory",
                        "The parent path is not a directory.",
                    ));
                }
                Err(error) => return Err(io_failure(error.kind())),
            }
        }
        Ok(false)
    }

    /// Atomic: content lands in a temporary sibling and is renamed into place,
    /// so a crash leaves either the old file or the new one, never a partial.
    fn atomic_replace(&self, path: &str, content: &str, verb: &str) -> ToolResult {
        let temporary = format!("{path}.kinesin-{}.tmp", uuid::Uuid::new_v4());
        if let Err(error) = self.root.write(&temporary, content.as_bytes()) {
            let _ = self.root.remove_file(&temporary);
            return io_failure(error.kind());
        }
        if let Err(error) = self.root.rename(&temporary, &self.root, path) {
            let _ = self.root.remove_file(&temporary);
            return io_failure(error.kind());
        }
        let mut result = ToolResult::success(None);
        result.body = serde_json::to_string(&json!({ verb: path, "bytes": content.len() }))
            .expect("fixed write result shape");
        result
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListedEntry {
    pub name: String,
    pub kind: String,
}

fn unsupported_type() -> ToolResult {
    ToolResult::failure(
        ToolStatus::Error,
        "unsupported_type",
        "Only regular UTF-8 text files can be read.",
    )
}

fn io_failure(kind: ErrorKind) -> ToolResult {
    match kind {
        ErrorKind::NotFound => ToolResult::failure(
            ToolStatus::Error,
            "not_found",
            "The requested workspace resource was not found.",
        ),
        ErrorKind::PermissionDenied => ToolResult::failure(
            ToolStatus::Denied,
            "access_denied",
            "The requested resource is not accessible within this workspace.",
        ),
        _ => ToolResult::failure(
            ToolStatus::Error,
            "io_error",
            "The workspace operation could not be completed.",
        ),
    }
}

fn escaped_char_len(character: char) -> usize {
    match character {
        '"' | '\\' | '\u{08}' | '\t' | '\n' | '\u{0c}' | '\r' => 2,
        '\0'..='\u{1f}' => 6,
        _ => character.len_utf8(),
    }
}

fn escaped_len(text: &str) -> usize {
    text.chars().map(escaped_char_len).sum()
}

fn escaped_prefix(text: &str, max_escaped_bytes: usize) -> &str {
    let mut used = 0;
    for (offset, character) in text.char_indices() {
        used += escaped_char_len(character);
        if used > max_escaped_bytes {
            return &text[..offset];
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct Fixture {
        root: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("kinesin-tools-tests-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(root.join("workspace/nested")).unwrap();
            Self { root }
        }
        fn reader(&self) -> WorkspaceReader {
            WorkspaceReader::open(&self.root.join("workspace")).unwrap()
        }
        fn writer(&self) -> WorkspaceWriter {
            WorkspaceWriter::open(&self.root.join("workspace")).unwrap()
        }
        fn read_back(&self, name: &str) -> String {
            std::fs::read_to_string(self.root.join("workspace").join(name)).unwrap()
        }
        fn tmp_files(&self) -> Vec<String> {
            std::fs::read_dir(self.root.join("workspace"))
                .unwrap()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.contains(".kinesin-"))
                .collect()
        }
        fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
            std::fs::write(self.root.join("workspace").join(name), bytes).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert!(self.root.starts_with(std::env::temp_dir()));
            assert!(
                self.root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("kinesin-tools-tests-")
            );
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn search(reader: &WorkspaceReader, path: &str, query: &str) -> ToolResult {
        search_cased(reader, path, query, false)
    }

    fn search_cased(
        reader: &WorkspaceReader,
        path: &str,
        query: &str,
        case_sensitive: bool,
    ) -> ToolResult {
        reader.execute_with_query(
            ToolName::SearchFiles,
            path,
            Some(query),
            case_sensitive,
            MAX_TOOL_BYTES,
            None,
        )
    }

    fn matches_of(result: &ToolResult) -> Vec<SearchMatch> {
        serde_json::from_str(&result.body).expect("search body is a match array")
    }

    #[test]
    fn search_reports_matching_files_and_one_based_lines() {
        let fixture = Fixture::new();
        fixture.write("alpha.txt", "one\nlanguage=Rust\nthree\n");
        fixture.write("beta.txt", "nothing here\n");
        fixture.write("nested/gamma.txt", "language=Rust again\n");
        let reader = fixture.reader();

        let result = search(&reader, ".", "language=");
        assert_eq!(result.status, ToolStatus::Ok);
        let found = matches_of(&result);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "alpha.txt");
        assert_eq!(found[0].line, 2, "line numbers are one-based");
        assert_eq!(found[0].text, "language=Rust");
        assert_eq!(found[1].name, "nested/gamma.txt");
        assert!(!result.truncated);

        // A term that appears nowhere is an empty result, not an error.
        let empty = search(&reader, ".", "absent-term");
        assert_eq!(empty.status, ToolStatus::Ok);
        assert!(matches_of(&empty).is_empty());
    }

    #[test]
    fn search_folds_case_by_default_and_stays_exact_on_request() {
        let fixture = Fixture::new();
        fixture.write("readme.md", "Language=Rust\nSTATUS=Ready\n");
        let reader = fixture.reader();

        // Lexical search fails on a casing guess, and a smaller model refines a
        // failed query least reliably, so the default match forgives casing.
        let folded = search(&reader, ".", "language=");
        assert_eq!(matches_of(&folded).len(), 1);
        assert_eq!(
            matches_of(&folded)[0].text,
            "Language=Rust",
            "the reported line is the file's bytes, not the folded form"
        );
        assert_eq!(matches_of(&search(&reader, ".", "STATUS")).len(), 1);
        assert_eq!(matches_of(&search(&reader, ".", "status")).len(), 1);

        // Exactness stays available for a caller that needs it.
        let exact = search_cased(&reader, ".", "language=", true);
        assert!(matches_of(&exact).is_empty());
        assert_eq!(
            matches_of(&search_cased(&reader, ".", "Language=", true)).len(),
            1
        );

        let parsed =
            TypedToolArgs::parse(r#"{"path":".","query":"x","case_sensitive":true}"#).unwrap();
        assert_eq!(parsed.case_sensitive, Some(true));
        assert_eq!(
            TypedToolArgs::parse(r#"{"path":".","query":"x"}"#)
                .unwrap()
                .case_sensitive,
            None,
            "an absent flag means the forgiving default"
        );
    }

    #[test]
    fn search_never_mints_evidence_a_candidate_could_cite() {
        let fixture = Fixture::new();
        fixture.write("alpha.txt", "language=Rust\n");
        let reader = fixture.reader();

        // Even when the runner offers an identifier, a partial view must not
        // carry one: only a complete successful read can certify a value.
        let result = reader.execute_with_query(
            ToolName::SearchFiles,
            ".",
            Some("language="),
            false,
            MAX_TOOL_BYTES,
            Some("e0"),
        );
        assert_eq!(result.status, ToolStatus::Ok);
        assert_eq!(result.evidence_id, None);
        assert!(!ToolName::SearchFiles.mints_evidence());
        assert!(!ToolName::ListFiles.mints_evidence());
        assert!(ToolName::ReadFile.mints_evidence());

        let read = reader.execute(ToolName::ReadFile, "alpha.txt", MAX_TOOL_BYTES, Some("e0"));
        assert_eq!(read.evidence_id.as_deref(), Some("e0"));
    }

    #[test]
    fn search_requires_its_own_term_and_refuses_one_on_other_tools() {
        let fixture = Fixture::new();
        fixture.write("alpha.txt", "language=Rust\n");
        let reader = fixture.reader();

        let missing = reader.execute_with_query(
            ToolName::SearchFiles,
            ".",
            None,
            false,
            MAX_TOOL_BYTES,
            None,
        );
        assert_eq!(missing.status, ToolStatus::Denied);
        assert_eq!(missing.error.unwrap().code, "missing_query");

        for name in [ToolName::ReadFile, ToolName::ListFiles] {
            let unexpected = reader.execute_with_query(
                name,
                "alpha.txt",
                Some("language="),
                false,
                MAX_TOOL_BYTES,
                None,
            );
            assert_eq!(unexpected.status, ToolStatus::Denied);
            assert_eq!(unexpected.error.unwrap().code, "unexpected_query");
        }

        for bad in [
            "",
            &"q".repeat(MAX_QUERY_BYTES + 1),
            "line\nbreak",
            "tab\there",
        ] {
            assert!(validated_query(bad).is_err(), "{bad:?}");
            let denied = search(&reader, ".", bad);
            assert_eq!(denied.status, ToolStatus::Denied, "{bad:?}");
        }
        assert!(TypedToolArgs::parse(r#"{"path":".","query":""}"#).is_err());
        let parsed = TypedToolArgs::parse(r#"{"path":".","query":"needle"}"#).unwrap();
        assert_eq!(parsed.query.as_deref(), Some("needle"));
    }

    #[test]
    fn search_stays_inside_the_capability_and_skips_unreadable_content() {
        let fixture = Fixture::new();
        fixture.write("alpha.txt", "language=Rust\n");
        // Invalid UTF-8 is skipped and reported, never searched as replacement text.
        fixture.write("binary.bin", [b'l', b'a', b'n', b'g', 0xff, 0xfe]);
        std::fs::write(fixture.root.join("outside-secret.txt"), "language=Secret\n").unwrap();
        let reader = fixture.reader();

        let escaping = search(&reader, "../", "language=");
        assert_eq!(escaping.status, ToolStatus::Denied);
        assert_eq!(escaping.error.unwrap().code, "invalid_path");

        let result = search(&reader, ".", "lang");
        let names: Vec<_> = matches_of(&result).into_iter().map(|m| m.name).collect();
        assert_eq!(names, vec!["alpha.txt".to_owned()]);
        assert!(
            !result.body.contains("Secret"),
            "a search must not reach outside its root"
        );
        assert!(result.truncated, "skipping unreadable content is reported");
    }

    #[test]
    fn search_bounds_depth_and_result_bytes_without_failing() {
        let fixture = Fixture::new();
        let deep = fixture.root.join("workspace").join("a/b/c/d/e/f");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("deep.txt"), "needle\n").unwrap();
        fixture.write("shallow.txt", "needle\n");
        let reader = fixture.reader();

        let bounded = search(&reader, ".", "needle");
        assert_eq!(bounded.status, ToolStatus::Ok);
        let names: Vec<_> = matches_of(&bounded).into_iter().map(|m| m.name).collect();
        assert!(names.contains(&"shallow.txt".to_owned()));
        assert!(
            !names.iter().any(|n| n.contains("f/deep.txt")),
            "the depth bound stops descent"
        );
        assert!(bounded.truncated, "an unreached subtree is incompleteness");

        // Many matches must fit the caller's budget rather than overflow it.
        fixture.write("many.txt", "needle\n".repeat(4_000));
        let capped = reader.execute_with_query(
            ToolName::SearchFiles,
            ".",
            Some("needle"),
            false,
            1_024,
            None,
        );
        assert_eq!(capped.status, ToolStatus::Ok);
        assert!(capped.encoded().unwrap().len() <= 1_024);
        assert!(capped.truncated);
    }

    #[test]
    fn write_creates_and_replaces_a_file_atomically() {
        let fixture = Fixture::new();
        let writer = fixture.writer();

        let created = writer.write_file(
            "note.txt", "first
",
        );
        assert_eq!(created.status, ToolStatus::Ok);
        assert_eq!(created.evidence_id, None, "a write cites no evidence");
        let body: serde_json::Value = serde_json::from_str(&created.body).unwrap();
        assert_eq!(body["wrote"], "note.txt");
        assert_eq!(body["bytes"], 6);
        assert_eq!(
            fixture.read_back("note.txt"),
            "first
"
        );

        let replaced = writer.write_file(
            "note.txt",
            "second longer
",
        );
        assert_eq!(replaced.status, ToolStatus::Ok);
        assert_eq!(
            fixture.read_back("note.txt"),
            "second longer
"
        );
        assert!(
            fixture.tmp_files().is_empty(),
            "the atomic temporary is never left behind"
        );

        // A nested write lands where a directory already exists.
        writer.write_file("nested/deep.txt", "x").assert_ok();
        assert_eq!(fixture.read_back("nested/deep.txt"), "x");
    }

    #[test]
    fn write_refuses_to_leave_the_root_or_clobber_structure() {
        let fixture = Fixture::new();
        std::fs::write(fixture.root.join("outside-secret.txt"), "old").unwrap();
        std::fs::create_dir(fixture.root.join("workspace/adir")).unwrap();
        let workspace = fixture.root.join("workspace");
        // A symlink inside the root pointing outside it.
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            fixture.root.join("outside-secret.txt"),
            workspace.join("link"),
        )
        .unwrap();
        #[cfg(windows)]
        let _ = std::os::windows::fs::symlink_file(
            fixture.root.join("outside-secret.txt"),
            workspace.join("link"),
        );
        let writer = fixture.writer();

        for (path, status, code) in [
            ("../outside-secret.txt", ToolStatus::Denied, "invalid_path"),
            ("adir", ToolStatus::Denied, "not_a_file"),
            // A missing parent is an I/O outcome, not a policy denial.
            ("missing_dir/child.txt", ToolStatus::Error, "not_found"),
        ] {
            let denied = writer.write_file(path, "attempt");
            assert_eq!(denied.status, status, "{path}");
            assert_eq!(denied.error.unwrap().code, code, "{path}");
        }
        assert_eq!(
            std::fs::read_to_string(fixture.root.join("outside-secret.txt")).unwrap(),
            "old",
            "nothing outside the root was touched"
        );

        // Writing through the escaping symlink is refused where symlinks exist.
        #[cfg(unix)]
        {
            let denied = writer.write_file("link", "attempt");
            assert_eq!(denied.status, ToolStatus::Denied);
            assert_eq!(denied.error.unwrap().code, "symlink_target");
            assert_eq!(
                std::fs::read_to_string(fixture.root.join("outside-secret.txt")).unwrap(),
                "old"
            );
        }
    }

    #[test]
    fn edit_replaces_a_unique_passage_and_refuses_an_ambiguous_one() {
        let fixture = Fixture::new();
        fixture.write(
            "src.txt",
            "language=Rust
edition=2024
language=note
",
        );
        let writer = fixture.writer();

        // A unique match is replaced exactly.
        let edited = writer.edit_file("src.txt", "edition=2024", "edition=2025");
        assert_eq!(edited.status, ToolStatus::Ok);
        let body: serde_json::Value = serde_json::from_str(&edited.body).unwrap();
        assert_eq!(body["edited"], "src.txt");
        assert_eq!(
            fixture.read_back("src.txt"),
            "language=Rust
edition=2025
language=note
"
        );
        assert!(fixture.tmp_files().is_empty());

        // "language=" appears twice: refuse rather than guess which.
        let ambiguous = writer.edit_file("src.txt", "language=", "lang=");
        assert_eq!(ambiguous.status, ToolStatus::Denied);
        assert_eq!(ambiguous.error.unwrap().code, "ambiguous_match");
        // An absent passage cannot edit, and the file is unchanged.
        let missing = writer.edit_file("src.txt", "absent text", "x");
        assert_eq!(missing.status, ToolStatus::Error);
        assert_eq!(missing.error.unwrap().code, "match_not_found");
        assert_eq!(
            fixture.read_back("src.txt"),
            "language=Rust
edition=2025
language=note
",
            "a refused edit changes nothing"
        );
    }

    #[test]
    fn edit_needs_an_existing_file_and_stays_bounded() {
        let fixture = Fixture::new();
        let writer = fixture.writer();

        // Editing a file that does not exist is a not-found, not a create.
        let absent = writer.edit_file("missing.txt", "a", "b");
        assert_eq!(absent.status, ToolStatus::Error);
        assert_eq!(absent.error.unwrap().code, "not_found");

        // Empty find is refused.
        fixture.write("f.txt", "body");
        assert_eq!(
            writer.edit_file("f.txt", "", "x").error.unwrap().code,
            "empty_find"
        );

        // A file larger than the editable bound is refused rather than truncated.
        fixture.write("big.txt", "x".repeat(MAX_WRITE_BYTES + 1));
        let denied = writer.edit_file("big.txt", "x", "y");
        assert_eq!(denied.status, ToolStatus::Denied);
        assert_eq!(denied.error.unwrap().code, "file_too_large");

        // A replacement that would push the file past the bound is refused.
        let headroom = MAX_WRITE_BYTES - 4;
        fixture.write("grow.txt", format!("{}MARK", "a".repeat(headroom)));
        let grown = writer.edit_file("grow.txt", "MARK", &"z".repeat(64));
        assert_eq!(grown.status, ToolStatus::Denied);
        assert_eq!(grown.error.unwrap().code, "content_too_large");

        // An escaping path is rejected at parse, before any dispatch. Called
        // directly, the same guard denies it inside the write capability.
        assert!(TypedToolArgs::parse(r#"{"path":"../x","find":"a","replace":"b"}"#).is_err());
        let escaping = writer.edit_file("../x", "a", "b");
        assert_eq!(escaping.status, ToolStatus::Denied);
        assert_eq!(escaping.error.unwrap().code, "invalid_path");
    }

    #[test]
    fn write_bounds_content_and_a_reader_cannot_write() {
        let fixture = Fixture::new();
        let writer = fixture.writer();

        let oversized = writer.write_file("big.txt", &"x".repeat(MAX_WRITE_BYTES + 1));
        assert_eq!(oversized.status, ToolStatus::Denied);
        assert_eq!(oversized.error.unwrap().code, "content_too_large");
        assert!(!fixture.root.join("workspace/big.txt").exists());

        // The read capability has no path to a write, even for write_file.
        let denied =
            fixture
                .reader()
                .execute(ToolName::WriteFile, "note.txt", MAX_TOOL_BYTES, None);
        assert_eq!(denied.status, ToolStatus::Denied);
        assert_eq!(denied.error.unwrap().code, "not_a_write_capability");
    }

    #[test]
    fn delete_removes_a_regular_file_and_refuses_structure() {
        let fixture = Fixture::new();
        fixture.write("gone.txt", "bye");
        std::fs::create_dir(fixture.root.join("workspace/adir")).unwrap();
        let workspace = fixture.root.join("workspace");
        std::fs::write(fixture.root.join("outside.txt"), "keep").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(fixture.root.join("outside.txt"), workspace.join("link"))
            .unwrap();
        let writer = fixture.writer();

        assert_eq!(writer.delete_file("gone.txt").status, ToolStatus::Ok);
        assert!(!workspace.join("gone.txt").exists());

        // A directory, a missing file, and an escaping path are all refused.
        assert_eq!(writer.delete_file("adir").error.unwrap().code, "not_a_file");
        assert_eq!(
            writer.delete_file("absent.txt").error.unwrap().code,
            "not_found"
        );
        assert_eq!(
            writer.delete_file("../outside.txt").error.unwrap().code,
            "invalid_path"
        );
        #[cfg(unix)]
        {
            // Deleting a symlink is refused, and its target survives.
            assert_eq!(
                writer.delete_file("link").error.unwrap().code,
                "symlink_target"
            );
            assert!(workspace.join("link").exists());
            assert_eq!(
                std::fs::read_to_string(fixture.root.join("outside.txt")).unwrap(),
                "keep"
            );
        }
    }

    #[test]
    fn move_renames_a_file_and_never_overwrites() {
        let fixture = Fixture::new();
        fixture.write("from.txt", "payload");
        fixture.write("occupied.txt", "existing");
        std::fs::create_dir(fixture.root.join("workspace/sub")).unwrap();
        let writer = fixture.writer();
        let workspace = fixture.root.join("workspace");

        // A plain rename into an existing directory.
        assert_eq!(
            writer.move_file("from.txt", "sub/to.txt").status,
            ToolStatus::Ok
        );
        assert!(!workspace.join("from.txt").exists());
        assert_eq!(fixture.read_back("sub/to.txt"), "payload");

        // The source must exist; a missing source is not-found.
        assert_eq!(
            writer
                .move_file("from.txt", "elsewhere.txt")
                .error
                .unwrap()
                .code,
            "not_found"
        );
        // Never overwrite: an existing destination is refused, both files intact.
        assert_eq!(
            writer
                .move_file("sub/to.txt", "occupied.txt")
                .error
                .unwrap()
                .code,
            "destination_exists"
        );
        assert_eq!(fixture.read_back("occupied.txt"), "existing");
        assert_eq!(fixture.read_back("sub/to.txt"), "payload");
        // A destination whose parent is missing is not-found; escaping is denied.
        assert_eq!(
            writer
                .move_file("sub/to.txt", "nodir/x.txt")
                .error
                .unwrap()
                .code,
            "not_found"
        );
        assert!(TypedToolArgs::parse(r#"{"path":"a","to":"../x"}"#).is_err());
    }

    #[test]
    fn shape_gates_every_tool_field() {
        // move needs a destination and nothing else; delete needs only a path.
        let mv = TypedToolArgs::parse(r#"{"path":"a","to":"b"}"#).unwrap();
        assert!(mv.shape_for(ToolName::MoveFile).is_ok());
        assert!(mv.shape_for(ToolName::DeleteFile).is_err());
        let del = TypedToolArgs::parse(r#"{"path":"a"}"#).unwrap();
        assert!(del.shape_for(ToolName::DeleteFile).is_ok());
        assert!(del.shape_for(ToolName::MoveFile).is_err());
        // A destination on a non-move tool is rejected.
        assert!(mv.shape_for(ToolName::WriteFile).is_err());
    }

    #[test]
    fn edit_argument_shape_requires_both_find_and_replace() {
        let full = TypedToolArgs::parse(r#"{"path":"a","find":"x","replace":"y"}"#).unwrap();
        assert!(full.shape_for(ToolName::EditFile).is_ok());
        // find/replace on a non-edit tool, or a half edit, is refused.
        assert!(full.shape_for(ToolName::WriteFile).is_err());
        let half = TypedToolArgs::parse(r#"{"path":"a","find":"x"}"#).unwrap();
        assert!(half.shape_for(ToolName::EditFile).is_err());
        // Oversized find is refused at parse.
        let huge = format!(
            r#"{{"path":"a","find":"{}","replace":"y"}}"#,
            "x".repeat(MAX_WRITE_BYTES + 1)
        );
        assert!(TypedToolArgs::parse(&huge).is_err());
    }

    #[test]
    fn argument_shape_matches_each_tool() {
        let write = TypedToolArgs::parse(r#"{"path":"a.txt","content":"hi"}"#).unwrap();
        assert!(write.shape_for(ToolName::WriteFile).is_ok());
        // Content on a read, or a missing write body, must be rejected.
        assert!(write.shape_for(ToolName::ReadFile).is_err());
        let read = TypedToolArgs::parse(r#"{"path":"a.txt"}"#).unwrap();
        assert!(read.shape_for(ToolName::WriteFile).is_err());
        assert!(read.shape_for(ToolName::ReadFile).is_ok());
        // Oversized content is refused at parse.
        let huge = format!(
            r#"{{"path":"a.txt","content":"{}"}}"#,
            "x".repeat(MAX_WRITE_BYTES + 1)
        );
        assert!(TypedToolArgs::parse(&huge).is_err());
    }

    #[test]
    fn arguments_deny_unknown_duplicate_and_escaping_paths() {
        for raw in [
            r#"{"path":"x","workspace":"other"}"#,
            r#"{"path":"x","path":"y"}"#,
            r#"{"path":"../secret"}"#,
            r#"{"path":7}"#,
            "{",
        ] {
            assert!(TypedToolArgs::parse(raw).is_err());
        }
        for path in [
            "/x",
            "C:/x",
            "C:x",
            "\\\\server\\x",
            "file:stream",
            "NUL",
            "COM¹",
            "a?b",
            "x\0",
        ] {
            assert!(normalized_path(path).is_err(), "{path:?}");
        }
        assert_eq!(TypedToolArgs::parse(r#"{"path":"."}"#).unwrap().path, ".");
    }

    #[test]
    fn real_empty_and_nested_reads_bind_only_successful_evidence() {
        let fixture = Fixture::new();
        fixture.write("empty", b"");
        fixture.write("nested/note", "language=Rust\n");
        let reader = fixture.reader();
        let empty = reader.execute(ToolName::ReadFile, "empty", 256, Some("e0"));
        assert_eq!(empty.status, ToolStatus::Ok);
        assert!(!empty.truncated);
        assert_eq!(empty.body, "");
        let full = reader.execute(ToolName::ReadFile, "nested/note", 256, Some("e1"));
        assert_eq!(full.body, "language=Rust\n");
        assert_eq!(full.evidence_id.as_deref(), Some("e1"));
        assert!(!full.truncated);
        for path in ["missing", "nested", "../outside"] {
            let error = reader.execute(ToolName::ReadFile, path, 256, Some("e2"));
            assert_ne!(error.status, ToolStatus::Ok);
            assert!(error.evidence_id.is_none());
            assert!(error.body.is_empty());
            assert!(error.encoded().unwrap().len() <= 256);
            assert!(
                !error
                    .encoded()
                    .unwrap()
                    .contains(&fixture.root.to_string_lossy().to_string())
            );
        }
    }

    #[test]
    fn escaping_and_unicode_prefixes_fit_the_actual_serialized_envelope() {
        let fixture = Fixture::new();
        let reader = fixture.reader();
        for content in [
            "x".repeat(100_000),
            "\"\\\n\0".repeat(10_000),
            "🦀".repeat(10_000),
        ] {
            fixture.write("large", &content);
            let result = reader.execute(ToolName::ReadFile, "large", 256, Some("e0"));
            assert_eq!(result.status, ToolStatus::Ok);
            assert!(result.truncated);
            assert!(content.starts_with(&result.body));
            assert!(result.encoded().unwrap().len() <= 256);
        }
        fixture.write("bad-interior", [b'a', 0xff, b'z']);
        fixture.write("bad-tail", [b'a', 0xf0, 0x9f]);
        for path in ["bad-interior", "bad-tail"] {
            let result = reader.execute(ToolName::ReadFile, path, 256, Some("e1"));
            assert_eq!(result.error.unwrap().code, "invalid_utf8");
            assert!(result.evidence_id.is_none());
        }
    }

    #[test]
    fn listing_sorts_only_whole_collected_names_and_reports_incompleteness() {
        let fixture = Fixture::new();
        fixture.write("nested/z.txt", "z");
        fixture.write("nested/a.txt", "a");
        let reader = fixture.reader();
        let result = reader.execute(ToolName::ListFiles, "nested", 8192, Some("ignored"));
        let entries: Vec<ListedEntry> = serde_json::from_str(&result.body).unwrap();
        assert_eq!(
            entries.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(),
            ["nested/a.txt", "nested/z.txt"]
        );
        assert!(!result.truncated);
        assert!(result.evidence_id.is_none());
        for i in 0..300 {
            fixture.write(&format!("file-{i:03}-long-name.txt"), "");
        }
        let large = reader.execute(ToolName::ListFiles, ".", 256, None);
        assert!(large.truncated);
        assert!(large.encoded().unwrap().len() <= 256);
        assert!(serde_json::from_str::<Vec<ListedEntry>>(&large.body).is_ok());
    }

    #[test]
    fn capability_denies_a_link_to_an_outside_sentinel() {
        let fixture = Fixture::new();
        let outside = fixture.root.join("outside-secret");
        std::fs::write(&outside, "OUTSIDE-SENTINEL").unwrap();
        let link = fixture.root.join("workspace/escape");
        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&outside, &link);
        #[cfg(unix)]
        let created = std::os::unix::fs::symlink(&outside, &link);
        match created {
            Ok(()) => {}
            Err(error)
                if error.kind() == ErrorKind::PermissionDenied
                    || error.raw_os_error() == Some(1314) =>
            {
                eprintln!(
                    "UNAVAILABLE: native symlink confinement test lacks OS symlink privilege: {error}"
                );
                return;
            }
            Err(error) => panic!("cannot prepare symlink fixture: {error}"),
        }
        let result = fixture
            .reader()
            .execute(ToolName::ReadFile, "escape", 8192, Some("e0"));
        assert_ne!(result.status, ToolStatus::Ok);
        assert!(result.body.is_empty());
        assert!(result.evidence_id.is_none());
    }

    #[test]
    fn capability_denies_outside_reads_while_a_regular_path_becomes_a_link() {
        use std::sync::Barrier;

        let fixture = Fixture::new();
        let outside = fixture.root.join("outside-secret");
        let changing = fixture.root.join("workspace/changing");
        std::fs::write(&outside, "OUTSIDE-SENTINEL").unwrap();
        let reader = fixture.reader();
        #[cfg(windows)]
        fn make_link(source: &Path, destination: &Path) -> std::io::Result<()> {
            std::os::windows::fs::symlink_file(source, destination)
        }
        #[cfg(unix)]
        fn make_link(source: &Path, destination: &Path) -> std::io::Result<()> {
            std::os::unix::fs::symlink(source, destination)
        }
        match make_link(&outside, &changing) {
            Ok(()) => {}
            Err(error)
                if error.kind() == ErrorKind::PermissionDenied
                    || error.raw_os_error() == Some(1314) =>
            {
                eprintln!(
                    "UNAVAILABLE: native changing-link test lacks OS symlink privilege: {error}"
                );
                return;
            }
            Err(error) => panic!("cannot prepare changing-link fixture: {error}"),
        }
        assert_ne!(
            reader
                .execute(ToolName::ReadFile, "changing", 256, None)
                .status,
            ToolStatus::Ok
        );
        std::fs::remove_file(&changing).unwrap();
        std::fs::write(&changing, "INSIDE").unwrap();
        assert_eq!(
            reader
                .execute(ToolName::ReadFile, "changing", 256, None)
                .body,
            "INSIDE"
        );

        // The capability is already open. The mutator changes only this fixture
        // entry; a failed unlink never falls through to writing through a link.
        let start = Barrier::new(2);
        std::thread::scope(|scope| {
            let mutator = scope.spawn(|| {
                start.wait();
                let mut replacements = 0;
                for _ in 0..256 {
                    match std::fs::remove_file(&changing) {
                        Ok(()) => {}
                        Err(error) if error.kind() == ErrorKind::NotFound => {}
                        Err(error)
                            if error.kind() == ErrorKind::PermissionDenied
                                || error.raw_os_error() == Some(32) =>
                        {
                            continue;
                        }
                        Err(error) => panic!("cannot remove fixture entry: {error}"),
                    }
                    make_link(&outside, &changing).unwrap();
                    replacements += 1;
                    std::thread::yield_now();
                    match std::fs::remove_file(&changing) {
                        Ok(()) => {}
                        Err(error)
                            if error.kind() == ErrorKind::PermissionDenied
                                || error.raw_os_error() == Some(32) =>
                        {
                            continue;
                        }
                        Err(error) => panic!("cannot remove fixture link: {error}"),
                    }
                    use std::io::Write;
                    std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&changing)
                        .unwrap()
                        .write_all(b"INSIDE")
                        .unwrap();
                }
                replacements
            });
            start.wait();
            for _ in 0..512 {
                let result = reader.execute(ToolName::ReadFile, "changing", 256, None);
                assert!(result.encoded().unwrap().len() <= 256);
                assert!(!result.body.contains("OUTSIDE-SENTINEL"));
                if result.status == ToolStatus::Ok {
                    // Opening while the regular file is being written can see
                    // its empty prefix. A snapshot of the whole file is not promised.
                    assert!("INSIDE".starts_with(&result.body));
                } else {
                    assert!(result.body.is_empty());
                    assert!(matches!(
                        result.error.unwrap().code.as_str(),
                        "not_found" | "access_denied" | "io_error"
                    ));
                }
            }
            assert!(mutator.join().unwrap() > 0);
        });
        assert_eq!(
            std::fs::read_to_string(outside).unwrap(),
            "OUTSIDE-SENTINEL"
        );
    }

    #[test]
    fn listing_stays_bounded_while_long_named_entries_disappear() {
        use std::sync::Barrier;

        let fixture = Fixture::new();
        let directory = fixture.root.join("workspace/nested");
        for index in 0..64 {
            fixture.write(
                &format!("nested/stable-{index:03}-{}.txt", "s".repeat(80)),
                "",
            );
        }
        let reader = fixture.reader();
        let start = Barrier::new(2);
        std::thread::scope(|scope| {
            let mutator = scope.spawn(|| {
                start.wait();
                for index in 0..256 {
                    let path =
                        directory.join(format!("volatile-{index:03}-{}.txt", "v".repeat(80)));
                    std::fs::write(&path, "").unwrap();
                    std::thread::yield_now();
                    std::fs::remove_file(path).unwrap();
                }
            });
            start.wait();
            for _ in 0..256 {
                let result = reader.execute(ToolName::ListFiles, "nested", 8_192, None);
                assert!(result.encoded().unwrap().len() <= 8_192);
                assert!(result.evidence_id.is_none());
                if result.status == ToolStatus::Ok {
                    let entries: Vec<ListedEntry> = serde_json::from_str(&result.body).unwrap();
                    assert!(entries.len() <= MAX_VISITED_ENTRIES);
                    assert!(entries.windows(2).all(|pair| pair[0].name <= pair[1].name));
                    assert!(
                        entries
                            .iter()
                            .all(|entry| entry.name.starts_with("nested/"))
                    );
                    // Neither membership nor completeness is stable under churn.
                } else {
                    assert!(result.body.is_empty());
                    assert!(matches!(
                        result.error.unwrap().code.as_str(),
                        "not_found" | "access_denied" | "io_error"
                    ));
                }
            }
            mutator.join().unwrap();
        });
    }
}
