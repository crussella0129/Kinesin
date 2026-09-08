# Local model preflight

Guide step 4 was exercised on 2026-09-08 using ordinary PowerShell HTTP requests,
before the Rust HTTP adapter. Text, a complete correlated tool exchange, and
streaming were reproduced with the profile below. These are compatibility
checks with synthetic inputs, not a task-accuracy benchmark or proof that the
harness meets its acceptance contract.

## Tested profile

| Item | Recorded value |
|------|----------------|
| Server | llama.cpp **b6500**, commit prefix **a7a98e0f**, official Windows x64 Vulkan release |
| Compiler | Clang 19.1.5, x86_64-pc-windows-msvc |
| GPU | NVIDIA GeForce RTX 2080 Ti, driver 596.49, 11,264 MiB reported by NVIDIA |
| Host memory | 31.93 GiB physical; 9.83 GiB available before loading |
| Model | Existing `C:\Users\charl\Animus\Models\qwen2.5-coder-7b-instruct-q4_k_m.gguf` |
| Model bytes | 4,683,073,536 |
| Model SHA-256 | `509287f78cb4d4cf6b3843734733b914b2c158e43e22a7f4bf5e963800894d3c` |
| Model source revision | `13fb94bfda8c8cf22497dc57b78f391a9acb426a` |
| Quantization | Q4_K_M; server reports `Q4_K - Medium` |
| Template | Original embedded GGUF template, 2,509 UTF-8 bytes; server selects **Hermes 2 Pro** |
| Template SHA-256 | `d5495a1e5db0611132a97e46a65dbb64a642a499421228b9c8b93229097fa9a4` |
| Endpoint | `http://127.0.0.1:8080` |
| API model ID | `kinesin-qwen25-coder-7b` |
| Capacity | One slot, actual `n_ctx_slot = 4096`; no context shifting |
| GPU allocation | 29/29 layers offloaded; model 4,168.09 MiB, KV 224 MiB, compute 304 MiB |
| CPU allocation | Four generation and four prompt-processing threads; mapped model buffer 292.36 MiB |
| Probe sampling | `temperature = 0.2`, `n = 1`; 64 or 128 output tokens as saved per request |

The local model's checksum matches the hash published with the
[official Qwen GGUF](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/blob/13fb94bfda8c8cf22497dc57b78f391a9acb426a/qwen2.5-coder-7b-instruct-q4_k_m.gguf).
No model was downloaded or modified. The runtime came from the
[official b6500 release](https://github.com/ggml-org/llama.cpp/releases/tag/b6500).
Its ZIP checksum was verified against the release asset's SHA-256:
`d485ce9cdda9967d2b27a054bb7e16b57a56a332a4b54b2f02964b95ee591548`.
Runtime files and full process logs stay in ignored `validation-output/`.

## Reproduce the baseline

From the repository root, with port 8080 available, the tested command is:

```powershell
& '.\validation-output\runtime\llama-b6500-vulkan\llama-server.exe' -m 'C:\Users\charl\Animus\Models\qwen2.5-coder-7b-instruct-q4_k_m.gguf' --host 127.0.0.1 --port 8080 --alias kinesin-qwen25-coder-7b -c 4096 -np 1 --jinja --no-context-shift -ngl 99 -t 4 -tb 4 --no-webui
```

This launches a manually managed local inference server. The validation session
used a hidden owned process, retained its PID in
`validation-output/preflight-server.pid`, and stopped only that process during
the failure and restart checks. An existing external model service must never
be stopped to make this command succeed.

Wait for `/health` to report `{"status":"ok"}`. Reuse the exact saved JSON request
body with `Invoke-WebRequest -Method Post -ContentType 'application/json' -NoProxy`
against `/v1/chat/completions`. The authoritative request and response pairs are
in [the fixture directory](../tests/fixtures/live/README.md). Health and both text
and structured-tool shapes were reproduced after stopping and restarting this
same command.

The selected runtime is an isolated local compatibility baseline. Choosing it
does not establish that an older backend is appropriate for a shared deployment.
A backend upgrade must repeat these exchanges before its profile is considered
compatible. See the [pinned server documentation](https://github.com/ggml-org/llama.cpp/tree/b6500/tools/server)
and [pinned function-calling guide](https://github.com/ggml-org/llama.cpp/blob/b6500/docs/function-calling.md).

## Observed behavior

| Exchange | Observation | Meaning for the harness |
|----------|-------------|-------------------------|
| Health and models | HTTP 200; expected alias; one actual 4096-token slot | Readiness and profile identity verified independently |
| Plain text | `finish_reason = "stop"`, `Hello Kinesin.` | A complete answer candidate |
| File request | `finish_reason = "tool_calls"`, `content = null`, one `read_file` call | Decode the JSON **string** in `function.arguments`; retain its ID |
| Correlated synthetic tool result | Final answer states the observed language is Rust | Model can consume the supplied result; no real file capability was exercised here |
| Greeting with tools advertised | `stop`, ordinary greeting, no call | Tools remain optional with `tool_choice = "auto"` |
| Text and tool SSE | Complete deltas, recognized finish reason, then `[DONE]` | Wait for the entire terminal protocol before normalization/dispatch |
| Streaming usage | A final usage-only chunk has `choices: []` between finish and `[DONE]` | Permit that bounded metadata shape; it is not another choice or an early EOF |
| Oversized input | HTTP 400, `exceed_context_size_error`, 5030 input tokens versus 4096 context | No tool effect and no completed answer |
| Generation near context limit | HTTP 200, `length`, 4056 prompt + 40 completion tokens | Incomplete generation even though HTTP succeeded |
| Stopped owned server | Transport connection failure, no HTTP response | Distinguish transport failure from a model answer |
| Chat `input_tokens` extension | HTTP 404 | This profile cannot use that newer extension; retain explicit context-error handling |

No run-wide success rate, latency percentile, two-slot capacity, adversarial task
acceptance, or recovery behavior is established by these probes. Those belong
to subsequent guide checkpoints and must be tested through the Rust product.

## Why the installed build was not selected

The globally installed b10034 (`505b1ed15`) executable had CPU/RPC backends only.
An isolated official Vulkan b10034 build detected the GPU and passed text, but
its tested automatic tool requests produced ordinary text, malformed JSON, or
an unsupported description of a call. One response claimed Python without any
file result. The harness must never reinterpret such content as permission to
execute a tool or evidence that the task was solved.

The embedded template contains a double-brace example. Correcting that example
and separately trying the documented `chatml` fallback did not make these
b10034 probes pass. Those diagnostics are retained, including the candidate
template; **the selected profile uses neither override**. A diagnostic
`tool_choice = "required"` request was also unsuccessful and is not the baseline
policy.

With the same model and original automatic-tool request, b6500 selected Hermes
2 Pro and returned the required structured call, including after a controlled
restart. This narrows the compatibility difference to the tested combinations;
it is not a proof of the exact upstream defect or a general claim that every
b10034 model fails tool calling. The unsuccessful fixtures preserve the evidence
needed for a later focused backend investigation.
