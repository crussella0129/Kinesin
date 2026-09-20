# Diagnostic pair 1: action-schema ordering

**Result: historical control failed; ordered candidate passed the tiny file task.**
Independent read-only inspection of the retained files and journals on 2026-09-20
used the frozen [diagnostic card](../../sprint-research/diagnostic-card.md).
This qualifies the held-out native/ordered requests 3/4, not the full live gate.

| Observation | Request 1: historical control | Request 2: ordered candidate |
| --- | --- | --- |
| Run ID | `72427c08-d777-4c84-af2e-44e8a6d97200` | `030c89b2-43d0-430b-bb83-d0a741f7af04` |
| Capture/core/adapter/tools | `4/8/5/5` | `4/8/6/5` |
| Journal start UTC | `15:15:10.512` | `15:15:49.617` |
| First model result | Answer describing future steps | Actual `read_file` proposal, then successful dispatch |
| Actual tool results | None | `read_file`, `edit_file`, `write_file`: all executed/ok |
| Inventory | Unchanged: pencil quantity 11 | Pencil quantity 7; every other JSON value preserved |
| Required note | Missing | Exact bytes `pencil: 11 -> 7\n` |
| Independent task result | **FAIL** | **PASS** |
| Journal terminal state | completed / unchecked / answer_candidate | completed / unchecked / answer_candidate |
| Model turns / tool calls | 2 / 0 | 5 / 3 |
| Completion reviews / repair nudges | 1 / 0 | 1 / 0 |
| Prompt / completion tokens, summed | 2,821 / 140 | 8,478 / 202 |
| Journal elapsed / summed model durations | 2.417 s / 2.381 s | 2.945 s / 2.872 s |

Checker, source-parser and output-contract versions are each 1 in both captures.
The control's final claim that no files were found had no supporting read or list
operation. Completed/unchecked therefore does not override its failed file oracle.

The inventory seed and unchanged control hash are
`f401975d50fc5a8b04505b640878b85544f681417353365b2c82d4d4e8f5cd9b`.
The candidate inventory hash is
`b6e34edac4bb25969f1294151328d805fa2cc56a86d920c70ae69971c7ba3d74`;
its note hash is
`3359819fce7a2ef1b7426f6dde431be2cbbdc5f314505a4b8dbbbf41169c3cea`.
The candidate preserved currency, item identities/order, both unit prices,
notebook quantity and location. No extra app files were present.

Both actual accepted prompts equal the 345-byte frozen prompt, including its
terminal LF, SHA-256
`21a062de089de131b43ea3f2dc8b4e93beb291c225fec14b11cad5df4821dcc8`.
Both captures have empty prior/session context. The same frozen Qwen2.5-Coder-7B
Q4_K_M model, b6500 runtime, alias, temperature 0, runtime seed 0 and request caps
were used. Each initial request reported 1,358 prompt tokens.

Actual first wire bodies are each 10,438 bytes. Their bytes equal the respective
pre-dispatch expected bodies; freeze, relay HTTP-200 receipt and journal request
hashes agree:

- Control: `ad71de2f942f02dfa79c05699790c6c1ecafcf4a597d816d2907a7085d140a0e`.
- Ordered: `b59a38996734cbd74f43fdff2cabffd7bcaa5f0872bc1565e294b0ec2793c5e5`.

Reordering only each control tool variant's properties from
`arguments,kind,name` to `kind,name,arguments` reproduces the candidate first
body byte-for-byte. Answer properties remain `kind,text`; all decoded JSON values
are equal, including both available response branches. Later histories and
request-derived call IDs diverge and are not treated as matched wire inputs.

The executables differ: the control is the retained S14 attempt-7 binary
`0c26da235bf99301737bf4c388089e2932199a1223bb9ac125e0d98bb8cfb094`;
the candidate is source commit `ad058c5101c9f9e3bf296e24caee9dc2ac81cac5`, binary
`904ee8cd367f2e779734c1771815900e8de3697053c9bd8b3ecd6f01f3a91efa`.
The control freeze's `current_source_files` describes the contemporary candidate,
not the archived executable's source. First-wire equivalence constrains the
first-response comparison; it does not prove whole-binary equivalence.

Retained raw evidence is under `target/s15-live/call-01/` and `call-02/`:
freeze/started records, expected and relayed requests/responses, capture, SQLite
journal and actual app files. Capture hashes at assessment are respectively
`18608ee676f80c8ba5d55b6e089dd4cbb18db696c86071a9529d92590de49565` and
`16b5b0b8669595907588aad61a724a6b9cd1b10d00ea3217373a626ffc5282f1`.
A bounded byte-identical durable subset is indexed in [evidence/](evidence/README.md).
Request 1's driver hit a postprocessing `KeyError`; its capture was exported
manually without resubmitting the request. Its journal duration above is not
claimed as driver wall time. Both slots remain consumed. Started records report
zero corrective operator messages; active preparation/assessment time was not
measured by these journals. No authoring-speed or latency benefit is inferred.

This ordered-only pass supports the narrow ordering prediction on this fixture.
One sequential pair, different executables and a shared runtime/cache do not
establish general reliability or prove the cause of every S14 failure. No model
capacity, feedback-repair, storefront or same-session follow-up qualification
follows yet. Official unit/integration tests remain deferred.
