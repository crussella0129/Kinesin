# Diagnostic pair 2: held-out protocol qualification

**Result: native failed; ordered passed. Select ordered for requests 5/6.**
Independent read-only inspection of retained files and journals on 2026-09-20
applied the [frozen card](../../sprint-research/diagnostic-card.md). The decision
uses these held-out outcomes, not request 2's earlier success. It does not yet
qualify repair, a full storefront or persistent same-session context.

| Observation | Request 3: native | Request 4: ordered |
| --- | --- | --- |
| Run ID | `a40346e2-18c3-411e-a503-3e8bb0712bae` | `5318c555-9eb6-4c15-b491-1d170adedd41` |
| Capture/core/adapter/tools | `4/7/4/5` | `4/8/6/5` |
| Journal start UTC | `15:17:03.459` | `15:17:50.096` |
| First result | Eight tool calls proposed together | One `read_file` call |
| Shipping JSON | Unchanged: `morning` | Only dispatch window changed to `evening` |
| Note contents | Literal `dispatch_window: OLD -> evening\n` | Exact `dispatch_window: morning -> evening\n` |
| Independent task result | **FAIL** | **PASS** |
| Model turns / tool calls | 3 / 8 | 5 / 3 |
| Tool results | 6 ok; 2 `match_not_found` errors | 3 ok |
| Completion reviews / repair nudges | 1 / 0 | 1 / 0 |
| Prompt / completion tokens, summed | 6,815 / 1,677 | 8,321 / 191 |
| Journal elapsed / summed model durations | 19.256 s / 19.151 s | 3.413 s / 3.317 s |

Checker, source-parser and output-contract versions are each 1. Both runs ended
completed/unchecked/answer_candidate; neither terminal label is the file oracle.
Native proposed dependent reads, edits and writes in its first batch, before any
read result could inform the later arguments. Its two exact-match edits used a
regex-looking `(\\w+)` pattern and failed. It wrote the placeholder note twice,
then described corrected contents as prose without further tool dispatch.
Ordered read the existing value, performed an exact edit, then wrote the correct
note. Carrier, package limit, sender and label format were preserved.

Seed/native JSON SHA-256:
`3cb026e20641fb7abef58339ce102d0c9336ce9ede5af1f492df6f30825449f7`.
Ordered JSON: `af047e0ad8d01cfe4b01ec079787349342ab699597cfed9f2b63f7d8b9d63371`.
Native wrong note: `867f1a4a03a2cedbbf77d6afcd8a2c091612e07ca72c8aa12dd64010dcc575c1`.
Ordered correct note: `edc1cbab585cba536f20d5e47419fd74c00d7689cacd548e98279590c238ca27`.
Both notes contain one LF; no extra app files were present.

Both accepted prompts equal the frozen 344-byte prompt, including its terminal
LF, hash `87445b8a3f177f729fa5d47334a8835daf4db028cf8e421508777a6c2cf0591b`.
Both have empty prior/session context and the same model/runtime, seed 0,
temperature 0 and resource limits. Both use source
`ad058c5101c9f9e3bf296e24caee9dc2ac81cac5`, binary
`904ee8cd367f2e779734c1771815900e8de3697053c9bd8b3ecd6f01f3a91efa`.

Each actual initial body equals its own frozen expected body; freeze, relay and
journal hashes agree. Native is 6,700 bytes, hash
`9dbebe2e75b2a6842c19ef4a90f2bdeaf5750f638b065c14be68beb5d89fde7c`;
ordered is 10,437 bytes, hash
`ffc6deb359f9721e71ad4469d336cd9c41e6fe17abbe7c26ca024efb94dd83f0`.
This compares native and structured protocols, including their tool framing and
single-action behavior. It is **not** another property-order-only comparison.
The batch/dependency behavior is observed; this pair cannot isolate which
protocol difference caused the outcome or establish a general speed advantage.

The two requests consume slots 3/4. Their started records report zero corrective
messages; active human preparation/assessment time is not measured by the
journals. No failed slot was replaced. Four of six diagnostic slots are used;
ordered is qualified only for the next unaided/assisted repair comparison.
Official unit/integration tests remain deferred.

Bounded byte-identical evidence for requests 1–4 is retained in
[evidence/](evidence/README.md), including these two captures, raw wire
requests/responses, profiles and final app files. Capture hashes are respectively
`61a3000a6ae3267feb733d844b8d36cae9660cc113618e4f5067df2697d3d057` and
`1c0065f91a7e3a59732ff9149ea9caf41f09a583d68ccfee05f5fdc0453ddbda`.
