# Controlled context and streaming comparisons

On 2026-09-08, all twelve measured checked context samples passed their frozen
file-field contracts. Eight paired text samples completed with unchecked
acceptance. Streaming exposed text sooner, while complete-answer latency showed
no consistent improvement. The text review also found presentation shortcomings.
Three warmups are retained separately, and every earlier task-card failure stays
intact. This comparison does not establish general model reliability.

The [driver](../tests/live_comparisons.rs) uses the real authority, runner,
capability read, independent file-fields checker and SQLite journal. Scripted
validation comes first; the separate ignored live test never launches or stops
the external model server. Every sample writes its own bounded result before
the next starts, including failed or inconclusive outcomes.

## Context comparison

Twelve measured checked samples cross two fresh value variants, three conditions,
and two repeats. One separate checked warmup precedes them. The same task alias,
required keys, source filename, generated instruction, tools, capture and budgets
apply to all conditions. Expected source values never appear in the prompt.

| Condition | Deliberate change |
|---|---|
| Opaque labels | All payload blocks in fixed order, with neutral section labels |
| Meaningful labels | Same payload blocks and order, with descriptive labels of the same byte length |
| Selected source | Meaningful labels, but irrelevant blocks removed before admission; every required field remains |

The first two conditions have identical total source bytes and required-field
positions. Assertions check the unchanged payload blocks and their order. Only
fixed-width comment labels change; the experiment does not shuffle evidence or
add instructions to source data. The selected condition changes input size
separately. It never trims a conversation after admission. Condition orders are
ABC/CBA for one variant and BCA/ACB for the other, balancing ordinal position.

The independent receipt establishes whether required fields match complete
actual observations. Four samples per condition are a small controlled probe,
not a reliability estimate or evidence of a general folder-layout advantage.

| Condition | Passed / measured | Source bytes | Median task time | Range |
|---|---:|---:|---:|---:|
| Opaque labels | 4 / 4 | 1,548–1,549 | 11.036 s | 11.000–11.455 s |
| Meaningful labels | 4 / 4 | 1,548–1,549 | 11.181 s | 11.010–11.186 s |
| Selected source | 4 / 4 | 78–79 | 10.678 s | 10.629–10.856 s |

Every context run made two model exchanges and one actual read, with zero
forbidden effects. For each value variant, all six candidates were byte-identical.
The first request was 1,049 bytes in every condition. The post-read request was
2,993–2,994 bytes for the equal-source-byte conditions and 1,471–1,472 bytes for
the selected source. The full measured section took 132.312 seconds, including
setup, admission, writer shutdown and result-file writes: 5.442 completed and
5.442 contract-passed runs per minute for this serial profile.

The label comparison found no acceptance advantage, and its four-per-condition
timings establish no reliable performance difference. Selection retained all
required evidence and had a lower median in these samples. Equal source bytes
do not establish equal model token counts: token usage is unknown in the retained
adapter observations. Prompt-cache state, tokenization and ordinary runtime
variation prevent attributing these small differences to a single mechanism.

## Streaming comparison

Eight measured text runs cross a short and a longer general explanation prompt,
streaming off/on, and two repeats. Two mode-specific warmups are retained
separately. Mode order alternates. The prompt and settings stay fixed per pair;
no tools or answers are inserted into the prompts. Freeform remains unchecked,
with a separate manual rubric for explanation accuracy and requested form.

The bounded observer records the first non-whitespace displayable text frame.
For nonstreaming, text becomes visible to this consumer when the durable final
candidate returns, matching the CLI's delivery rule. Both use the same run
start clock. The recorded pre-send control timestamp yields dispatch-to-visible
time; this includes delivery to the controller consumer, not physical display
rendering. It is neither first HTTP byte nor precise first-token timing.

The report also preserves full run time, request hashes/bytes, final candidate
hash/bytes, frames, receipt and failures. Different final outputs are identified
before attributing timing differences to streaming. Raw paired timings and
median/range are appropriate for this sample; a precise tail-latency claim is not.

| Prompt / repeat | First visible: stream | First visible: complete | Durable result returned: stream | Durable result returned: complete |
|---|---:|---:|---:|---:|
| Short / 1 | 0.102 s | 3.294 s | 3.284 s | 3.298 s |
| Short / 2 | 0.102 s | 3.303 s | 3.370 s | 3.307 s |
| Long / 1 | 1.020 s | 18.720 s | 19.177 s | 18.724 s |
| Long / 2 | 0.101 s | 17.897 s | 17.980 s | 17.901 s |

First-visible times begin at the recorded pre-send observation; durable-result
times begin at the admitted run's start and include terminal acknowledgement
and result retrieval. They have deliberately different origins.
Every pair returned exactly the same final bytes and SHA-256: 138 bytes for the
short prompt and 824 for the long prompt. The streamed outputs produced 32 and
178 bounded frames respectively, with no observer lag or partial-tool effect.
Frame counts are not reported as token counts. A first frame can be only the
beginning of a word, so its arrival does not mean a complete useful answer exists.

The measured text section took 87.297 seconds: 5.498 completed runs per minute
and zero runtime contract passes. All eight receipts remained unchecked. The
two short-prompt warmups preceded these pairs; the model stayed loaded, and
cache reuse was neither disabled nor reset between samples. These are serial
warm-model observations, with no cold-start, additional-slot or p95 claim.

Codex reviewed the complete final texts against the requested explanations,
examples and form; this was a separate qualitative review, not another runtime
checker. All eight explanations supplied appropriate concepts/examples. The
short answer's second punctuated unit was a sentence fragment. The long answer
had six sentences, but several were lengthy despite the request for short ones.
Their content requirements were met and presentation was only partly satisfied.
The [review](../tests/fixtures/live-comparisons/review.json) records those limits
without upgrading any receipt or counting all instruction requirements as met.

## Reproduction and profile

After the scripted test passes and other measurements finish, provision an owned
loopback attach server using the pinned b6500 Vulkan/zero-layer-offload profile
in [model evidence](../tests/fixtures/live/zero-offload-profile.json). Retain the
new process identity, startup logs, executable/model identities and host load.
Zero offloaded layers still permits a Vulkan compute allocation; it is not a
claim of GPU-independent execution.

The measured server was an owned hidden loopback process on native Windows. It
used the original embedded Hermes 2 Pro template, one 4,096-token slot, temperature
zero, 192 output tokens, four generation/prompt threads, and port 8083. Startup
confirmed zero of 29 offloaded layers, 4,460.45 MiB CPU model storage, 224 MiB CPU
KV storage, and 827.47 MiB Vulkan compute storage. Other applications remained;
no other agent compilation, tests or model work overlapped this comparison.
The final slot was idle, and the exact process identity was checked locally
before stopping and joining only this server. Machine identifiers, process IDs,
exact launch timestamps and system-wide host snapshots remain in ignored local
evidence; the published profile contains the reproducible workload settings.

This used a debug test executable for model/task comparison. It does not replace
the release-build runtime-overhead benchmark. The production baseline was
`17ecbc36fcdc9946c907d4807b2e9d8d19c933ed`, with the then-uncommitted comparison
driver. The [provenance record](../tests/fixtures/live-comparisons/provenance.json)
binds the driver source, exact measured executable, lockfile and raw report by
SHA-256. Later commits must not be treated as the measured executable merely
because their tests pass.

```powershell
cargo test --test live_comparisons scripted_controlled_context_and_streaming_contracts -- --nocapture
$env:KINESIN_COMPARISON_ENDPOINT = 'http://127.0.0.1:8083'
cargo test --test live_comparisons pinned_live_context_and_streaming_comparisons -- --ignored --nocapture
```

The driver prints its fresh ignored output directory. It retains twenty measured
samples and three warmups; failures are not replaced by extra attempts. The
[raw report](../tests/fixtures/live-comparisons/report.json),
[server profile](../tests/fixtures/live-comparisons/server-profile.json),
[measurement scope](../tests/fixtures/live-comparisons/host-before.json), and
[cleanup outcome](../tests/fixtures/live-comparisons/host-after.json)
preserve the public evidence for this run. The raw report's review fields retain their original pending
value; the linked separate review records the subsequent assessment.
