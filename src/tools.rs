//! Two bounded read operations behind a private filesystem capability.

use std::io::{ErrorKind, Read};
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use serde::{Deserialize, Serialize};

use crate::config::{ToolName, validate_id, validate_relative_path};

pub const MAX_TOOL_BYTES: usize = 8_192;
pub const MAX_VISITED_ENTRIES: usize = 256;
pub const MAX_ARGUMENT_BYTES: usize = 65_536;

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
}

impl TypedToolArgs {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_ARGUMENT_BYTES {
            return Err("tool arguments exceed 64 KiB".into());
        }
        let mut args: Self =
            serde_json::from_str(raw).map_err(|_| "invalid tool arguments or fields")?;
        args.path = normalized_path(&args.path)?;
        Ok(args)
    }
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
        match name {
            ToolName::ReadFile => self.read_file(path, limit, evidence_id),
            ToolName::ListFiles => self.list_files(path, limit),
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
