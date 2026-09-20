//! A session-owned loopback preview of explicitly granted public workspace assets.

use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::ingress::{IngressLimits, IngressStats};
use crate::tools::{ToolResult, ToolStatus, normalized_path};

const MAX_ASSET_BYTES: usize = 1024 * 1024;
const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

pub struct WorkspacePreview {
    root: Arc<Dir>,
    active: Mutex<Option<ActivePreview>>,
}

struct ActivePreview {
    path: String,
    url: String,
    shutdown: CancellationToken,
    server: JoinHandle<Result<(), String>>,
}

impl Drop for ActivePreview {
    fn drop(&mut self) {
        self.shutdown.cancel();
        self.server.abort();
    }
}

impl WorkspacePreview {
    /// Trusted startup only; the caller supplies an operator-approved root.
    pub fn new(root: &Path) -> Result<Self, String> {
        Ok(Self {
            root: Arc::new(
                Dir::open_ambient_dir(root, ambient_authority())
                    .map_err(|_| "cannot open approved preview workspace")?,
            ),
            active: Mutex::new(None),
        })
    }

    /// Start at most one preview. Run cancellation prevents startup, while the
    /// successful server belongs to the session and survives a completed turn.
    pub async fn start(
        &self,
        path: &str,
        cancel: &CancellationToken,
        deadline: Instant,
        max_output: usize,
    ) -> ToolResult {
        if max_output < 256 {
            return failure(
                "invalid_limit",
                "Preview results require at least 256 bytes.",
            );
        }
        if normalized_path(path).is_err()
            || (path != "." && path.split('/').any(|part| part.starts_with('.')))
        {
            return failure("invalid_path", "Use a relative public workspace directory.");
        }
        let mut current = tokio::select! {
            biased;
            () = cancel.cancelled() => return failure("cancelled", "Preview startup cancelled."),
            () = tokio::time::sleep_until(deadline) => return startup_timeout(),
            current = self.active.lock() => current,
        };
        if let Some(active) = current.as_ref()
            && !active.server.is_finished()
        {
            return if active.path == path {
                running_result(&active.url)
            } else {
                failure(
                    "preview_already_running",
                    "This workspace already previews another directory; restart the session to change it.",
                )
            };
        }
        // A stopped server can be replaced, but never leave its owned task behind.
        if let Some(mut previous) = current.take() {
            let _ = (&mut previous.server).await;
        }
        if cancel.is_cancelled() {
            return failure("cancelled", "Preview startup cancelled.");
        }
        if Instant::now() >= deadline {
            return startup_timeout();
        }
        let workspace = self.root.clone();
        let selected_path = path.to_owned();
        let mut preparation = tokio::task::spawn_blocking(move || {
            let root = open_directory(&workspace, &selected_path).map_err(|_| {
                failure(
                    "preview_directory_unavailable",
                    "The preview directory must exist inside the workspace without symbolic links.",
                )
            })?;
            read_asset(&root, "index.html").map_err(|_| {
                failure(
                    "preview_index_unavailable",
                    "The preview directory needs a regular index.html no larger than 1 MiB.",
                )
            })?;
            Ok(root)
        });
        let prepared = tokio::select! {
            biased;
            () = cancel.cancelled() => Err(failure("cancelled", "Preview startup cancelled.")),
            () = tokio::time::sleep_until(deadline) => Err(startup_timeout()),
            result = &mut preparation => Ok(result),
        };
        let root = match prepared {
            Ok(Ok(Ok(root))) => root,
            Ok(Ok(Err(result))) => return result,
            Ok(Err(_)) => return failure("tool_worker_failed", "Preview preparation failed."),
            Err(stopped) => {
                // Filesystem work cannot be cancelled. The runner retains this
                // future and its tool permit through its settlement grace and,
                // if needed, until this owned worker actually finishes.
                let _ = preparation.await;
                return stopped;
            }
        };
        let listener = tokio::select! {
            biased;
            () = cancel.cancelled() => return failure("cancelled", "Preview startup cancelled."),
            () = tokio::time::sleep_until(deadline) => return startup_timeout(),
            listener = TcpListener::bind("127.0.0.1:0") => match listener {
                Ok(listener) => listener,
                Err(_) => return failure("preview_bind_failed", "Cannot open a local preview port."),
            },
        };
        let address = match listener.local_addr() {
            Ok(address) => address,
            Err(_) => {
                return failure(
                    "preview_bind_failed",
                    "Cannot resolve the local preview port.",
                );
            }
        };
        let prefix = format!("/{}/", uuid::Uuid::new_v4().simple());
        let host = address.to_string();
        let origin = format!("http://{host}");
        let url = format!("{origin}{prefix}");
        let state = Arc::new(PreviewState {
            root,
            host,
            origin,
            prefix,
        });
        let router = Router::new().fallback(asset).with_state(state);
        let shutdown = CancellationToken::new();
        if cancel.is_cancelled() {
            return failure("cancelled", "Preview startup cancelled.");
        }
        if Instant::now() >= deadline {
            return startup_timeout();
        }
        let server = tokio::spawn(crate::ingress::serve(
            listener,
            router,
            shutdown.clone(),
            IngressLimits {
                max_connections: 16,
                header_timeout: Duration::from_secs(5),
                connection_lifetime: Duration::from_secs(30),
                shutdown_grace: Duration::from_secs(1),
            },
            Arc::new(IngressStats::default()),
        ));
        *current = Some(ActivePreview {
            path: path.to_owned(),
            url: url.clone(),
            shutdown,
            server,
        });
        running_result(&url)
    }

    /// Stop accepting, drain bounded connections, and join the server task.
    pub async fn shutdown(&self) -> Result<(), String> {
        let Some(mut active) = self.active.lock().await.take() else {
            return Ok(());
        };
        active.shutdown.cancel();
        (&mut active.server)
            .await
            .map_err(|_| "preview server task failed".to_owned())?
    }
}

impl Drop for WorkspacePreview {
    fn drop(&mut self) {
        // ActivePreview owns both cancellation and abort if normal async cleanup
        // is interrupted or startup fails after resources were prepared.
        self.active.get_mut().take();
    }
}

struct PreviewState {
    root: Dir,
    host: String,
    origin: String,
    prefix: String,
}

async fn asset(State(state): State<Arc<PreviewState>>, request: Request) -> Response {
    let headers = request.headers();
    if headers.get_all(header::HOST).iter().count() != 1
        || headers
            .get(header::HOST)
            .and_then(|value| value.to_str().ok())
            != Some(state.host.as_str())
        || headers
            .get(header::ORIGIN)
            .is_some_and(|value| value.to_str().ok() != Some(state.origin.as_str()))
    {
        return secured(StatusCode::FORBIDDEN.into_response());
    }
    let head = request.method() == Method::HEAD;
    if request.method() != Method::GET && !head {
        let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("GET, HEAD"));
        return secured(response);
    }
    let Some(raw) = request.uri().path().strip_prefix(&state.prefix) else {
        return secured(StatusCode::NOT_FOUND.into_response());
    };
    let Some(path) = decode_path(raw) else {
        return secured(StatusCode::NOT_FOUND.into_response());
    };
    let path = if path.is_empty() {
        "index.html".to_owned()
    } else {
        path
    };
    let Some(content_type) = public_content_type(&path) else {
        return secured(StatusCode::NOT_FOUND.into_response());
    };
    let bytes = match tokio::task::spawn_blocking(move || read_asset(&state.root, &path)).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(status)) => return secured(status.into_response()),
        Err(_) => return secured(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    };
    let length = bytes.len();
    let mut response = Response::new(if head {
        Body::empty()
    } else {
        Body::from(bytes)
    });
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&length.to_string()).expect("bounded asset length"),
    );
    secured(response)
}

fn secured(mut response: Response) -> Response {
    for (name, value) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::CONTENT_SECURITY_POLICY, CSP),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "no-referrer"),
    ] {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    response
}

fn open_directory(root: &Dir, path: &str) -> std::io::Result<Dir> {
    let mut directory = root.try_clone()?;
    if path != "." {
        for component in path.split('/') {
            directory = directory.open_dir_nofollow(component)?;
        }
    }
    Ok(directory)
}

fn read_asset(root: &Dir, path: &str) -> Result<Vec<u8>, StatusCode> {
    if normalized_path(path).is_err()
        || path.split('/').any(|part| part.starts_with('.'))
        || public_content_type(path).is_none()
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let (parent, name) = path.rsplit_once('/').unwrap_or((".", path));
    let directory = open_directory(root, parent).map_err(|_| StatusCode::NOT_FOUND)?;
    let metadata = directory
        .symlink_metadata(name)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !metadata.is_file() || metadata.len() > MAX_ASSET_BYTES as u64 {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = directory
        .open_with(name, &options)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !file
        .metadata()
        .map_err(|_| StatusCode::NOT_FOUND)?
        .is_file()
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut bytes = Vec::new();
    file.take((MAX_ASSET_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if bytes.len() > MAX_ASSET_BYTES {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(bytes)
}

fn public_content_type(path: &str) -> Option<&'static str> {
    Some(match path.rsplit('.').next()? {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => return None,
    })
}

fn decode_path(raw: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(raw.len());
    let mut bytes = raw.bytes();
    while let Some(byte) = bytes.next() {
        decoded.push(if byte == b'%' {
            let high = char::from(bytes.next()?).to_digit(16)?;
            let low = char::from(bytes.next()?).to_digit(16)?;
            (high * 16 + low) as u8
        } else {
            byte
        });
    }
    String::from_utf8(decoded).ok()
}

fn running_result(url: &str) -> ToolResult {
    ToolResult {
        status: ToolStatus::Ok,
        body: json!({ "url": url, "status": "running" }).to_string(),
        truncated: false,
        error: None,
        evidence_id: None,
    }
}

fn failure(code: &'static str, message: &'static str) -> ToolResult {
    ToolResult::failure(ToolStatus::Error, code, message)
}

fn startup_timeout() -> ToolResult {
    failure("timed_out", "Preview startup exceeded the run deadline.")
}
