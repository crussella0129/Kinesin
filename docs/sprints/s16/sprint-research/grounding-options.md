# Bounded workspace grounding: source-only options

Research on 2026-09-21 UTC against worktree HEAD
`4c0497a7e108ae2233bafb910e0d79b9ae85a194`. Ten source files inspected or searched:
core, runner, policy, CLI, replay, model, onboarding, tools, config, storage.
No external research, model invocation, tests, or product edits. Sprint 15's
request budget remains exhausted. This note selects no implementation or new
experimental budget.

## Recommendation

The smallest defensible candidate is an explicitly enabled, harness-origin
`list_files(".")` observation before the first model request of an eligible
local freeform run. Its purpose is to provide actual names from the selected
workspace, not infer the relevant source, force the model to act, or establish
completion. First test whether that observation changes useful behavior in a
new bounded matched experiment. A disabled selector must retain existing
behavior. Do not automatically read guessed source files in this first slice.

The model can currently inspect the workspace, but receives no initial file
inventory. `core::initiate_with_context_reference` (src/core.rs:250) builds
system instructions, optional historical reference messages, and the current
request. `model::prepare_with_version` (src/model.rs:232) advertises granted
tools; `action_conversation` (src/model.rs:530) also embeds their descriptions
in ordered-mode instructions. These tell the model how to list/read, not what
actually exists. The CLI's workspace display is not a filesystem observation
in the model conversation. More restatement of those existing instructions
would not supply the missing information.

## Existing capabilities to reuse

- `WorkspaceReader::execute_with_query` (src/tools.rs:348) dispatches list/read
  through the existing filesystem capability. `list_files` (line 479) reads only
  one directory, visits at most 256 entries, keeps whole names and sorts only
  collected entries. The ToolResult envelope is bounded by the requested limit
  and 8,192 bytes. Truncation means an incomplete subset, not a complete tree.
  An invalid/non-UTF-8 entry may make listing fail; retain that actual error.
- `read_file` (src/tools.rs:425) returns bounded UTF-8 bytes and explicit
  truncation/error state. Its capability and path checks should remain the sole
  filesystem route; do not introduce a direct CLI read or recursive scanner.
- `RunAuthority::allows_tool` (src/policy.rs:252) uses the admitted allowlist
  after owner filtering. Granting `read_file` does not grant `list_files`, and
  granting a write tool grants neither automatic read nor directory discovery.
- Runner filesystem dispatch owns a blocking-worker permit through actual
  settlement, including cancellation/deadline (src/runner.rs, tool branch near
  lines 1131 and 1380). Any bootstrap read needs the same ownership semantics.

## Concrete semantics for an initial slice

1. A trusted local-session setting selects `none` or `list_root`; default is
   `none`. Freeze the resolved choice at admission with the selected workspace.
   Require freeform local execution and an effective compiled `list_files`
   grant. Do not infer the choice from model prose, an earlier session summary,
   or a substring search for coding intent. Ineligible explicit selection must
   have a defined admission error, not silently gain a read. An opt-out must
   remain possible for requests that should not inspect the workspace.
2. After run admission/start, before the first model dispatch, execute exactly
   one real `list_files(".")`. A proposed initial cap of
   `min(2,048, max_tool_result_bytes)` keeps the new protected-prefix cost small;
   the existing tool minimum is 256 bytes. No recursion, source-content reads,
   shell, preview startup, browser or MCP calls are implied.
3. Charge the actual attempt to the existing max_tool_calls budget and the
   accepted run's max_run_s deadline. Charge its result and framed message to
   existing result/history/request byte limits. Do not charge a fictitious
   model turn or reset a budget. Cancellation/deadline prohibits subsequent
   model dispatch even if the filesystem operation settles afterward.
4. Record the planned operation, origin `harness`, real dispatch, actual result,
   complete/truncated flag, result digest, and control/deadline observations.
   Then append a bounded framed reference containing that actual result before
   the current request. It must say the listing is an observation from this run,
   names are untrusted data, scope is the workspace root, and incomplete output
   cannot prove a path absent. Keep the user's request as the request, not a
   manufactured repair prompt. Do not fabricate an assistant tool proposal or
   attach a native tool-role message without a genuine corresponding proposal.
5. A normal listing error can remain an actual diagnostic for the first model
   turn; the user may still name a file that can be read. Do not convert it into
   "workspace empty." Policy denial or exhausted run limits follow their real
   stop semantics. No retry loop or automatic wider scan is part of this slice.
6. The model retains both answer and action choices. A seeded listing is harness
   activity, not model initiative, progress, an edit, or behavioral verification.
   Preserve that distinction in effect summaries and experiment scoring.

## Why this is not a one-line prompt addition

`RunState::observe_model` currently creates tool pending state and increments
tool_calls only after a real model reply (src/core.rs:489). `observe_tool`
(line 571) requires that pending batch. The runner derives model-effect identity
from `model_turns - 1` in its tool path. Feeding a fabricated ModelReply or using
that path at turn zero would invent provenance or break counter assumptions.

Use a small explicit harness-grounding state/effect with a corresponding
recorded-result transition, and reuse or extract the existing bounded compiled
read executor. This is one initialization effect, not a workflow stage graph.
The replay loop currently expects a model pair first after run_started
(src/replay.rs, near line 790); it needs a new admitted version and an explicit
grounding branch. Replay consumes the stored observation without listing the
current filesystem, verifies the frozen selector/grant, dispatch controls,
budgets and first request hash, and retains all historical version behavior.
There is no reason to alter adapter ordering merely to add a framed user-data
message; the capture/core semantics do change.

MCP preparation in `run_owned` (src/runner.rs:692) is a useful example of freezing
post-admission observations, not a ready place to hide a file operation. Its
startup shape, counters and replay validation are specific to discovery of tool
definitions. A real list/read must not disappear from operation accounting.

Admission currently checks the initial conversation before any live read
(src/policy.rs:296; src/cli.rs:534). Keep that stage free of filesystem I/O.
Reserve the explicit small grounding allowance or recheck actual framed bytes
after the observation, before model dispatch; specify stop/eviction behavior
instead of silently enlarging limits. `drop_oldest_compactable`
(src/core.rs:387) protects initial user messages before the first assistant:
inserting source text there makes it uncompactable. This is another reason to
start with a small listing, not multiple file bodies.

Metadata-only capture must retain descriptors/digests without the directory
body or filenames smuggled into a new unvalidated metadata field. Private replay
capture may retain the actual bounded result. Follow the existing
`storage::validate_capture` / `sensitive_metadata` boundary (src/storage.rs:1079)
and explicit replay payload shape. Current model dispatch is distinct from
durable capture consent; a grant does not make arbitrary extra context useful.

## Source attachments and decision risks

If a later experiment needs source bytes, use an explicit trusted relative-path
selection with an effective `read_file` grant and the same real execution,
provenance, bounds and replay rules. Do not pick every `.js` file, open guessed
`app.js`, recurse through packages, or silently include hidden/config files.
Those policies introduce relevance and disclosure choices, can bias the fixture,
and may consume the budget before useful work. A model-selected subsequent read
already has a correctly owned and recorded path today.

A directory observation may still be ignored, may omit a relevant nested file,
or may increase answer bias/context pressure. Successful named-file tasks do
not prove listing will cure repair initiation. Automatic source attachment may
help code reasoning while concealing continued discovery failure. Treat such a
result as assisted grounding, not autonomous discovery or storefront acceptance.
The next comparison should change one grounding input while freezing the repair
task, model, protocol, permissions, artifacts and budgets. No additional call
is authorized by this research note.
