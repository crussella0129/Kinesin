# Sprint 10 core, context, accounting, and replay audit

This audit followed the completed intent-only pass. It reads implementation and
tests against INT-0001, INT-0002, INT-0004, and proposed INT-0006, INT-0014,
INT-0017, INT-0018. Findings below distinguish implemented invariants, concrete
defects, and acceptance evidence that has not been established. References are
to the pre-repair sprint 10 checkout.

## Findings requiring repair

### C1 — Unreported usage fields become reported zeros (P1, INT-0001)

`src/model.rs:35` defines `WireUsage` with `#[serde(default)]` on both `u64`
fields (`:36–39`). `decode_reply` converts that structure directly to `Usage`
(`:542`); streaming does the same at `:1027`. Consequently:

```text
usage:{}                         -> Some(prompt_tokens=0, completion_tokens=0)
usage:{prompt_tokens:7}          -> Some(prompt_tokens=7, completion_tokens=0)
usage:{completion_tokens:3}      -> Some(prompt_tokens=0, completion_tokens=3)
```

This contradicts the explicit honest-absence criterion. The existing tests cover
a complete object and an entirely absent object, not partial/empty objects:
`src/model.rs:1325–1369`, `tests/runner_tools.rs:988`,
`tests/cli_inspect.rs:323`.

Minimal repair: deserialize both counts independently as `Option<u64>`, carry
that knowledge through `Usage`, and omit each unknown counter independently.
Reported zero remains `Some(0)`. An empty usage object supplies no known count.
Exercise absent, empty, prompt-only, completion-only, and explicit-zero objects
in both response modes. Preserve compatibility with usage chunks that report
only `total_tokens` without inventing either component.

### C2 — Partial run coverage is presented as a complete token total (P1, INT-0001)

`RunJournal` has one `usage_reported` flag (`src/runner.rs:330–333`), set whenever
any call reports usage (`:826–834`). Terminal counters expose both accumulated
sums once that flag is true (`:340–346`). In a two-call run where only the first
call reports `(40,8)`, inspect therefore displays `(40,8)` as the run totals,
with no indication that the second call is unknown. A failed/cancelled model
exchange also returns no usage (`:808–818`). Existing accumulation tests make
every call report a complete object (`tests/runner_tools.rs:926–984`).

Minimal repair: track coverage independently for prompt and completion counts
across every attempted model dispatch. Expose a total only if all attempted
calls report that component; optionally expose separately named observed sums
and coverage counts if useful. A never-dispatched run should remain unknown.
Use checked addition: saturation at `u64::MAX` should not masquerade as an exact
total. Add mixed-present/absent and partial-component tests across multiple
calls, including a call that returns a transport error. Existing per-event
reported counts can remain useful independently of whether the run total is
known. This is runner metadata, not an authority or model-decision change.

### C3 — Freeform file-reading runs cannot discard old reads (P1, INT-0002)

Every `read_file` gets an evidence ID (`src/runner.rs:1070–1071`) and enters the
bounded evidence inventory (`:1227–1237`), even when acceptance is unchecked.
`RunState::drop_oldest_compactable` protects every group with an evidence ID
(`src/core.rs:304–313`), without consulting the state's existing `checked_task`
flag. Thus a freeform coding run dominated by successful file reads cannot
discard any old complete read groups. It reaches `history_bytes_limit`, or the
separate 65,536-byte evidence ceiling (`src/verification.rs:12,141–143`), despite
compaction being enabled and old reads being irrelevant to checked acceptance.

Source-level reproduction: create a normal freeform state with three completed
read-call/result groups, each carrying the ordinary top-level `evidence_id`,
then call `compact_until_fits` with floor 2 and a history limit slightly below
its current encoded length. It returns `(0,false)`; removing evidence IDs from
the otherwise identical groups permits a complete oldest group to be removed.
The existing core evidence-protection test constructs an unchecked state and
therefore encodes this overbroad behavior (`src/core.rs:650–675`). Existing
end-to-end compaction tests deliberately use bulky `list_files` groups rather
than ordinary reads (`tests/runner_tools.rs:1205`, `tests/replay.rs:802`).

Minimal repair: preserve evidence groups and retain the independent evidence
inventory for checked runs; permit whole old read groups to compact in
freeform runs and avoid retaining an unused verification inventory there.
Keep complete-group atomicity, the frozen initial prefix, recent floor,
journal observations, and checked-run evidence behavior. Update both runner
and replay together; decide explicitly how freeform evidence IDs remain stable
(a monotonically increasing ID counter or making unchecked reads evidence-free),
then bump the applicable replay semantic version rather than letting existing
captures silently change meaning. Tests should exercise a freeform run with
enough reads to cross both former bottlenecks, a checked counterpart retaining
every required observation, and replay of the repaired freeform run.

### C4 — Continuation admission excludes its actual prior-answer context (P2, INT-0002)

Authorization validates a cited answer's separate 8 KiB limit
(`src/policy.rs:295–303`) but computes initial conversation budget using only
instructions and the new prompt (`:388–396`). Actual initialization adds the
prior answer plus its framing as a second user message
(`src/core.rs:231–243`). A continuation can therefore be admitted and immediately
stop at `history_bytes_limit` before any model call; all those initial messages
are protected from compaction. Reuse the actual initial-conversation builder
when validating history/request budgets, including JSON framing. Test a prior
answer that individually fits 8 KiB but makes the combined conversation exceed
a narrowed run budget. This closes inconsistent admission, not general session
memory.

## Original cache/session acceptance remains open

### C5 — INT-0004's benchmark does not use the implemented session shape (P1 evidence gap)

Interactive turns are separate runs (`src/cli.rs:1111–1190`). Continuation
resolution reads only the previous run's retained candidate (`:775–796`), and
the new state is `[system, user(framed prior answer), user(new prompt)]`
(`src/core.rs:231–243`). It does not preserve the previous user prompt, assistant
role, tool observations, or full conversation prefix. The server can reuse a
stable system prefix, but this does not establish the original goal of reuse
of conversation context across interactive turns.

The offline prefix-extension test explicitly tests successive model calls
*within one run* (`tests/runner_tools.rs:1360–1424`). The live benchmark directly
POSTs hand-built messages using its own reqwest client
(`tests/live_evaluation.rs:460–520`); its second request manually includes the
original user prompt and assistant reply, which actual continuation does not.
It asserts evaluated tokens are fewer than total, not a controlled timing
reduction against cache-disabled runs. `cache_prompt` itself is correctly
emitted (`src/model.rs:236–240`) and frozen into replay settings
(`src/replay.rs:235`), with request parity tested (`tests/replay.rs:893`).

Repair planning must preserve the original promise: construct and test the
actual session continuation context with an immutable, frozen representation
that retains useful common prefixes while preserving origin/authority framing
and context budgets. A faithful benchmark should submit consecutive harness
session turns, retain their prepared requests, compare cache-on/off prompt
evaluation under a recorded workload/machine/server, and include concurrent
unrelated owner sessions and dropped/reassigned capacity. The current model
options contain only `cache_prompt`, no explicit session/slot owner identity
(`src/model.rs:97–116`). This does not demonstrate a data-leak bug by itself;
correctness may legitimately rely on server prefix matching, but the intent's
owner/concurrency/cache-lifetime criterion lacks focused evidence. Do not mark
the original live criterion complete solely because a replacement ignored
benchmark is authored or offline scripted tests pass.

## Criterion-level assessment

| Intent / criterion | Assessment and evidence |
| --- | --- |
| INT-0001 stored/inspect totals | Complete objects accumulate and inspect exposes the terminal counters (`src/runner.rs:826–834`, `src/cli.rs:848–868`; `tests/runner_tools.rs:926`, `tests/cli_inspect.rs:295`). C1/C2 qualify exactness and absence. |
| INT-0001 honest absence | Whole-object absence is tested; partial-object and partial-run absence are defective (C1/C2). |
| INT-0001 stream/nonstream | Both decode usage (`src/model.rs:542,1027`); both request/response paths tested. |
| INT-0001 storage/replay compatibility | Optional values live in existing event JSON. Replay knows token-counter names but deliberately does not recompute their values (`src/replay.rs:911–932`); no token totals authenticity should be claimed. |
| INT-0002 bounded continuation | Compaction repeatedly drops oldest eligible units (`src/model.rs:197–217`) at model and tool boundaries (`src/runner.rs:709,877,1255`). Positive tests cover list-heavy runs; ordinary freeform reads encounter C3. |
| INT-0002 complete tool groups | Group removal is whole assistant-call plus consecutive tool-result messages (`src/core.rs:286–302`), preserving recent partial/in-flight groups; `drop_oldest_removes_whole_group` tests the basic case (`:605`). A multi-call group test would strengthen coverage. |
| INT-0002 checked evidence | Groups with IDs are protected, and independent evidence records remain bounded (`src/verification.rs:68–154`). `checked_run_evidence_survives_compaction` (`tests/runner_tools.rs:1319`) passes acceptance while list groups compact. It introduces the evidence read after old bulky groups, so it is weaker than a test that tries to evict an old evidence group; the core test covers that latter shape. |
| INT-0002 immutable/replay | Shared pure helper runs in runner and replay (`src/replay.rs:597,780,950`); `replay_reproduces_a_compacted_run` (`tests/replay.rs:802`) verifies actual captured request parity. |
| INT-0004 measured session speed | Original criterion not established by current workload; C5. |
| INT-0004 immutability/replay | Cache flag is frozen and reproduced; dedicated replay test exists. |
| INT-0004 owner/concurrency lifetime | Server-managed prefix reuse is the implicit mechanism. No focused owner-interleaving/slot-loss proof found; C5. |
| INT-0006 origin-marked installed skills | Proposed, unimplemented. `RunAuthority` has configuration instructions, prompt, optional prior answer, and frozen MCP definitions (`src/policy.rs:166–190`), but no installed-skill input/discovery/loading mechanism. |
| INT-0006 untrusted workspace/budget | Existing immutable authority and file-result framing are a useful foundation, not implementation of installation consent or on-demand frozen loading. No dedicated skill acceptance tests found. |
| INT-0014 chain/signature | Proposed, unimplemented. `Event` and `Terminal` (`src/storage.rs:85–105`) contain ordinary metadata/digests, not chain links or receipt signatures. |
| INT-0014 replay/inspect tamper result | Replay explicitly limits its promise to internal consistency, excluding origin authentication (`src/replay.rs:937–941`); inspect displays summaries/counters. This is honest baseline behavior, not realized tamper evidence. |
| INT-0014 owner/privacy preservation | Owner fields and private replay remain existing foundations; cryptographic guarantees require new work. |
| INT-0017 approval/JIT | Proposed, unimplemented. `Effect` is Model/Tools/Candidate/Stop (`src/core.rs:127–133`), and allow-list authority is fixed for the run (`src/policy.rs:238–240`). No requested/approved/denied/timed-out action protocol or step elevation. The checked-task write prohibition (`src/policy.rs:376–379`) is an existing guard, not per-action approval. |
| INT-0018 child authority/budgets/replay/cancellation | Proposed, unimplemented. Run authority has owner/run identity but no parent identity or child budget derivation (`src/policy.rs:166–190`); core effects contain no spawn/gather operation. Independent admitted runs and pure replay are foundations, not subagent acceptance. |

## Validation conducted during this audit

`cargo build --lib --locked` passed on the current checkout. Findings C1–C5
come from source and test inspection; no implementation files were edited and
no new repository tests were added. A temporary standalone Rust reproduction
attempt for C1/C3 could not link because invoking rustc outside Cargo omitted
the transitive `windows.0.52.0.lib` native search path. It is not counted as an
executed reproduction; the report preserves the precise source-derived cases
for the planned regression tests. No live model measurement was executed.
