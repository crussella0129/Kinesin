# Diagnostic pair 3: unaided versus observed repair

**Both requests failed. All six diagnostic slots are consumed; T-123 and T-124
remain blocked by the unmet unaided-repair prerequisite.** Independent read-only
inspection of the captures, wire bodies and app files on 2026-09-20 applied the
[frozen diagnostic card](../../sprint-research/diagnostic-card.md). No full
storefront attempt or official unit/integration check is justified by this pair.

| Observation | Request 5: unaided | Request 6: authentic browser feedback |
| --- | --- | --- |
| Run ID | `88a1318a-c848-44ca-afb6-4f34229410d6` | `2f6fb621-84f5-4323-9ff8-d63f7971e045` |
| Capture/core/adapter/tools | `4/8/6/5` | `4/8/6/5` |
| Journal start UTC | `15:25:16.681` | `15:25:41.678` |
| Model turns / tool calls | 2 / 0 | 2 / 0 |
| Completion reviews / repair nudges | 1 / 0 | 1 / 0 |
| Actual app edits / previews | 0 / 0 | 0 / 0 |
| Independent repair result | **FAIL** | **FAIL** |
| Prompt / completion tokens, summed | 2,783 / 77 | 3,271 / 123 |
| Journal elapsed / summed model durations | 1.368 s / 1.331 s | 1.945 s / 1.905 s |

Checker, source-parser and output-contract versions are each 1. Both terminal
states are completed/unchecked/answer_candidate, not verified requested work.
Neither request exhausted its token, turn, tool or run-time budget.

Request 5 asked the user to supply code and then said no files were provided.
It did not claim a successful repair, but the missing-files assertion was not an
observation: the app files existed in the granted workspace, and list/read tools
were available. No inspection was attempted. Request 6 initially claimed to
have changed an `updateCart` function in `cart.js`, then claimed to have edited
JavaScript. The actual seed uses `app.js`; both claims are contradicted by zero
tool dispatches and byte-identical final files. The completion review did not
produce inspection or corrective action in either arm.

The authentic [preparation observation](repair-preparation/observation.json)
records Codex operating the unmodified seed through the in-app browser at
`2026-09-20T15:22:48.497Z` (engine version unavailable). Adding the $2 Pencil and
$5 Notebook displayed two items but total $5 rather than $7. Search for Notebook
and clearing search worked. The observation is bound to these seed hashes; both
final app copies have exactly the same three files and hashes:

- `app.js`: `8bfccd3e533a710807b711031661a228a443e73b9f7bdc22ffffb22c8fc062ff`.
- `index.html`: `b7876f7e7e6ec72d7445b6220b1adc66b7e4cc4acd60b5759b6a3cec43af1ee5`.
- `style.css`: `ca72c560519bb7b4b158c5b907566a7bb4bb4332d8ed3105667ad493a21f0df7`.

No model-created preview existed for a post-repair browser check. The unchanged
bytes show that no repair was applied to the observed defective seed; they are
not presented as a new browser observation or as evidence that repair ran.

Both arms use source `ad058c5101c9f9e3bf296e24caee9dc2ac81cac5`, binary
`904ee8cd367f2e779734c1771815900e8de3697053c9bd8b3ecd6f01f3a91efa`, the same
Qwen2.5-Coder-7B Q4_K_M/b6500 backend, temperature 0, seed 0 and original caps.
Their prior/session contexts are empty. Actual first bodies equal their frozen
expected bodies, with matching freeze/relay/journal hashes:

- Unaided: 10,391 bytes; `dc2e4e804d9988fb766bc732b8210f04778d5533ae2ca016ff16424efa5ffdea`.
- Assisted: 11,162 bytes; `067c343e33faf81cead461e4140ff1f28616e6bbef1b297eeedfcf6566df4061`.

Decoded first bodies differ only in the final user message. Accepted prompts
equal their pre-dispatch frozen files: 298 bytes/hash
`f8618671b422710147b38008511c17446e999c40e91a22976100e454842e08c6` and
1,069 bytes/hash `2398a18ee1ff1d83187fa4b97a6837b22776819b46b775eecee68c80693bec58`.
The recorded one-line serialization inserts a space and the authentic observation
before the base prompt's terminal LF; its base sentence is unchanged. This was
frozen before dispatch, avoiding extra CLI requests from embedded line breaks.
The assisted prompt supplies observed behavior and arithmetic, not patch code.

Slot 6 is explicitly operator-assisted diagnostic input. Neither arm received a
subsequent corrective prompt. Session close records report exit code 0; recorded
session elapsed times (21.015/28.063 s) include the period awaiting closure and
must not be substituted for the journal request durations above. Human active
preparation/assessment time is not measured by these journals.

This pair fails the prediction that adding this authentic observation would be
sufficient to obtain a repair on this fixture. Failure occurs before file
inspection/action selection, so it does not isolate code-repair capacity or prove
that automatic browser feedback can never help. The tiny file-task gains in
pairs 1/2 remain real but did not transfer to this repair request. The locked stop
rule applies: retain the failures, do not add replacement diagnostic slots, and
do not proceed to the full workload or official tests.

Raw evidence is retained under [evidence/](evidence/README.md). Capture hashes:
`3fa2a046f8a019657847cf8c486bf5c9ec44a5292d0f1d1b872a3496f2696b6d` and
`8931a3c3357cecd1f986f2e070251b40893a583d3cb7d6b50790cadb4ca76787`.
