# Sprint 10: actual remote deployment and session measurements

Executed on 2026-09-12 during Build against the integrated capture-v3 harness.
These are scoped deployment/compatibility observations, not general model
accuracy or production certification. INT-0026 and INT-0027 remain proposed
because their broader continuity, concurrent-slot and deployment criteria remain.

## Deployment

| Item | Observed value |
| --- | --- |
| Client | Native Windows harness |
| Server | Separate Debian host, nighthawk, user charles |
| CPU / RAM | Intel i7-10700, 8 cores / 16 threads; approximately 16 GiB RAM |
| GPU | NVIDIA RTX 2070 Super, 8,192 MiB; Vulkan0 selected |
| Runtime | llama.cpp b6500, a7a98e0f, official Linux Vulkan asset |
| Runtime archive SHA-256 | ee15538d98808f15d13a474d2554bc708a475e35bf560e1d0d21b9e96bc5f8ea |
| Model | Qwen2.5-Coder-7B-Instruct Q4_K_M, 4,683,073,536 bytes |
| Model SHA-256 | 509287f78cb4d4cf6b3843734733b914b2c158e43e22a7f4bf5e963800894d3c |
| Model source revision | 13fb94bfda8c8cf22497dc57b78f391a9acb426a |
| Actual allocation | 29/29 layers on GPU; model 4,168.09 MiB, KV 224 MiB, compute 304 MiB |
| Verified capacity | One slot, 4,096 tokens; no context shifting |
| Remote listener | 127.0.0.1:18080 only, independently inspected with ss |
| Local endpoint | http://127.0.0.1:8080, through authenticated Tailscale SSH forwarding |
| Exposure check | Direct request to the server's tailnet model port failed with curl exit 7; forwarded health returned status ok |

The [official runtime release](https://github.com/ggml-org/llama.cpp/releases/tag/b6500)
archive hash was checked before extraction. The model hash matches the
[pinned preflight profile](../../../model-preflight.md). Downloads and logs were
staged in a dedicated validation directory; no system installation was required.
Tailscale SSH host verification remained enabled. Tailnet connectivity alone
was insufficient until the operator established host-side SSH access.

The GPU host launched the pinned server with:

```text
llama-server -m qwen2.5-coder-7b-instruct-q4_k_m.gguf --host 127.0.0.1 --port 18080 --alias kinesin-qwen25-coder-7b -c 4096 -np 1 --jinja --no-context-shift --device Vulkan0 -ngl 99 -t 4 -tb 4 --no-webui
```

The client established the local forward with:

```text
tailscale ssh charles@nighthawk -N -T -o ExitOnForwardFailure=yes -L 127.0.0.1:8080:127.0.0.1:18080
```

The test consumes independently recorded host/listener/tunnel provenance; it
does not infer two machines or encryption from a URL or an environment label.
Client configuration enforces HTTPS for direct non-loopback origins. This
deployment uses SSH for encryption between two loopback endpoints.

## Checked harness run

`secure_two_host_model_serving` passed through the production authorization,
HTTP adapter, local file capability, checker, journal and offline replay:

- Three model turns and one local read; 8,568 ms measured run time.
- Runtime completed; both required fields passed against the actual read.
- Pure replay reported consistent under capture version 3.
- No model port was exposed directly on the tailnet.

The source was synthetic project/language data. Replay and request hashes were
checked without reconnecting. This proves one tested configuration; it does not
establish arbitrary remote endpoints, remote MCP authorization or multi-owner
model capacity.

## Faithful session cache observation

The test executed actual immutable harness turns and carried the preceding
candidate through normal continuation authorization. A byte-preserving recording
proxy checked each outbound request against its journaled prepared hash. The
second turn retained a shared system prefix, not a fabricated growing transcript.

After two warmup sessions, six pairs alternated ordering. Every measured second
turn reported 1,597 total prompt tokens: 1,549 cached and 48 evaluated.

| Request setting | Six prompt-evaluation samples (ms) | Mean (ms) |
| --- | --- | --- |
| cache_prompt false: extension omitted | 248.043, 246.657, 247.086, 247.623, 246.804, 247.223 | 247.239 |
| cache_prompt true: extension emitted | 245.873, 245.912, 247.192, 247.341, 248.594, 249.321 | 247.372 |

This establishes shared-prefix reuse in actual sessions in both modes. It does
**not** establish a causal speedup from the flag: omission does not disable
b6500's caching. An earlier un-warmed three-pair mean was distorted by its first
sample and was not accepted as flag-benefit evidence. A true cache-disabled
control, full-history continuity and concurrent-slot behavior remain unverified.

The second-turn prepared hashes were stable within each mode:

- Omitted: 70a9e3fcb6880948cf01b5b26ada44eba9ccde689a540366d8880c8438fa5bea
- Emitted: 7c40ff35910694e9c187027a85770687149f4472663db5c6961aeebe6968f97a

## Reproduction and retained artifacts

With the pinned server and verified forward running, set
`KINESIN_REMOTE_PROVENANCE` to the JSON path produced by the independent
deployment checks, and `KINESIN_BENCH_MACHINE` to the actual hardware/runtime
provenance. Then execute:

```text
cargo test --locked --test live_evaluation secure_two_host_model_serving -- --ignored --exact --nocapture
cargo test --locked --test live_evaluation actual_session_cache_timing_measurement -- --ignored --exact --nocapture
```

The executed report is retained locally in
`validation-output/secure-two-host-213841a0-6eda-4b04-bb34-c5f6dbb832c6/report.json`;
deployment provenance is in `validation-output/s10-remote/provenance.json`;
full synthetic cache requests, candidates, timings and hashes are in
`target/cache-session-measurement.json`. Those runtime artifacts are ignored;
this chapter preserves the bounded reviewable measurements. Final cleanup and
whole-sprint results are recorded in the test report.
