//! Bounded local model discovery and the human terminal picker.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, QueueableCommand};

pub const MAX_MODEL_DIRECTORIES: usize = 16;
pub const MAX_SCANNED_ENTRIES: usize = 4_096;
pub const MAX_MODELS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFile {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelChoice {
    File(ModelFile),
    Other,
}

/// Stable user storage comes first. The launch directory and installed binary
/// directory retain compatibility with both `model/` and `models/` layouts.
/// No drive scan, executable execution, network lookup, or directory creation.
pub fn default_model_directories(cwd: &Path, executable: &Path) -> Result<Vec<PathBuf>, String> {
    #[cfg(windows)]
    let data_root = absolute_environment("LOCALAPPDATA")?.join("Kinesin");
    #[cfg(unix)]
    let data_root = match std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        Some(path) if path.is_absolute() => path,
        _ => absolute_environment("HOME")?.join(".local/share"),
    }
    .join("kinesin");
    model_directories(&data_root, cwd, executable)
}

fn absolute_environment(name: &str) -> Result<PathBuf, String> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| format!("{name} must name an absolute directory; use --model-path to select a file explicitly"))
}

pub(crate) fn model_directories(
    data_root: &Path,
    cwd: &Path,
    executable: &Path,
) -> Result<Vec<PathBuf>, String> {
    let executable_dir = executable
        .parent()
        .ok_or("cannot locate executable directory")?;
    if !data_root.is_absolute() || !cwd.is_absolute() || !executable_dir.is_absolute() {
        return Err("model discovery roots must be absolute".into());
    }
    let mut roots = Vec::new();
    for path in [
        data_root.join("models"),
        cwd.join("model"),
        cwd.join("models"),
        executable_dir.join("model"),
        executable_dir.join("models"),
    ] {
        if !roots.contains(&path) {
            roots.push(path);
        }
    }
    Ok(roots)
}

/// Inspect direct children only. Symlink entries are skipped during discovery;
/// an operator may explicitly select a symlink with `--model-path`, which is
/// then resolved to its canonical regular target before runtime admission.
pub fn discover_ggufs(roots: &[PathBuf]) -> Result<Vec<ModelFile>, String> {
    if roots.len() > MAX_MODEL_DIRECTORIES {
        return Err("model discovery exceeds 16 folders; use --model-path".into());
    }
    let mut directories = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut models = Vec::new();
    let mut scanned = 0;
    for root in roots {
        if !root.is_absolute() {
            return Err("model discovery roots must be absolute".into());
        }
        let root = match root.canonicalize() {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => {
                return Err(format!(
                    "cannot access model folder {}",
                    safe_label(&root.display().to_string())
                ));
            }
        };
        if !directories.insert(root.clone()) {
            continue;
        }
        let entries = std::fs::read_dir(&root).map_err(|_| {
            format!(
                "cannot read model folder {}",
                safe_label(&root.display().to_string())
            )
        })?;
        for entry in entries {
            if scanned >= MAX_SCANNED_ENTRIES {
                return Err(
                    "model discovery exceeds 4096 directory entries; use --model-path".into(),
                );
            }
            scanned += 1;
            let entry = entry.map_err(|_| "cannot inspect model folder entry")?;
            if !entry
                .file_type()
                .map_err(|_| "cannot inspect model file type")?
                .is_file()
                || !is_gguf(&entry.path())
            {
                continue;
            }
            let model = match inspect_model(&entry.path()) {
                Ok(model) => model,
                // A non-GGUF file merely named *.gguf is not offered as a model.
                Err(_) => continue,
            };
            if files.insert(model.path.clone()) {
                if models.len() >= MAX_MODELS {
                    return Err("model discovery exceeds 64 GGUF files; use --model-path".into());
                }
                models.push(model);
            }
        }
    }
    models.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(models)
}

/// An explicit operator path can be outside the discovery folders. File
/// validation is bounded to metadata and the four-byte format signature.
pub fn resolve_model_path(path: &Path, cwd: &Path) -> Result<ModelFile, String> {
    if path.as_os_str().is_empty() || path.as_os_str().len() > crate::config::MAX_PATH_BYTES {
        return Err("model path must be nonempty and at most 4096 bytes".into());
    }
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        cwd.join(path)
    };
    inspect_model(&path)
}

fn is_gguf(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"))
}

fn inspect_model(path: &Path) -> Result<ModelFile, String> {
    let path = path
        .canonicalize()
        .map_err(|_| "model path must name an existing GGUF file")?;
    if !is_gguf(&path) {
        return Err("model file must have a .gguf extension".into());
    }
    // Inspect before opening so a FIFO or device is never treated as a model.
    if !std::fs::metadata(&path)
        .map_err(|_| "cannot inspect model file")?
        .is_file()
    {
        return Err("model path must name a regular GGUF file".into());
    }
    let mut options = File::options();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    let mut file = options.open(&path).map_err(|_| "cannot read model file")?;
    let metadata = file
        .metadata()
        .map_err(|_| "cannot inspect open model file")?;
    if !metadata.is_file() || metadata.len() < 4 {
        return Err("model path must name a regular GGUF file".into());
    }
    let mut magic = [0; 4];
    file.read_exact(&mut magic)
        .map_err(|_| "cannot read model file signature")?;
    if &magic != b"GGUF" {
        return Err("selected file does not have a GGUF signature".into());
    }
    Ok(ModelFile {
        path,
        bytes: metadata.len(),
    })
}

/// Pick a default with Enter, move the caret with arrows, or choose another
/// path. Machine/redirected terminals get the same options as a numbered list.
pub fn choose_model(models: &[ModelFile], preferred: Option<&Path>) -> Result<ModelChoice, String> {
    let selected = default_index(models, preferred)?;
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return numbered_choice(
            models,
            selected,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
        );
    }
    let _terminal = PickerTerminal::enter().map_err(|_| "cannot prepare terminal model picker")?;
    let mut selected = selected;
    let mut output = io::stdout().lock();
    loop {
        render_picker(&mut output, models, selected).map_err(|_| "cannot display model picker")?;
        match event::read().map_err(|_| "cannot read model selection")? {
            Event::Key(key) => match key_action(key, selected, models.len() + 1) {
                SelectionAction::Move(next) => selected = next,
                SelectionAction::Accept => return Ok(choice_at(models, selected)),
                SelectionAction::Cancel => return Err("model selection cancelled".into()),
                SelectionAction::Ignore => {}
            },
            Event::Resize(_, _) => {}
            _ => continue,
        }
    }
}

fn default_index(models: &[ModelFile], preferred: Option<&Path>) -> Result<usize, String> {
    if models.len() > MAX_MODELS {
        return Err("model picker supports at most 64 GGUF files".into());
    }
    Ok(preferred
        .and_then(|path| models.iter().position(|model| model.path == path))
        .unwrap_or(0))
}

fn choice_at(models: &[ModelFile], selected: usize) -> ModelChoice {
    models
        .get(selected)
        .cloned()
        .map(ModelChoice::File)
        .unwrap_or(ModelChoice::Other)
}

#[derive(Debug, PartialEq, Eq)]
enum SelectionAction {
    Move(usize),
    Accept,
    Cancel,
    Ignore,
}

fn key_action(key: KeyEvent, selected: usize, count: usize) -> SelectionAction {
    if key.kind == KeyEventKind::Release {
        return SelectionAction::Ignore;
    }
    if key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C' | 'd' | 'D')))
    {
        return SelectionAction::Cancel;
    }
    match key.code {
        KeyCode::Up | KeyCode::BackTab => SelectionAction::Move((selected + count - 1) % count),
        KeyCode::Down | KeyCode::Tab => SelectionAction::Move((selected + 1) % count),
        KeyCode::Home => SelectionAction::Move(0),
        KeyCode::End => SelectionAction::Move(count - 1),
        KeyCode::Enter => SelectionAction::Accept,
        _ => SelectionAction::Ignore,
    }
}

struct PickerTerminal {
    restore_raw: bool,
    alternate: bool,
}

impl PickerTerminal {
    fn enter() -> io::Result<Self> {
        let restore_raw = !terminal::is_raw_mode_enabled()?;
        if restore_raw {
            terminal::enable_raw_mode()?;
        }
        let mut guard = Self {
            restore_raw,
            alternate: false,
        };
        io::stdout().execute(EnterAlternateScreen)?;
        guard.alternate = true;
        Ok(guard)
    }
}

impl Drop for PickerTerminal {
    fn drop(&mut self) {
        // Restore on Enter, Esc, Ctrl+C, I/O failure, and panic unwinding. Raw
        // mode delivers Ctrl+C as a key instead of terminating mid-restoration.
        if self.alternate {
            let _ = io::stdout().execute(LeaveAlternateScreen);
        }
        if self.restore_raw {
            let _ = terminal::disable_raw_mode();
        }
    }
}

fn render_picker(output: &mut impl Write, models: &[ModelFile], selected: usize) -> io::Result<()> {
    let (width, height) = terminal::size().unwrap_or((80, 24));
    let budget = usize::from(width.saturating_sub(1)).max(1);
    let count = models.len() + 1;
    let visible = usize::from(height.saturating_sub(9))
        .clamp(1, 12)
        .min(count);
    let start = selected.saturating_sub(visible - 1);
    output.queue(MoveTo(0, 0))?.queue(Clear(ClearType::All))?;
    writeln!(output, "{}\r", clipped("Choose a local model", budget))?;
    writeln!(
        output,
        "{}\r\n",
        clipped("Up/Down: move   Enter: select   Esc/Ctrl+C: cancel", budget)
    )?;
    for index in start..start + visible {
        let caret = if index == selected { '>' } else { ' ' };
        let label = option_label(models, index);
        writeln!(
            output,
            "{}\r",
            clipped(&format!("{caret} {}. {label}", index + 1), budget)
        )?;
    }
    writeln!(
        output,
        "\r\n{}\r",
        clipped(&format!("Option {} of {count}", selected + 1), budget)
    )?;
    if let Some(model) = models.get(selected) {
        writeln!(output, "{}\r", clipped("Selected path:", budget))?;
        let path = safe_label(&model.path.display().to_string());
        let mut remaining = path.as_str();
        for row in 0..4 {
            let part = clipped(remaining, budget);
            if part.is_empty() {
                break;
            }
            if row == 3 && part.len() < remaining.len() {
                writeln!(
                    output,
                    "{}...\r",
                    clipped(remaining, budget.saturating_sub(3))
                )?;
            } else {
                writeln!(output, "{part}\r")?;
            }
            remaining = &remaining[part.len()..];
            if remaining.is_empty() {
                break;
            }
        }
    }
    output.flush()
}

fn option_label(models: &[ModelFile], index: usize) -> String {
    models
        .get(index)
        .map(|model| {
            format!(
                "{} ({:.2} GiB)",
                safe_label(
                    &model
                        .path
                        .file_name()
                        .unwrap_or_else(|| model.path.as_os_str())
                        .to_string_lossy()
                ),
                model.bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        })
        .unwrap_or_else(|| "Choose another GGUF file...".into())
}

fn safe_label(text: &str) -> String {
    let mut safe = String::new();
    for character in text.chars() {
        if character.is_control()
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

// UTF-8 bytes are an upper bound on the terminal columns occupied by these
// escaped labels. Conservative clipping avoids wrapped menu rows, including
// wide glyphs, without cutting a UTF-8 codepoint.
fn clipped(text: &str, budget: usize) -> &str {
    let mut end = text.len().min(budget);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn numbered_choice(
    models: &[ModelFile],
    selected: usize,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<ModelChoice, String> {
    writeln!(output, "Choose a local model:").map_err(|_| "cannot write model options")?;
    for index in 0..=models.len() {
        writeln!(output, "  {}. {}", index + 1, option_label(models, index))
            .map_err(|_| "cannot write model option")?;
    }
    loop {
        write!(
            output,
            "Model [{}] (number, Enter for default, or q to cancel): ",
            selected + 1
        )
        .and_then(|()| output.flush())
        .map_err(|_| "cannot write model prompt")?;
        let mut answer = Vec::new();
        input
            .take(66)
            .read_until(b'\n', &mut answer)
            .map_err(|_| "cannot read model selection")?;
        if answer.is_empty() {
            return Err("model selection cancelled".into());
        }
        if answer.len() > 65 {
            return Err("model selection answer exceeds 64 bytes".into());
        }
        let answer = std::str::from_utf8(&answer)
            .map_err(|_| "model selection must be UTF-8")?
            .trim();
        if answer.eq_ignore_ascii_case("q") || matches!(answer, "\u{3}" | "\u{1b}") {
            return Err("model selection cancelled".into());
        }
        let index = if answer.is_empty() {
            Some(selected)
        } else {
            answer
                .parse::<usize>()
                .ok()
                .and_then(|value| value.checked_sub(1))
                .filter(|index| *index <= models.len())
        };
        if let Some(index) = index {
            return Ok(choice_at(models, index));
        }
        writeln!(output, "Choose one of the listed numbers.")
            .map_err(|_| "cannot write model selection error")?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("kinesin-model-picker-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&root).unwrap();
            Self(root.canonicalize().unwrap())
        }
        fn model(&self, name: &str) -> ModelFile {
            let path = self.0.join(name);
            std::fs::write(&path, b"GGUFsynthetic model fixture").unwrap();
            inspect_model(&path).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn model_layout_is_stable_and_supports_both_compatibility_folders() {
        let fixture = Fixture::new();
        let roots = model_directories(
            &fixture.0.join("data"),
            &fixture.0.join("project"),
            &fixture.0.join("bin/kinesin"),
        )
        .unwrap();
        assert_eq!(
            roots,
            vec![
                fixture.0.join("data/models"),
                fixture.0.join("project/model"),
                fixture.0.join("project/models"),
                fixture.0.join("bin/model"),
                fixture.0.join("bin/models")
            ]
        );
        assert!(!fixture.0.join("data").exists());
        assert!(
            model_directories(
                Path::new("relative"),
                &fixture.0,
                &fixture.0.join("kinesin")
            )
            .is_err()
        );
    }

    #[test]
    fn discovery_is_deterministic_bounded_nonrecursive_and_deduplicated() {
        let fixture = Fixture::new();
        let z = fixture.model("z model.GGUF");
        let a = fixture.model("a.gguf");
        std::fs::write(fixture.0.join("ignore.txt"), b"GGUFsynthetic").unwrap();
        std::fs::write(fixture.0.join("invalid.gguf"), b"not a model").unwrap();
        std::fs::create_dir(fixture.0.join("nested")).unwrap();
        std::fs::write(fixture.0.join("nested/hidden.gguf"), b"GGUF").unwrap();
        let found = discover_ggufs(&[
            fixture.0.clone(),
            fixture.0.clone(),
            fixture.0.join("absent"),
        ])
        .unwrap();
        assert_eq!(found, vec![a, z]);
        assert!(
            discover_ggufs(&vec![fixture.0.clone(); MAX_MODEL_DIRECTORIES + 1])
                .unwrap_err()
                .contains("16 folders")
        );
    }

    #[test]
    fn discovery_refuses_an_oversized_model_catalog() {
        let fixture = Fixture::new();
        for index in 0..=MAX_MODELS {
            fixture.model(&format!("{index:03}.gguf"));
        }
        assert!(
            discover_ggufs(std::slice::from_ref(&fixture.0))
                .unwrap_err()
                .contains("64 GGUF")
        );
    }

    #[test]
    fn explicit_path_resolves_external_files_and_rejects_invalid_inputs() {
        let fixture = Fixture::new();
        let model = fixture.model("outside folder.gguf");
        assert_eq!(
            resolve_model_path(Path::new("outside folder.gguf"), &fixture.0).unwrap(),
            model
        );
        assert_eq!(
            resolve_model_path(&model.path, &fixture.0.join("elsewhere")).unwrap(),
            model
        );
        assert!(resolve_model_path(Path::new("missing.gguf"), &fixture.0).is_err());
        assert!(resolve_model_path(&fixture.0, &fixture.0).is_err());
        std::fs::write(fixture.0.join("false.gguf"), b"fake").unwrap();
        assert!(
            resolve_model_path(Path::new("false.gguf"), &fixture.0)
                .unwrap_err()
                .contains("signature")
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_skipped_by_discovery_but_explicit_operator_paths_resolve() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        let model = fixture.model("real.gguf");
        symlink(&model.path, fixture.0.join("linked.gguf")).unwrap();
        assert_eq!(
            discover_ggufs(std::slice::from_ref(&fixture.0)).unwrap(),
            vec![model.clone()]
        );
        assert_eq!(
            resolve_model_path(Path::new("linked.gguf"), &fixture.0).unwrap(),
            model
        );
    }

    #[test]
    fn arrows_enter_escape_and_control_c_have_deterministic_selection_semantics() {
        let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
        assert_eq!(
            key_action(key(KeyCode::Down), 0, 3),
            SelectionAction::Move(1)
        );
        assert_eq!(key_action(key(KeyCode::Up), 0, 3), SelectionAction::Move(2));
        assert_eq!(
            key_action(key(KeyCode::Enter), 1, 3),
            SelectionAction::Accept
        );
        assert_eq!(key_action(key(KeyCode::Esc), 1, 3), SelectionAction::Cancel);
        assert_eq!(
            key_action(
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                1,
                3
            ),
            SelectionAction::Cancel
        );
        let mut release = key(KeyCode::Down);
        release.kind = KeyEventKind::Release;
        assert_eq!(key_action(release, 0, 3), SelectionAction::Ignore);
    }

    #[test]
    fn numbered_fallback_preserves_default_offers_other_and_handles_cancellation() {
        let fixture = Fixture::new();
        let models = [fixture.model("one.gguf"), fixture.model("two.gguf")];
        assert_eq!(default_index(&models, Some(&models[1].path)).unwrap(), 1);
        assert_eq!(
            numbered_choice(&models, 1, &mut &b"\n"[..], &mut Vec::new()).unwrap(),
            ModelChoice::File(models[1].clone())
        );
        let mut output = Vec::new();
        assert_eq!(
            numbered_choice(&models, 0, &mut &b"99\n3\n"[..], &mut output).unwrap(),
            ModelChoice::Other
        );
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("Choose another GGUF file...")
        );
        assert_eq!(
            numbered_choice(&[], 0, &mut &b"\n"[..], &mut Vec::new()).unwrap(),
            ModelChoice::Other
        );
        assert!(numbered_choice(&models, 0, &mut &b"q\n"[..], &mut Vec::new()).is_err());
        assert!(numbered_choice(&models, 0, &mut &b""[..], &mut Vec::new()).is_err());
    }

    #[test]
    fn displayed_paths_escape_controls_and_clip_without_splitting_utf8() {
        assert_eq!(
            safe_label("model\x1b[2J\n\u{202e}.gguf"),
            "model\\u{1b}[2J\\u{a}\\u{202e}.gguf"
        );
        assert_eq!(clipped("模型.gguf", 5), "模");
        assert_eq!(clipped("test", 3), "tes");
        let model = ModelFile {
            path: PathBuf::from("very/long/private/installation/path/selected.gguf"),
            bytes: 1024,
        };
        assert!(option_label(&[model], 0).starts_with("selected.gguf"));
    }
}
