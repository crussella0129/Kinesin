# Connecting the harness to its model

The Rust CLI talks directly to a manually started `llama-server` over HTTP.
Keep the state machine synchronous; add Tokio and async reqwest for the first
real network operation. Later, the same adapter supports several admitted runs
and streaming. Kineserve is an optional process owner after those features work.
There is no FFI or extra forwarding process in this path.

Follow the [build guide](https://github.com/crussella0129/building-an-agent-harness/blob/main/build-guide.md) in order. The
[loop contract](loop-and-tools.md), [security policy](security.md), and
[resource limits](performance.md) define behavior the adapter must preserve.
[Task acceptance](verification.md) distinguishes a complete provider answer from
a checked task result.

## Prove one server and model combination

Use a native build appropriate for the target OS/GPU, or build a pinned llama.cpp
revision. Record server commit/build, backend, GGUF provenance and checksum,
weight quantization, template, actual context capacity, and launch arguments.
Native Windows is supported; WSL is a deployment choice.
[llama.cpp build guide](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)

Start with one slot. This is a command shape, not a command to run with a literal
placeholder:

```text
llama-server -m <model.gguf> --host 127.0.0.1 --port 8080 -c 4096 -np 1 --jinja --no-context-shift
```

On Windows the executable may be `llama-server.exe`. Supply a real quoted model
path and choose acceleration flags from that build's help. Check startup logs
for the actual context and template. Do not infer readiness from an open socket.

Before connecting Rust, exercise these exchanges using an ordinary HTTP client.
Save bounded, sanitized request/response fixtures under `tests/fixtures` when
you implement; that directory does not yet contain the completed test suite.

| Exchange | Required evidence |
|----------|-------------------|
| `GET /health` | Ready status from the actual server, with loading/failure distinguished |
| `GET /v1/models` | A verified model identifier for the profile |
| Plain `POST /v1/chat/completions` | One complete answer and understood finish reason |
| Same endpoint with one read-only tool | Identified tool call with valid argument encoding |
| Assistant tool call plus correlated tool result | Model uses the supplied result in its next reply |
| Greeting with tools available | Final answer can arrive without any tool call |
| Too-large prompt and generation near the context boundary | Context-error and truncation cases are classified without executing tools |
| Streaming text and streaming tool reply | Complete framing, finish marker, and assembly behavior verified |

The server documents these APIs but does not make a blanket compatibility
guarantee for every OpenAI-shaped client or model. `--jinja` enables template
support; successful tool use still depends on the model/template combination.
Verify it again when the server, model, or template changes.
[Server API](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md),
[function-calling guide](https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md)

## Add one async HTTP adapter

Use `serde` and `serde_json` for typed wire values. A concrete `ModelClient` enum
with `Scripted` and `Http` variants is sufficient initially. Give it ordinary
async methods and keep provider DTOs inside `model.rs` (the Koil responsibility).
The core consumes domain observations and never depends on reqwest or provider
JSON. Introduce a trait
only when a real caller benefits from it; a dynamically dispatched async trait
is not a prerequisite for this project.

Create a configured `reqwest::Client` once per endpoint/security profile and
clone it when a runner needs a handle. The client already contains a connection
pool and shared ownership; an extra `Arc<Client>` is unnecessary. Keep-alive
reuse avoids repeated connection setup but does not limit concurrent requests.
[reqwest Client](https://docs.rs/reqwest/latest/reqwest/struct.Client.html)

The configured `base_url` is an origin such as `http://127.0.0.1:8080`, without
`/v1`. Append the fixed chat path in the adapter. Model output cannot change the
endpoint. The initial policy is:

| HTTP behavior | Required configuration |
|---------------|------------------------|
| Redirects | Reject; a reply cannot choose a new model destination |
| Proxy | Disable automatic system/environment proxy use for the private model profile |
| Retries | Explicitly disable automatic retries for generation attempts |
| Connection | `connect_timeout_s` from the model profile |
| Total exchange | `request_timeout_s`, capped by remaining run time, including response-body consumption |
| Read stall | `read_timeout_s`, in addition to the total deadline |
| TLS | Validate certificates and hostname for HTTPS; never disable verification to make a connection work |
| Response size | Bound decoded bytes while consuming the body |

These settings require explicit choices: current reqwest defaults include
redirects, automatic proxy behavior, no total/read deadline, and protocol-NACK
retries. Use the selected release's documented feature names and lock it in
`Cargo.lock`. Do not copy dependency flags from an older tutorial unchecked.
[ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html),
[retry::never](https://docs.rs/reqwest/latest/reqwest/retry/fn.never.html)

Let the HTTP library handle HTTP message framing. Do not split a socket response
at a blank line and assume the rest is the complete body; persistent connections
and chunked transfer have protocol rules.
[HTTP/1.1 body framing](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.3)

## Prepare once, dispatch under the runner's authority

The model request sequence is deliberate:

1. Prepare the endpoint and exact serialized body without I/O. Validate its
   profile, conversation shape, request-byte bound, and applicable budgets.
2. Authorize the model effect and its data destination. Wait for model capacity
   within the shorter model-queue timeout and remaining run deadline.
3. Send the intent to the journal writer and await its committed acknowledgement.
   Store the appropriate metadata or opt-in replay delta described in
   [traces](traces.md), rather than copying the entire request each turn.
4. Recheck cancellation and the deadline. If dispatch is no longer allowed,
   resolve the recorded intent with a before-send outcome and release capacity.
5. Send exactly the already prepared bytes. Read the bounded result, classify
   it, and settle the outcome through the runner's journal path.
6. Only then let the core consume the normalized observation. Keep the exchange
   permit until model-observation settlement; release it before executing any
   subsequent tools or the task-acceptance checker. An ordinary complete answer
   is an answer candidate, not an accepted task.

The adapter does not write database events independently. No database or
filesystem operation blocks a Tokio worker. A delayed journal acknowledgement
does not permit dispatch using an expired run budget.

Classify timeout, transport failure, non-success HTTP status, oversized body,
malformed JSON, unsupported shape, incomplete stream, and normal completion
distinctly. Bound error bodies and error strings too. A timeout records an
abandoned attempt; it does not establish that server inference ended. Automatic
retry could consume capacity and create a second generation for an uncertain
first attempt.

## Finalize an answer candidate

After the model observation commits, release model capacity and keep the runner's
active reservation. For freeform work, acceptance is `unchecked`. For a checked
task, invoke the frozen `FileFieldsV1` checker from [verification](verification.md)
inline over the candidate and actual runner-owned file evidence. The checker
has no HTTP call, file reread, worker queue, or automatic repair loop.

Its initial bounds are 1–4 criteria, 8 KiB each for serialized specification,
checked candidate, and receipt, and 64 KiB retained evidence per run, alongside
the existing tool/history bounds. A model-provided citation or digest cannot
create evidence. The runner's observed bytes and identities determine what the
checker may use. Do not issue another model request to judge its own answer.

Check cancellation and remaining execution time before and after assessment.
The settlement grace permits recording existing outcomes, never launching or
repeating assessment. Before the terminal command enters the storage inbox,
apply observed cancellation/expiry instead of publishing an accepted task.
Once that command is accepted by the inbox, settle it without recall or a second
terminal write; a later cancellation/deadline cannot overwrite committed state.

Commit candidate/result, fingerprint, acceptance receipt/projection, and terminal
event in one transaction. Only that committed result is final. The execution
phase `completed` alone is insufficient: `task_accepted` requires both
`completed` and acceptance `passed`. GET, CLI, export, and terminal SSE carry
both outcomes. An observer closes after catching up with the atomic receipt,
not when the HTTP model response ends.

## Initial chat request policy

| Field | Policy |
|-------|--------|
| `model` | Identifier verified for the configured server profile |
| `messages` | Installed instructions, freeform user input or the frozen checked-task instruction, and complete retained conversation groups |
| `stream` | `false` for the first exchange; `true` at the streaming milestone |
| `n` | `1`; accept only the expected choice at index 0 |
| `max_tokens` | Map from the project's `max_output_tokens` limit |
| Sampling/template options | Explicit values from the tested model profile |
| `tools` | Advertise only capabilities granted to this run |
| `tool_choice` | `"auto"` when tools are advertised |
| `parallel_tool_calls` | Explicitly `false`; still validate any returned batch |
| `grammar`, `json_schema`, `response_format` | Omit for the initial tool loop |

Keep sampling advice with a particular model and template; there is no universal
temperature or thinking flag. The provisional output budget of 512 tokens may
be too small for some reasoning models. Evaluate an appropriate supported mode
or change the explicit budget after measuring; truncation remains an incomplete
outcome, never execution completion or task acceptance.

Preserve the assistant message containing tool calls and append each `tool`
result with its original `tool_call_id`. Decode the JSON inside
`function.arguments`, then validate its type, limits, and capability policy.
`finish_reason` and reply shape are interpreted together. An unknown or
incomplete finish state cannot dispatch tools. The canonical examples and
classification rules live in [loop-and-tools.md](loop-and-tools.md).

A checked task's final content must satisfy its typed output contract. Complete
chat framing and a recognized `stop` merely produce a candidate for that check;
well-framed narrative may still fail the output contract. The initial checked
task supplies the required shape in its frozen instruction and needs no special
provider grammar or `response_format` option.

Custom grammar plus tools has backend-specific restrictions. Keep it out of the
baseline and test a separate, tools-disabled structured-final-answer operation
if a later use case needs it. Do not assume that a valid generated JSON object
is authorized or semantically correct.
[llama.cpp request parsing](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/server-common.cpp)

## Assemble streaming replies completely

Incrementally process body bytes into SSE frames and provider deltas. A network
chunk can end in the middle of UTF-8, a line, an event, or an argument string.
Use a maintained parser where practical and test your integration with fragmented
fixtures. SSE supports comments, several line endings, and multiple `data`
lines; an event is dispatched at its framing boundary.
[SSE framing](https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream)

Set explicit implementation bounds before appending: at most 64 KiB for an
unfinished SSE frame, 64 KiB for accumulated tool arguments in one reply, and
the configured `max_response_bytes` for total decoded response bytes. The
history and tool-call limits still apply after assembly. Count content,
reasoning/extension fields, and ignored material toward the response-byte cap.
Reject an oversized frame even if the total-response allowance is larger.
Check the decoder's internal buffering as well as its emitted events. A library
that exposes only completed events needs a bounded input/framing guard or a
documented internal frame cap; checking size after it emits cannot enforce the
64 KiB unfinished-frame limit.

Correlate tool deltas by the provider's choice/call index and IDs. Require one
consistent completed name/ID per call, append argument fragments in order, and
reject contradictions or excess calls. A valid JSON prefix is insufficient:
wait for the tested terminal protocol, the complete reply, and batch validation
before authorizing any tool. Require the pinned provider's terminal marker
(such as `[DONE]`) as well as a recognized completion state. EOF or a broken
stream without that completion is incomplete; it may not turn into an answer or
a tool effect by accident.

Test split UTF-8, split `data:` lines, comments, multi-line data, several events
in one chunk, interleaved call deltas, missing IDs, duplicate terminal markers,
truncated arguments, stalls, oversized frames, and early EOF. A byte-arrival
timeout alone cannot catch an endless drip, so retain the total deadline.

Display text as provisional through bounded observer queues. Persist the
normalized completed observation once, then authorize the next effect or assess
an answer candidate. Provisional text remains provisional throughout checking;
only the atomic terminal result and receipt settle acceptance. Never block
response consumption indefinitely on a slow terminal or subscriber.
[Performance](performance.md) defines observer limits and permit lifetimes.

## Verify concurrent inference capacity

The first server baseline has one slot, so the harness uses one in-flight model
exchange even though its proposed general setting is two. At the concurrency
milestone, a candidate command is:

```text
llama-server -m <model.gguf> --host 127.0.0.1 --port 8080 -c 8192 -np 2 --jinja --no-context-shift
```

Verify the installed build's resulting capacity before enabling two model
requests. Do not assume each slot now has 4,096 tokens. Upstream context/KV
allocation options evolve, including unified-cache and per-slot limits. Inspect
startup logs and protected slot/metadata diagnostics where the selected build
supports them, then send simultaneous prompts near the intended context limit.
Never expose those diagnostics to untrusted service callers.

The inference server owns slots and continuous batching. The harness owns
session admission, model permits, policy, and deadlines. More active agent runs
do not require the same number of active inference slots: some runners may be
executing tools or waiting. Benchmark latency and useful throughput together.
[Server options](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md),
[batching internals](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README-dev.md)

For context preflight, probe the pinned server's chat input-token counting
extension. Include the same tools and template options as generation and reserve
output space against the actual configured slot context. Bare text tokenization
does not necessarily include chat/template/tool overhead. Bound the counting
request and include its time in the run deadline; it is another network
operation, not a free local check. If unsupported, retain a conservative profile
and explicit context-error handling rather than silently shifting history.
[Chat token counting](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md#post-v1chatcompletionsinput_tokens-token-counting)

## Optional backend acceleration

Speculative decoding is a backend capability, not a second model call added to
the runner. OoO-Spec needs target candidate verification and cache/tokenizer
integration; the ordinary chat endpoint supplies no such hook. If an independently
verified backend supports it, record that mode in its compatibility profile and
repeat the text/tool/stream, context, cancellation, and concurrent-load checks.
Do not advertise OoO-Spec support for the current llama.cpp profile merely because
both systems expose HTTP. [Research boundary](https://github.com/crussella0129/building-an-agent-harness/blob/main/paper-review.md#speculation-belongs-to-inference)

Treat any sidecar receiving dialogue/tool schemas, or historical-call cache, as
part of the approved data flow. Apply owner isolation, capture/retention policy,
request/schema size limits, and aggregate compute admission. A small returned
hint does not mean a small or non-sensitive sidecar request. The harness still
waits for a complete valid reply and authorizes each effect itself. Backend
verification of draft tokens also does not replace task-contract checking or
permit an unchecked answer to acquire a passed receipt.

## Optional Kineserve ownership

Add managed startup after concurrent runs and shutdown work. Attach mode always
remains available and never terminates the external server. Managed mode owns
one directly launched child shared by the controller, not one child per run.
Cancelling one session must not kill inference for every other session.

Use `tokio::process::Command` with separate arguments and retain the child
handle. Inherit output or drain bounded pipes continuously. Poll readiness under
a startup deadline while checking early process exit. A healthy unrelated
server on a conflicting port is not proof that the owned child started.

On controller shutdown or startup failure, stop and wait for the owned child.
Tokio documents that a dropped child normally keeps running; `kill_on_drop` is
only a fallback, and explicit cleanup remains necessary. Direct child ownership
does not supply portable process-tree cleanup. Windows Job Objects, wrappers,
and WSL bridges require separate ownership tests.
[Tokio process Child](https://docs.rs/tokio/latest/tokio/process/struct.Child.html),
[Command options](https://docs.rs/tokio/latest/tokio/process/struct.Command.html)

## Private remote inference and later shared service

An OS-managed WireGuard tunnel can make a model on another host reachable while
tools continue running beside the harness. Keep that data-flow distinction
visible to the operator: file results sent to inference leave the tool machine.
Configure the tunnel outside the Rust program. Bind/firewall the model endpoint
for the intended private path and verify rejection from unintended networks.
Keep tunnel keys and API credentials outside prompts and journal payloads.
[WireGuard quick start](https://www.wireguard.com/quickstart/),
[Windows tunnel service](https://git.zx2c4.com/wireguard-windows/about/docs/enterprise.md)

The later shared Kinesin API uses one controller behind TLS, authenticated
owner-scoped run access, and explicit service admission limits. This is a
different boundary from a private model connection. WireGuard supplies network
connectivity and encryption; it does not assign application run ownership or
fairly schedule tenants. Follow [security](security.md) and
[performance](performance.md) before exposing the service.

If mixing native Windows and WSL, test the actual NAT/mirrored network mode and
connection direction. Do not broaden listener binding merely to work around an
unexplained connection failure.
[Microsoft WSL networking](https://learn.microsoft.com/en-us/windows/wsl/networking)

These instructions describe intended integration work. No server was launched,
model downloaded, compatibility fixture captured, or throughput measured during
this documentation review.
