# Synthetic live preflight fixtures

Captured on 2026-09-08 with the local model profile in
[model-preflight.md](../../../docs/model-preflight.md). These files are provider
compatibility observations. The tool result was supplied by the preflight HTTP
client using synthetic `project=Kinesin` and `language=Rust` data; these captures
do not prove a Rust tool, checker, journal, or complete run was implemented.

All individual captures are below 64 KiB. JSON values are preserved, with a
trailing file newline added when saving. The SSE files preserve provider framing
and may also have a trailing file newline. Generated call IDs and timestamps are
synthetic-session metadata, not credentials. No private workspace file contents,
absolute model path, or authentication header occurs in the captured bodies.

## Selected b6500 profile

| Files | Purpose |
|-------|---------|
| `health.response.json`, `models.response.json`, `slots.response.json` | Ready server, API alias, actual capacity |
| `text.request.json`, `text.response.json` | Plain complete text |
| `tool-call.request.json`, `tool-call.response.json` | One actual automatic structured tool call |
| `tool-result.request.json`, `tool-result.response.json` | Original assistant call plus correlated synthetic result, then final answer |
| `greeting-with-tools.*.json` | Tools advertised, ordinary answer selected |
| `text-stream.request.json`, `text-stream.response.sse` | Complete streaming text protocol |
| `tool-call-stream.request.json`, `tool-call-stream.response.sse` | Streaming tool deltas and completed arguments |
| `context-overflow.*.json` | HTTP 400 input-context rejection |
| `context-boundary.*.json` | HTTP 200 with `length` at the generation boundary |
| `input-tokens.response.json` | HTTP 404 for unsupported chat token-counting extension; input was `tool-call.request.json` |
| `text-after-restart.response.json`, `tool-call-after-restart.response.json` | Same request and command reproduced after restart |
| `server-b6500-stopped.failure.json` | Sanitized connection-failure category after stopping the owned server |
| `tool-b6500.*.json` | Original successful comparison capture, identical in substance to `tool-call.*.json` |

Tool-call-only content is null in the non-streaming response. Arguments are a JSON
string inside the outer JSON. Stream tool arguments arrive in several fragments;
the final usage-only chunk has an empty `choices` array before `[DONE]`.

## Unsuccessful b10034 diagnostics

`health-b10034`, `models-b10034`, `slots-b10034`, and `text-b10034` show the earlier
build's basic text support. `server-stopped.failure.json` records its controlled
stop. These are not the selected profile.

| Prefix | Template/request combination |
|--------|------------------------------|
| `tool-described` | Original embedded template; ordinary automatic-tool request; fenced JSON text returned |
| `tool-malformed` | Original template; additional format instruction; malformed JSON text returned |
| `tool-corrected-template` | Candidate double-brace correction; ordinary request; unsupported XML and an unevidenced Python answer returned |
| `tool-explicit` | Candidate correction plus explicit format example; ordinary JSON text returned |
| `tool-required` | Candidate correction plus diagnostic required-tool selection; generation ended with `length` |
| `tool-chatml` | Built-in `chatml` fallback; model declined file access |

`model-template-b10034-candidate.jinja` is the unsuccessful diagnostic override,
derived from the user's local Apache-2.0 Qwen model template. It is **not** part of
the selected launch command. Its SHA-256 is
`cd8e9439f0570856fd70470bf8889ebd8b5d1107207f67a5efb46e342330527f`.

## Zero-layer-offload fallback

The `zero-offload-*` files retain a separate 2026-09-08 bounded compatibility
probe after severe GPU contention. See `zero-offload-profile.json` for its scope,
timings, and the remaining Vulkan compute allocation. Text and tool requests
reuse `text.request.json` and `tool-call.request.json`; the correlated request
uses the actual fresh tool-call ID. This does not replace the GPU baseline.
