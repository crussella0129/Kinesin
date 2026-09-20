//! Real loopback previews and grant boundaries, added after the live storefront pass.

use std::path::{Path, PathBuf};
use std::time::Duration;

use kinesin::config::Config;
use kinesin::core::{ModelReply, ToolCall};
use kinesin::model::ModelClient;
use kinesin::policy::Submission;
use kinesin::preview::WorkspacePreview;
use kinesin::runner::{RunResources, admit, run_admitted};
use kinesin::storage::{Command, QueueLimits, Response, Storage};
use kinesin::tools::{ToolResult, ToolStatus};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant, timeout};
use tokio_util::sync::CancellationToken;
use url::Url;

const WATCHDOG: Duration = Duration::from_secs(10);
const CONFIG: &str = r#"
version = 1
instructions = "Use the granted workspace tools."
[storage]
path = "state/kinesin.sqlite"
capture = "replay"
[[workspaces]]
id = "practice"
root = "workspace"
tools = ["read_file", "start_preview"]
[[models]]
id = "local"
base_url = "http://127.0.0.1:8080"
model_id = "preview-fixture"
context_size = 4096
verified_slots = 1
temperature = 0.0
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("kinesin-preview-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("workspace/public")).unwrap();
        std::fs::create_dir(root.join("workspace/other")).unwrap();
        let fixture = Self(root);
        fixture.write("public/index.html", b"<h1>Store version one</h1>");
        fixture.write("other/index.html", b"<h1>Another storefront</h1>");
        fixture
    }

    fn workspace(&self) -> PathBuf {
        self.0.join("workspace")
    }

    fn write(&self, path: &str, bytes: &[u8]) {
        std::fs::write(self.workspace().join(path), bytes).unwrap();
    }

    fn parse(&self, source: &str) -> Result<Config, String> {
        Config::parse(source, &self.0.join("kinesin.toml"))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(self.0.is_absolute() && self.0.starts_with(std::env::temp_dir()));
        assert!(
            self.0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("kinesin-preview-")
        );
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn start(preview: &WorkspacePreview, path: &str) -> ToolResult {
    preview
        .start(
            path,
            &CancellationToken::new(),
            Instant::now() + WATCHDOG,
            256,
        )
        .await
}

fn preview_url(result: &ToolResult) -> Url {
    assert_eq!(result.status, ToolStatus::Ok, "{:?}", result.error);
    assert!(result.encoded().unwrap().len() <= 256);
    let body: Value = serde_json::from_str(&result.body).unwrap();
    Url::parse(body["url"].as_str().unwrap()).unwrap()
}

struct Reply {
    status: u16,
    headers: String,
    body: Vec<u8>,
}

/// Send the request target verbatim: URL clients otherwise normalize traversal
/// before the server receives it, which would miss the actual boundary.
async fn request(url: &Url, method: &str, path: &str, headers: &[(&str, &str)]) -> Reply {
    timeout(WATCHDOG, async {
        let authority = format!("127.0.0.1:{}", url.port().unwrap());
        let host = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("host"))
            .map_or(authority.as_str(), |(_, value)| *value);
        let mut wire = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
        for (name, value) in headers {
            if !name.eq_ignore_ascii_case("host") {
                wire.push_str(&format!("{name}: {value}\r\n"));
            }
        }
        wire.push_str("\r\n");
        let mut socket = TcpStream::connect(&authority).await.unwrap();
        socket.write_all(wire.as_bytes()).await.unwrap();
        let mut bytes = Vec::new();
        socket
            .take(2 * 1024 * 1024)
            .read_to_end(&mut bytes)
            .await
            .unwrap();
        let split = bytes
            .windows(4)
            .position(|part| part == b"\r\n\r\n")
            .unwrap();
        let headers = String::from_utf8(bytes[..split].to_vec()).unwrap();
        let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
        Reply {
            status,
            headers: headers.to_ascii_lowercase(),
            body: bytes[split + 4..].to_vec(),
        }
    })
    .await
    .expect("preview HTTP request did not settle")
}

async fn assert_closed(url: &Url) {
    timeout(WATCHDOG, async {
        while TcpStream::connect(("127.0.0.1", url.port().unwrap()))
            .await
            .is_ok()
        {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("preview listener remained open");
}

#[tokio::test]
async fn real_http_reloads_assets_and_keeps_one_preview_until_shutdown() {
    let fixture = Fixture::new();
    fixture.write("public/styles.css", b"body { color: navy; }");
    fixture.write("public/store data.json", b"{\"stock\":2}");
    let preview = WorkspacePreview::new(&fixture.workspace()).unwrap();
    let run_cancel = CancellationToken::new();
    let result = preview
        .start("public", &run_cancel, Instant::now() + WATCHDOG, 256)
        .await;
    let url = preview_url(&result);
    assert_eq!(url.host_str(), Some("127.0.0.1"));
    let first = request(&url, "GET", url.path(), &[]).await;
    assert_eq!(first.status, 200);
    assert_eq!(first.body, b"<h1>Store version one</h1>");
    for required in [
        "content-type: text/html; charset=utf-8",
        "cache-control: no-store",
        "x-content-type-options: nosniff",
        "referrer-policy: no-referrer",
        "script-src 'self'",
        "style-src 'self'",
        "frame-ancestors 'none'",
    ] {
        assert!(
            first.headers.contains(required),
            "missing header {required}"
        );
    }
    assert!(!first.headers.contains("unsafe-inline"));
    let css_path = format!("{}styles.css", url.path());
    let head = request(&url, "HEAD", &css_path, &[]).await;
    assert_eq!(head.status, 200);
    assert!(head.body.is_empty());
    assert!(head.headers.contains("content-length: 21"));
    let data = request(
        &url,
        "GET",
        &format!("{}store%20data.json", url.path()),
        &[],
    )
    .await;
    assert_eq!(data.status, 200);
    assert_eq!(data.body, b"{\"stock\":2}");

    fixture.write("public/index.html", b"<h1>Store version two</h1>");
    run_cancel.cancel(); // A completed turn's token does not own the session server.
    let edited = request(&url, "GET", url.path(), &[]).await;
    assert_eq!(edited.body, b"<h1>Store version two</h1>");
    assert_eq!(preview_url(&start(&preview, "public").await), url);
    let other = start(&preview, "other").await;
    assert_eq!(other.error.unwrap().code, "preview_already_running");
    preview.shutdown().await.unwrap();
    preview.shutdown().await.unwrap();
    assert_closed(&url).await;
}

#[tokio::test]
async fn real_http_enforces_selected_subtree_origin_methods_and_asset_limits() {
    let fixture = Fixture::new();
    fixture.write("secret.html", b"OUTSIDE_SELECTED_SUBTREE");
    fixture.write("public/.hidden.html", b"HIDDEN");
    fixture.write("public/source.rs", b"SOURCE");
    std::fs::File::create(fixture.workspace().join("public/huge.png"))
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();
    std::fs::create_dir(fixture.workspace().join("public/subdirectory")).unwrap();
    let preview = WorkspacePreview::new(&fixture.workspace()).unwrap();
    let url = preview_url(&start(&preview, "public").await);
    for suffix in [
        "../secret.html",
        "%2e%2e/secret.html",
        "%2e%2e%2fsecret.html",
        ".hidden.html",
        "source.rs",
        "huge.png",
        "subdirectory/",
        "bad%Q0.html",
    ] {
        let reply = request(&url, "GET", &format!("{}{suffix}", url.path()), &[]).await;
        assert_eq!(reply.status, 404, "unexpected access: {suffix}");
        assert!(!String::from_utf8_lossy(&reply.body).contains("OUTSIDE_SELECTED_SUBTREE"));
    }
    assert_eq!(request(&url, "GET", "/index.html", &[]).await.status, 404);
    assert_eq!(
        request(&url, "GET", url.path(), &[("Host", "evil.example")])
            .await
            .status,
        403
    );
    assert_eq!(
        request(
            &url,
            "GET",
            url.path(),
            &[("Origin", "https://evil.example")]
        )
        .await
        .status,
        403
    );
    let origin = url.origin().ascii_serialization();
    assert_eq!(
        request(&url, "GET", url.path(), &[("Origin", &origin)])
            .await
            .status,
        200
    );
    let post = request(&url, "POST", url.path(), &[]).await;
    assert_eq!(post.status, 405);
    assert!(post.headers.contains("allow: get, head"));
    preview.shutdown().await.unwrap();
}

#[tokio::test]
async fn cancellation_deadline_and_drop_do_not_leave_a_listener() {
    let fixture = Fixture::new();
    let preview = WorkspacePreview::new(&fixture.workspace()).unwrap();
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    let result = preview
        .start("public", &cancelled, Instant::now() + WATCHDOG, 256)
        .await;
    assert_eq!(result.error.unwrap().code, "cancelled");
    let result = preview
        .start("public", &CancellationToken::new(), Instant::now(), 256)
        .await;
    assert_eq!(result.error.unwrap().code, "timed_out");
    assert_eq!(
        start(&preview, "../outside").await.error.unwrap().code,
        "invalid_path"
    );
    let url = preview_url(&start(&preview, "public").await);
    assert_eq!(request(&url, "GET", url.path(), &[]).await.status, 200);
    drop(preview);
    assert_closed(&url).await;
}

fn symlink_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(source, destination)
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, destination)
    }
}

#[tokio::test]
async fn native_symlink_cannot_expose_an_outside_asset() {
    let fixture = Fixture::new();
    fixture.write("secret.html", b"OUTSIDE_LINK_TARGET");
    let created = symlink_file(
        &fixture.workspace().join("secret.html"),
        &fixture.workspace().join("public/leak.html"),
    );
    match created {
        Ok(()) => {}
        Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(1314) =>
        {
            eprintln!("UNAVAILABLE: preview symlink test lacks OS symlink privilege: {error}");
            return;
        }
        Err(error) => panic!("cannot prepare preview symlink fixture: {error}"),
    }
    let preview = WorkspacePreview::new(&fixture.workspace()).unwrap();
    let url = preview_url(&start(&preview, "public").await);
    let reply = request(&url, "GET", &format!("{}leak.html", url.path()), &[]).await;
    assert_eq!(reply.status, 404);
    assert!(!String::from_utf8_lossy(&reply.body).contains("OUTSIDE_LINK_TARGET"));
    preview.shutdown().await.unwrap();
}

#[test]
fn preview_grants_reject_service_configuration_and_checked_authority() {
    let fixture = Fixture::new();
    let service = format!(
        "{CONFIG}\n[service]\nlisten = \"127.0.0.1:8081\"\ncredential_verifiers = \"private/verifiers.json\"\nmax_submission_bytes = 65536\nmax_page_size = 20\nidempotency_retention_hours = 24\n"
    );
    assert_eq!(
        fixture.parse(&service).unwrap_err(),
        "start_preview is available only in local CLI sessions"
    );
    let checked = format!(
        "{CONFIG}\n[[tasks]]\nid = \"facts\"\nversion = 1\nchecker = \"file_fields_v1\"\nchecker_version = 1\nworkspace = \"practice\"\n[[tasks.criteria]]\nid = \"language\"\npath = \"project.txt\"\nkey = \"language\"\n"
    );
    let config = fixture.parse(&checked).unwrap();
    assert_eq!(
        config
            .authorize_local(Submission::Checked {
                task: "facts".into(),
                model: "local".into(),
                limits: None,
                capture: None,
            })
            .unwrap_err(),
        "a checked task workspace cannot enable a write tool"
    );
}

#[tokio::test]
async fn actual_runner_denies_an_ungranted_preview_call() {
    let fixture = Fixture::new();
    let config = fixture
        .parse(&CONFIG.replace("\"read_file\", \"start_preview\"", "\"read_file\""))
        .unwrap();
    let authority = config
        .authorize_local(Submission::Freeform {
            workspace: "practice".into(),
            model: "local".into(),
            prompt: "Preview this storefront.".into(),
            continues: None,
            limits: None,
            capture: None,
        })
        .unwrap();
    let database = config.storage().path.clone();
    let storage =
        tokio::task::spawn_blocking(move || Storage::start(database, QueueLimits::default()))
            .await
            .unwrap()
            .unwrap();
    let resources = RunResources::from_config(&config)
        .unwrap()
        .remove("local")
        .unwrap();
    let client = ModelClient::scripted([
        ModelReply::ToolCalls {
            content: None,
            calls: vec![ToolCall {
                id: "ungranted-preview".into(),
                name: "start_preview".into(),
                arguments: json!({"path":"public"}).to_string(),
            }],
        }
        .into(),
        ModelReply::Answer("Preview is not granted.".into()).into(),
    ]);
    admit(&authority, &storage.client(), None).await.unwrap();
    let record = run_admitted(
        authority.clone(),
        client,
        storage.client(),
        resources.clone(),
        CancellationToken::new(),
        Instant::now(),
    )
    .await
    .unwrap();
    assert_eq!(record.phase, "completed");
    let response = storage
        .client()
        .execute(
            Command::Events {
                owner_id: authority.owner().into(),
                run_id: authority.run_id().into(),
                after: None,
                limit: 100,
            },
            Instant::now() + WATCHDOG,
        )
        .await
        .unwrap();
    let Response::Events(events) = response else {
        panic!("expected recorded tool events");
    };
    let result = &events
        .iter()
        .find(|event| event.kind == "tool_finished")
        .unwrap()
        .data;
    assert_eq!(result["dispatch"], "denied");
    assert_eq!(
        result["replay"]["observation"]["error"]["code"],
        "tool_denied"
    );
    resources.shutdown_previews().await.unwrap();
    storage.shutdown().await.unwrap();
}
