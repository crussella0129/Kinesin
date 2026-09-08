//! Koil: prepare immutable wire bytes, then execute a model exchange.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use futures_util::{FutureExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;

use crate::config::ModelConfig;
use crate::core::{Message, ModelReply, Role};

#[derive(Clone, Debug)]
pub struct ModelOptions {
    pub origin: String,
    pub served_model: String,
    pub temperature: f64,
    pub max_output_tokens: usize,
    pub max_request_bytes: usize,
    pub max_response_bytes: usize,
    pub stream: bool,
    pub tools: Vec<String>,
    /// A JSON Schema for the whole reply. Mutually exclusive with `tools`: the
    /// server installs its own grammar for tool calls from the chat template,
    /// so a second constraint has no slot. The builder enforces this by
    /// construction rather than by convention.
    pub constraint: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct PreparedRequest {
    origin: String,
    bytes: Vec<u8>,
    sha256: String,
    max_response_bytes: usize,
    stream: bool,
}

impl PreparedRequest {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn max_response_bytes(&self) -> usize {
        self.max_response_bytes
    }
    pub fn is_streaming(&self) -> bool {
        self.stream
    }
}

pub fn fingerprint(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn conversation_json(messages: &[Message]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            let role = match message.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };
            let mut wire = json!({"role":role,"content":message.content});
            if !message.tool_calls.is_empty() {
                wire["tool_calls"] = Value::Array(
                    message
                        .tool_calls
                        .iter()
                        .map(|call| {
                            json!({
                                "id":call.id,"type":"function",
                                "function":{"name":call.name,"arguments":call.arguments}
                            })
                        })
                        .collect(),
                );
            }
            if let Some(id) = &message.tool_call_id {
                wire["tool_call_id"] = json!(id);
            }
            wire
        })
        .collect()
}

pub fn prepare(messages: &[Message], options: &ModelOptions) -> Result<PreparedRequest, String> {
    if messages.is_empty()
        || options.max_output_tokens == 0
        || options.max_response_bytes == 0
        || !options.temperature.is_finite()
    {
        return Err("invalid model request settings".into());
    }
    let mut request = json!({
        "model":options.served_model,"messages":conversation_json(messages),
        "n":1,"temperature":options.temperature,"max_tokens":options.max_output_tokens,
        "stream":options.stream
    });
    if let Some(schema) = &options.constraint {
        // Constrained turn: never carries tools.
        request["response_format"] = json!({
            "type": "json_schema",
            "json_schema": {"name": "kinesin_candidate", "schema": schema, "strict": true}
        });
    } else if !options.tools.is_empty() {
        let definitions = options.tools.iter().map(|name| {
            let (description, parameters) = match name.as_str() {
                "read_file" => ("Read bounded UTF-8 file contents inside the workspace. Use a relative path. Output reports truncation and an evidence_id for the actual observation.", json!({
                    "type":"object","properties":{"path":{"type":"string"}},
                    "required":["path"],"additionalProperties":false
                })),
                "list_files" => ("List a bounded nonrecursive subset of entries inside the workspace. Use a relative path, or . for its root. Output reports incomplete results.", json!({
                    "type":"object","properties":{"path":{"type":"string"}},
                    "required":["path"],"additionalProperties":false
                })),
                "search_files" => ("Find a literal term in workspace text files below a relative path, or . for its root. Returns matching file names and line numbers, not whole files, and reports incomplete results. Prefer a distinctive term: matching is exact text, so an absent term returns nothing rather than something similar. It cites no evidence: read a file to observe a value you intend to report.", json!({
                    "type":"object","properties":{
                        "path":{"type":"string"},
                        "query":{"type":"string","description":"Literal text to find. Not a pattern or expression."},
                        "case_sensitive":{"type":"boolean","description":"Omit to ignore capitalization, which is usually what you want."}
                    },
                    "required":["path","query"],"additionalProperties":false
                })),
                "write_file" => ("Create or replace one text file at a relative workspace path. Provide the complete new contents; the write replaces the whole file atomically. It reports the bytes written and cites no evidence. It cannot create directories, write through a symbolic link, or leave the workspace.", json!({
                    "type":"object","properties":{
                        "path":{"type":"string"},
                        "content":{"type":"string","description":"The complete new file contents."}
                    },
                    "required":["path","content"],"additionalProperties":false
                })),
                "edit_file" => ("Replace one exact passage in an existing workspace text file. Provide find, the exact current text, and replace, its new text. The find text must appear exactly once; if it is absent or appears more than once the edit is refused rather than guessed. Make find long enough to be unique. It cites no evidence and cannot leave the workspace or write through a symbolic link.", json!({
                    "type":"object","properties":{
                        "path":{"type":"string"},
                        "find":{"type":"string","description":"The exact current text to replace. Must occur exactly once."},
                        "replace":{"type":"string","description":"The new text."}
                    },
                    "required":["path","find","replace"],"additionalProperties":false
                })),
                "delete_file" => ("Delete one regular file at a relative workspace path. It refuses a directory or a symbolic link, and a missing file is an error. It cannot be undone, so delete only a file you are sure about.", json!({
                    "type":"object","properties":{"path":{"type":"string"}},
                    "required":["path"],"additionalProperties":false
                })),
                "move_file" => ("Rename or move one regular file from path to a new relative workspace path. The destination must not already exist, so a move never overwrites another file. Both paths stay inside the workspace.", json!({
                    "type":"object","properties":{
                        "path":{"type":"string"},
                        "to":{"type":"string","description":"The new relative path. Must not already exist."}
                    },
                    "required":["path","to"],"additionalProperties":false
                })),
                _ => return Err("unsupported compiled tool".to_owned()),
            };
            Ok(json!({"type":"function","function":{
                "name":name,"description":description,"parameters":parameters
            }}))
        }).collect::<Result<Vec<_>, String>>()?;
        request["tools"] = Value::Array(definitions);
        request["tool_choice"] = json!("auto");
        request["parallel_tool_calls"] = json!(false);
    }
    let bytes = serde_json::to_vec(&request).map_err(|_| "request serialization failed")?;
    if bytes.len() > options.max_request_bytes {
        return Err("request_bytes_limit".into());
    }
    Ok(PreparedRequest {
        origin: options.origin.clone(),
        sha256: fingerprint(&bytes),
        bytes,
        max_response_bytes: options.max_response_bytes,
        stream: options.stream,
    })
}

#[derive(Clone, Debug)]
pub struct ScriptStep {
    pub delay: Duration,
    pub reply: ModelReply,
}

impl From<ModelReply> for ScriptStep {
    fn from(reply: ModelReply) -> Self {
        Self {
            delay: Duration::ZERO,
            reply,
        }
    }
}

#[derive(Debug)]
pub struct ScriptedClient {
    script: Mutex<VecDeque<ScriptStep>>,
    requests: Mutex<Vec<Vec<u8>>>,
}

#[derive(Clone, Debug)]
pub enum ModelClient {
    Scripted(Arc<ScriptedClient>),
    Http(HttpClient),
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    client: reqwest::Client,
    origin: String,
    max_response_bytes: usize,
}

impl HttpClient {
    async fn metadata(&self, path: &str) -> Result<Value, String> {
        let mut response = self
            .client
            .get(format!("{}{}", self.origin.trim_end_matches('/'), path))
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map_err(|_| "model_readiness_transport")?;
        if !response.status().is_success()
            || response.content_length().is_some_and(|size| size > 65_536)
            || response
                .headers()
                .get(reqwest::header::CONTENT_ENCODING)
                .is_some_and(|encoding| encoding != "identity")
        {
            return Err("model_readiness_response".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "model_readiness_transport")?
        {
            if chunk.len() > 65_536 - bytes.len() {
                return Err("model_readiness_limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| "model_readiness_response".into())
    }
    pub fn new(profile: &ModelConfig, max_response_bytes: usize) -> Result<Self, String> {
        let origin = url::Url::parse(&profile.base_url).map_err(|_| "invalid model origin")?;
        if !matches!(origin.scheme(), "http" | "https")
            || origin.host_str().is_none()
            || !origin.username().is_empty()
            || origin.password().is_some()
            || origin.query().is_some()
            || origin.fragment().is_some()
            || !matches!(origin.path(), "" | "/")
            || max_response_bytes == 0
        {
            return Err("invalid model origin or response limit".into());
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .retry(reqwest::retry::never())
            .timeout(Duration::from_secs(profile.request_timeout_s))
            .connect_timeout(Duration::from_secs(profile.connect_timeout_s))
            .read_timeout(Duration::from_secs(profile.read_timeout_s))
            .build()
            .map_err(|_| "model HTTP client setup failed")?;
        Ok(Self {
            client,
            origin: profile.base_url.trim_end_matches('/').to_owned(),
            max_response_bytes,
        })
    }

    async fn send(
        &self,
        request: &PreparedRequest,
        mut observer: Option<&mut TextObserver>,
    ) -> Result<ModelReply, String> {
        if request.origin.trim_end_matches('/') != self.origin {
            return Err("prepared_destination_mismatch".into());
        }
        let mut response = self
            .client
            .post(format!("{}/v1/chat/completions", self.origin))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request.bytes.clone())
            .send()
            .await
            .map_err(http_error)?;
        let status = response.status();
        let limit = self.max_response_bytes.min(request.max_response_bytes);
        if response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .is_some_and(|value| value.as_bytes() != b"identity")
        {
            return Err("unsupported_content_encoding".into());
        }
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err("response_bytes_limit".into());
        }
        if request.stream && status.is_success() {
            let is_sse = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(';').next())
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/event-stream"));
            if !is_sse {
                return Err("unsupported_stream_content_type".into());
            }
            let mut decoder = StreamDecoder::new(limit)?;
            while let Some(chunk) = response.chunk().await.map_err(http_error)? {
                decoder.push_with_text(&chunk, observer.as_deref_mut())?;
            }
            return decoder.finish();
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(http_error)? {
            if chunk.len() > limit.saturating_sub(bytes.len()) {
                return Err("response_bytes_limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        if !status.is_success() {
            return Err(format!("http_status_{}", status.as_u16()));
        }
        decode_reply(&bytes)
    }
}

fn http_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "model_timeout"
    } else if error.is_connect() {
        "model_connection_failed"
    } else {
        "model_transport_failed"
    }
    .into()
}

#[derive(Deserialize)]
struct WireResponse {
    choices: Vec<WireChoice>,
    error: Option<Value>,
}
#[derive(Deserialize)]
struct WireChoice {
    index: usize,
    finish_reason: String,
    message: WireMessage,
}
#[derive(Deserialize)]
struct WireMessage {
    role: String,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<WireCall>,
}
#[derive(Deserialize)]
struct WireCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: WireFunction,
}
#[derive(Deserialize)]
struct WireFunction {
    name: String,
    arguments: String,
}

pub fn decode_reply(bytes: &[u8]) -> Result<ModelReply, String> {
    let wire: WireResponse = serde_json::from_slice(bytes).map_err(|_| "invalid_provider_json")?;
    if wire.error.is_some() {
        return Err("provider_error".into());
    }
    let [choice] =
        <[WireChoice; 1]>::try_from(wire.choices).map_err(|_| "protocol_choice_count")?;
    if choice.index != 0 || choice.message.role != "assistant" {
        return Err("protocol_choice_or_role".into());
    }
    if choice.finish_reason == "length" {
        return Ok(ModelReply::Incomplete("generation_length".into()));
    }
    match choice.finish_reason.as_str() {
        "stop" if choice.message.tool_calls.is_empty() => {
            let content = choice.message.content.ok_or("empty_response")?;
            if content.trim().is_empty() {
                return Err("empty_response".into());
            }
            Ok(ModelReply::Answer(content))
        }
        "tool_calls" if !choice.message.tool_calls.is_empty() => {
            let mut ids = std::collections::HashSet::new();
            let calls = choice
                .message
                .tool_calls
                .into_iter()
                .map(|call| {
                    if call.kind != "function"
                        || call.id.is_empty()
                        || call.id.len() > 128
                        || call.function.name.is_empty()
                        || call.function.name.len() > 64
                        || call.function.arguments.len() > 65536
                        || !ids.insert(call.id.clone())
                    {
                        return Err("protocol_tool_call".to_owned());
                    }
                    Ok(crate::core::ToolCall {
                        id: call.id,
                        name: call.function.name,
                        arguments: call.function.arguments,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(ModelReply::ToolCalls {
                content: choice.message.content,
                calls,
            })
        }
        _ => Err("protocol_finish_reason".into()),
    }
}

impl ModelClient {
    /// Bounded llama-server readiness/identity checks, not generation requests.
    pub async fn ready(&self, profile: &ModelConfig) -> bool {
        let Self::Http(client) = self else {
            return true;
        };
        if client.origin != profile.base_url {
            return false;
        }
        let Ok(health) = client.metadata("/health").await else {
            return false;
        };
        if health["status"] != "ok" {
            return false;
        }
        let Ok(models) = client.metadata("/v1/models").await else {
            return false;
        };
        if !models["data"]
            .as_array()
            .is_some_and(|models| models.iter().any(|model| model["id"] == profile.model_id))
        {
            return false;
        }
        let Ok(slots) = client.metadata("/slots").await else {
            return false;
        };
        slots.as_array().is_some_and(|slots| {
            slots.len() >= profile.verified_slots
                && slots.iter().take(profile.verified_slots).all(|slot| {
                    slot["n_ctx"]
                        .as_u64()
                        .is_some_and(|size| size >= u64::from(profile.context_size))
                })
        })
    }
    pub fn http(profile: &ModelConfig, max_response_bytes: usize) -> Result<Self, String> {
        HttpClient::new(profile, max_response_bytes).map(Self::Http)
    }

    pub fn origin(&self) -> Option<&str> {
        match self {
            Self::Scripted(_) => None,
            Self::Http(client) => Some(&client.origin),
        }
    }
    pub fn scripted(steps: impl IntoIterator<Item = ScriptStep>) -> Self {
        Self::Scripted(Arc::new(ScriptedClient {
            script: Mutex::new(steps.into_iter().collect()),
            requests: Mutex::new(Vec::new()),
        }))
    }

    pub fn captured_requests(&self) -> Result<Vec<Vec<u8>>, String> {
        match self {
            Self::Http(_) => Err("live requests are not captured outside the journal".into()),
            Self::Scripted(client) => client
                .requests
                .lock()
                .map(|requests| requests.clone())
                .map_err(|_| "scripted request lock poisoned".into()),
        }
    }

    pub async fn send(&self, request: &PreparedRequest) -> Result<ModelReply, String> {
        self.send_inner(request, None).await
    }

    /// Text is provisional. The caller owns this bounded observer and its
    /// display task; dropping it cannot cancel or authorize model/tool work.
    pub async fn send_with_text(
        &self,
        request: &PreparedRequest,
        observer: &mut TextObserver,
    ) -> Result<ModelReply, String> {
        self.send_inner(request, Some(observer)).await
    }

    async fn send_inner(
        &self,
        request: &PreparedRequest,
        observer: Option<&mut TextObserver>,
    ) -> Result<ModelReply, String> {
        match self {
            Self::Http(client) => client.send(request, observer).await,
            Self::Scripted(client) => {
                client
                    .requests
                    .lock()
                    .map_err(|_| "scripted request lock poisoned")?
                    .push(request.bytes.clone());
                let step = client
                    .script
                    .lock()
                    .map_err(|_| "script lock poisoned")?
                    .pop_front()
                    .ok_or("script_exhausted")?;
                if !step.delay.is_zero() {
                    tokio::time::sleep(step.delay).await;
                }
                check_scripted_size(&step.reply, request.max_response_bytes)?;
                if request.stream
                    && let (Some(observer), ModelReply::Answer(text)) = (observer, &step.reply)
                {
                    observer.emit(text);
                }
                Ok(step.reply)
            }
        }
    }
}

fn check_scripted_size(reply: &ModelReply, limit: usize) -> Result<(), String> {
    // The fake has no HTTP body. Count its complete normalized payload before
    // returning it, including JSON string encoding, under the same per-run cap.
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit - self.bytes {
                return Err(std::io::Error::other("response byte limit"));
            }
            self.bytes += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { bytes: 0, limit };
    let mut count = |text: &str| {
        serde_json::to_writer(&mut counter, text).map_err(|_| "response_bytes_limit".to_owned())
    };
    match reply {
        ModelReply::Answer(text) | ModelReply::Failure(text) | ModelReply::Incomplete(text) => {
            count(text)?
        }
        ModelReply::ToolCalls { content, calls } => {
            if let Some(text) = content {
                count(text)?;
            }
            for call in calls {
                count(&call.id)?;
                count(&call.name)?;
                count(&call.arguments)?;
            }
        }
    }
    Ok(())
}

/// A per-exchange/run display path: 128 pending messages of at most 2 KiB each.
/// A lagging reader is disconnected without waiting on the model's effect path.
#[derive(Clone)]
pub struct TextObserver {
    sender: Arc<Mutex<Option<mpsc::Sender<String>>>>,
    lagged: Arc<AtomicBool>,
}
pub struct TextReceiver {
    receiver: mpsc::Receiver<String>,
    lagged: Arc<AtomicBool>,
}
impl TextObserver {
    pub fn bounded() -> (Self, TextReceiver) {
        let (sender, receiver) = mpsc::channel(128);
        let lagged = Arc::new(AtomicBool::new(false));
        (
            Self {
                sender: Arc::new(Mutex::new(Some(sender))),
                lagged: Arc::clone(&lagged),
            },
            TextReceiver { receiver, lagged },
        )
    }
    fn emit(&mut self, mut text: &str) {
        let Ok(mut shared_sender) = self.sender.lock() else {
            return;
        };
        while !text.is_empty() {
            let Some(sender) = &*shared_sender else {
                return;
            };
            let mut end = text.len().min(2048);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            match sender.try_send(text[..end].to_owned()) {
                Ok(()) => text = &text[end..],
                Err(mpsc::error::TrySendError::Full(_)) => {
                    self.lagged.store(true, Ordering::Release);
                    *shared_sender = None;
                    return;
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    *shared_sender = None;
                    return;
                }
            }
        }
    }
}
impl TextReceiver {
    pub async fn recv(&mut self) -> Option<String> {
        self.receiver.recv().await
    }
    pub fn is_lagged(&self) -> bool {
        self.lagged.load(Ordering::Acquire)
    }
}

const MAX_SSE_FRAME_BYTES: usize = 64 * 1024;
const MAX_STREAM_ARGUMENT_BYTES: usize = 64 * 1024;
const MAX_STREAM_CALLS: usize = 24;

/// Bounded SSE framing plus the selected chat-completions delta protocol.
/// `finish` is the only operation that returns an executable complete reply.
/// Malformed input poisons this decoder; ignoring an error cannot recover a pass.
pub struct StreamDecoder {
    limit: usize,
    received: usize,
    frame_bytes: usize,
    line: Vec<u8>,
    frame: Vec<u8>,
    pending_cr: bool,
    first_line: bool,
    failed: Option<String>,
    reply: StreamingReply,
}

impl StreamDecoder {
    pub fn new(max_response_bytes: usize) -> Result<Self, String> {
        if max_response_bytes == 0 {
            return Err("invalid_stream_limit".into());
        }
        Ok(Self {
            limit: max_response_bytes,
            received: 0,
            frame_bytes: 0,
            line: Vec::new(),
            frame: Vec::new(),
            pending_cr: false,
            first_line: true,
            failed: None,
            reply: StreamingReply::default(),
        })
    }
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.push_with_text(bytes, None)
    }
    fn push_with_text(
        &mut self,
        bytes: &[u8],
        observer: Option<&mut TextObserver>,
    ) -> Result<(), String> {
        if let Some(problem) = &self.failed {
            return Err(problem.clone());
        }
        let result = self.push_inner(bytes, observer);
        if let Err(problem) = &result {
            self.failed = Some(problem.clone());
        }
        result
    }
    fn push_inner(
        &mut self,
        bytes: &[u8],
        mut observer: Option<&mut TextObserver>,
    ) -> Result<(), String> {
        if bytes.len() > self.limit - self.received {
            return Err("response_bytes_limit".into());
        }
        self.received += bytes.len();
        for &byte in bytes {
            // Defer CR's line boundary until we know whether LF follows. This
            // counts CRLF bytes in their original frame, including split pairs.
            if self.pending_cr {
                self.pending_cr = false;
                if byte == b'\n' {
                    self.count_frame_byte()?;
                    self.finish_line(observer.as_deref_mut())?;
                    continue;
                }
                self.finish_line(observer.as_deref_mut())?;
            }
            self.count_frame_byte()?;
            match byte {
                b'\r' => self.pending_cr = true,
                b'\n' => self.finish_line(observer.as_deref_mut())?,
                _ => self.line.push(byte),
            }
        }
        Ok(())
    }
    fn count_frame_byte(&mut self) -> Result<(), String> {
        if self.frame_bytes == MAX_SSE_FRAME_BYTES {
            return Err("stream_frame_bytes_limit".into());
        }
        self.frame_bytes += 1;
        Ok(())
    }
    fn finish_line(&mut self, observer: Option<&mut TextObserver>) -> Result<(), String> {
        let bytes = std::mem::take(&mut self.line);
        let line = std::str::from_utf8(&bytes).map_err(|_| "stream_invalid_utf8")?;
        let line = if self.first_line {
            self.first_line = false;
            line.strip_prefix('\u{feff}').unwrap_or(line)
        } else {
            line
        };
        // The guard counts raw bytes before allocation. Once CR/CRLF is
        // unambiguous, normalize the boundary to LF (never increasing size).
        // The maintained decoder owns SSE field, comment, and data semantics.
        self.frame.extend_from_slice(line.as_bytes());
        self.frame.push(b'\n');
        if line.is_empty() {
            self.frame_bytes = 0;
            let frame = std::mem::take(&mut self.frame);
            // One complete bounded frame is already available: polling an
            // iterator-backed stream never waits on an executor or performs I/O.
            // The prefix comment prevents eventsource-stream 0.2.3's initial
            // BOM byte-slicing path; the guard stripped only the actual initial
            // BOM above. A later BOM remains ordinary SSE field data.
            let source = futures_util::stream::iter([
                Ok::<_, std::convert::Infallible>(&b":\n"[..]),
                Ok(frame.as_slice()),
            ]);
            let mut decoder = eventsource_stream::EventStream::new(source);
            if let Some(event) = decoder
                .next()
                .now_or_never()
                .ok_or("stream_decoder_pending")?
            {
                let event = event.map_err(|_| "invalid_sse_frame")?;
                if event.event != "message" {
                    return Err("unsupported_stream_event".into());
                }
                self.reply.observe(&event.data, observer)?;
            }
        }
        Ok(())
    }
    pub fn finish(mut self) -> Result<ModelReply, String> {
        if let Some(problem) = self.failed {
            return Err(problem);
        }
        if self.pending_cr {
            self.pending_cr = false;
            self.finish_line(None)?;
        }
        if !self.line.is_empty() || !self.frame.is_empty() {
            return Err("stream_incomplete".into());
        }
        self.reply.finish()
    }
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
    usage: Option<Value>,
    error: Option<Value>,
}
#[derive(Deserialize)]
struct StreamChoice {
    index: usize,
    delta: Option<StreamDelta>,
    finish_reason: Option<String>,
}
#[derive(Deserialize, Default)]
struct StreamDelta {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<StreamToolDelta>>,
}
#[derive(Deserialize)]
struct StreamToolDelta {
    index: usize,
    id: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    function: Option<StreamFunctionDelta>,
}
#[derive(Deserialize)]
struct StreamFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}
#[derive(Default)]
struct PartialTool {
    id: Option<String>,
    kind: Option<String>,
    name: Option<String>,
    arguments: String,
}
#[derive(Default)]
struct StreamingReply {
    role_seen: bool,
    content: String,
    content_seen: bool,
    tools: BTreeMap<usize, PartialTool>,
    argument_bytes: usize,
    finish_reason: Option<String>,
    usage_seen: bool,
    done: bool,
}

impl StreamingReply {
    fn observe(&mut self, data: &str, observer: Option<&mut TextObserver>) -> Result<(), String> {
        if self.done {
            return Err("stream_after_done".into());
        }
        if data == "[DONE]" {
            if self.finish_reason.is_none() {
                return Err("stream_done_before_finish".into());
            }
            self.done = true;
            return Ok(());
        }
        let chunk: StreamChunk = serde_json::from_str(data).map_err(|_| "invalid_stream_json")?;
        if chunk.error.is_some() {
            return Err("provider_error".into());
        }
        if chunk.choices.is_empty() {
            if self.finish_reason.is_none()
                || self.usage_seen
                || !chunk.usage.as_ref().is_some_and(Value::is_object)
            {
                return Err("protocol_stream_usage".into());
            }
            self.usage_seen = true;
            return Ok(());
        }
        if self.finish_reason.is_some() {
            return Err("stream_after_finish".into());
        }
        let [choice] =
            <[StreamChoice; 1]>::try_from(chunk.choices).map_err(|_| "protocol_choice_count")?;
        if choice.index != 0 {
            return Err("protocol_choice_or_role".into());
        }
        if choice.delta.is_none() && choice.finish_reason.is_none() {
            return Err("protocol_stream_delta".into());
        }
        let delta = choice.delta.unwrap_or_default();
        if let Some(role) = delta.role {
            if role != "assistant" {
                return Err("protocol_choice_or_role".into());
            }
            self.role_seen = true;
        }
        if !self.role_seen
            && (delta.content.as_ref().is_some_and(|v| !v.is_empty())
                || delta.tool_calls.as_ref().is_some_and(|v| !v.is_empty()))
        {
            return Err("protocol_stream_missing_role".into());
        }
        if let Some(text) = delta.content {
            self.content_seen = true;
            self.content.push_str(&text);
            if let Some(observer) = observer {
                observer.emit(&text);
            }
        }
        if let Some(calls) = delta.tool_calls {
            if calls.len() > MAX_STREAM_CALLS {
                return Err("stream_tool_count_limit".into());
            }
            let mut seen = std::collections::HashSet::new();
            for call in calls {
                if call.index >= MAX_STREAM_CALLS || !seen.insert(call.index) {
                    return Err("protocol_stream_tool_index".into());
                }
                let partial = self.tools.entry(call.index).or_default();
                merge_identity(&mut partial.id, call.id, 128)?;
                merge_identity(&mut partial.kind, call.kind, 16)?;
                if partial.kind.as_ref().is_some_and(|kind| kind != "function") {
                    return Err("protocol_tool_call".into());
                }
                if let Some(function) = call.function {
                    merge_identity(&mut partial.name, function.name, 64)?;
                    if let Some(arguments) = function.arguments {
                        if arguments.len() > MAX_STREAM_ARGUMENT_BYTES - self.argument_bytes {
                            return Err("stream_argument_bytes_limit".into());
                        }
                        self.argument_bytes += arguments.len();
                        partial.arguments.push_str(&arguments);
                    }
                }
            }
        }
        if let Some(reason) = choice.finish_reason {
            if !matches!(reason.as_str(), "stop" | "tool_calls" | "length") {
                return Err("protocol_finish_reason".into());
            }
            self.finish_reason = Some(reason);
        }
        Ok(())
    }
    fn finish(self) -> Result<ModelReply, String> {
        if !self.done {
            return Err("stream_incomplete".into());
        }
        if !self.role_seen {
            return Err("protocol_stream_missing_role".into());
        }
        let reason = self.finish_reason.ok_or("stream_incomplete")?;
        if reason == "length" {
            return Ok(ModelReply::Incomplete("generation_length".into()));
        }
        let mut calls = Vec::new();
        let mut ids = std::collections::HashSet::new();
        for (expected, (index, partial)) in self.tools.into_iter().enumerate() {
            if index != expected {
                return Err("protocol_stream_tool_index".into());
            }
            let id = partial.id.ok_or("protocol_tool_call")?;
            if !ids.insert(id.clone()) || partial.kind.as_deref() != Some("function") {
                return Err("protocol_tool_call".into());
            }
            calls.push(crate::core::ToolCall {
                id,
                name: partial.name.ok_or("protocol_tool_call")?,
                arguments: partial.arguments,
            });
        }
        match reason.as_str() {
            "stop" if calls.is_empty() && !self.content.trim().is_empty() => {
                Ok(ModelReply::Answer(self.content))
            }
            "stop" if calls.is_empty() => Err("empty_response".into()),
            "tool_calls" if !calls.is_empty() => Ok(ModelReply::ToolCalls {
                content: self.content_seen.then_some(self.content),
                calls,
            }),
            _ => Err("protocol_finish_reason".into()),
        }
    }
}

fn merge_identity(
    target: &mut Option<String>,
    incoming: Option<String>,
    limit: usize,
) -> Result<(), String> {
    if let Some(value) = incoming {
        // The tested protocol sends complete identities once (or repeats the
        // same identity). Only function arguments are concatenated fragments.
        if value.trim().is_empty()
            || value.len() > limit
            || value.chars().any(char::is_control)
            || target.as_ref().is_some_and(|known| known != &value)
        {
            return Err("protocol_stream_identity".into());
        }
        *target = Some(value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(tools: Vec<String>, constraint: Option<Value>) -> ModelOptions {
        ModelOptions {
            origin: "http://127.0.0.1:8080".into(),
            served_model: "fixture".into(),
            temperature: 0.0,
            max_output_tokens: 64,
            max_request_bytes: 131_072,
            max_response_bytes: 65_536,
            stream: false,
            tools,
            constraint,
        }
    }

    fn body(options: &ModelOptions) -> Value {
        let messages = [Message::text(Role::User, "hello".into())];
        let prepared = prepare(&messages, options).expect("prepared request");
        serde_json::from_slice(prepared.bytes()).expect("request is JSON")
    }

    #[test]
    fn a_constrained_request_never_carries_tools() {
        // The server installs its own grammar for tool calls from the chat
        // template, so a second constraint has no slot to occupy.
        let schema = json!({"type":"object"});
        let constrained = body(&base(vec!["read_file".into()], Some(schema.clone())));
        assert_eq!(constrained["response_format"]["type"], "json_schema");
        assert_eq!(
            constrained["response_format"]["json_schema"]["schema"],
            schema
        );
        assert_eq!(
            constrained["response_format"]["json_schema"]["strict"],
            true
        );
        assert!(
            constrained.get("tools").is_none(),
            "a constrained turn withdraws tools by construction"
        );
        assert!(constrained.get("tool_choice").is_none());

        let gathering = body(&base(vec!["read_file".into()], None));
        assert!(gathering.get("response_format").is_none());
        assert_eq!(gathering["tools"][0]["function"]["name"], "read_file");
        assert_eq!(gathering["tool_choice"], "auto");

        let plain = body(&base(Vec::new(), None));
        assert!(plain.get("tools").is_none() && plain.get("response_format").is_none());
    }

    use crate::core;

    pub(crate) fn options() -> ModelOptions {
        ModelOptions {
            origin: "scripted".into(),
            served_model: "scripted".into(),
            temperature: 0.0,
            max_output_tokens: 64,
            max_request_bytes: 131072,
            max_response_bytes: 1048576,
            stream: false,
            tools: Vec::new(),
            constraint: None,
        }
    }

    #[tokio::test]
    async fn preparation_and_script_use_the_same_bytes() {
        let (state, _) = core::initiate("Instructions".into(), "Question".into()).unwrap();
        let prepared = prepare(state.messages(), &options()).unwrap();
        let client = ModelClient::scripted([ModelReply::Answer("Answer".into()).into()]);
        assert_eq!(
            client.send(&prepared).await.unwrap(),
            ModelReply::Answer("Answer".into())
        );
        assert_eq!(
            client.captured_requests().unwrap(),
            vec![prepared.bytes().to_vec()]
        );
        assert_eq!(prepared.sha256(), fingerprint(prepared.bytes()));
        assert_eq!(
            client.send(&prepared).await.unwrap_err(),
            "script_exhausted"
        );
    }

    #[test]
    fn request_bounds_include_encoding_and_tool_definitions() {
        let (state, _) = core::initiate("\\\"".repeat(20), "x".into()).unwrap();
        let mut settings = options();
        settings.max_request_bytes = 80;
        assert_eq!(
            prepare(state.messages(), &settings).unwrap_err(),
            "request_bytes_limit"
        );
        settings.max_request_bytes = 131072;
        settings.tools.push("read_file".into());
        let wire: Value =
            serde_json::from_slice(prepare(state.messages(), &settings).unwrap().bytes()).unwrap();
        assert_eq!(wire["parallel_tool_calls"], false);
        assert_eq!(wire["n"], 1);
        assert!(wire.get("response_format").is_none());
    }

    #[test]
    fn zero_delay_script_is_ready_without_a_timer_runtime() {
        use futures_util::FutureExt;

        let (state, _) = core::initiate("Instructions".into(), "Question".into()).unwrap();
        let prepared = prepare(state.messages(), &options()).unwrap();
        let client = ModelClient::scripted([ModelReply::Answer("Answer".into()).into()]);
        assert_eq!(
            client.send(&prepared).now_or_never(),
            Some(Ok(ModelReply::Answer("Answer".into())))
        );
        assert_eq!(client.captured_requests().unwrap().len(), 1);
    }
}
