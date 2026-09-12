# INT-0029 — An interactive entry point for real workspace work

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0029
- **State:** active
- **Work evidence:** [T-108 repair plan](../interactive-entry-repair.md#t-108-interactive-entry-and-workspace-operations)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
An operator launches `kinesin` from any directory, receives an introduction,
chooses the working folder and interacts with a useful workspace assistant.
Model and capability choices are visible, answers and tool progress are readable,
and explicit automation keeps structured output. This follows the requested
Codex CLI style of entry and conversation without claiming parity with its model,
full conversation memory, terminal editor or broader tool ecosystem.

## Acceptance criteria
- A product-only installed command resolves outside its source checkout. Bare
  terminal startup introduces the program and selects an existing working folder;
  current directory is a visible default rather than a hidden demo workspace.
- Per-user configuration persists outside the selected workspace. First use
  configures the model and supported file capabilities; existing settings/files
  are preserved. Explicit configuration and noninteractive automation retain
  their authority and do not receive unexpected setup questions.
- The granted directory tool creates a single named directory under an existing
  workspace parent, including spaces, without shell execution or replacement.
  Missing grants, traversal, symlink parents and outside destinations fail without
  the denied effect. Checked runs cannot acquire mutation authority.
- Human sessions show the selected folder/model, useful tool progress, answers
  and actionable errors. Ordinary replies do not dump machine receipts or portray
  absent independent acceptance checks as failed operations. Structured output
  remains explicitly available; terminal control characters stay escaped. Ctrl+C
  exits an idle session promptly and settles an active request before shutdown.
- Actual interactive create-folder and list-files requests produce and report
  the expected filesystem result in the chosen folder. Windows/Linux checks,
  authorization/replay regressions, formatting, linting and independent review
  support the implementation; setup and model limitations remain explicit.

## Rationale
The user reached a prompt but received a refusal for folder creation, a listing
of an unexplained example folder and raw result JSON. The earlier hello test
proved connectivity, not the intended working experience. INT-0028's documented
example workflow remains historical evidence; this follow-on owns normal entry.

## Alternatives
Keep explaining the demo configuration (does not supply the requested product
entry); enable unrestricted shell commands (unnecessary for directory creation);
build a full-screen TUI first (does not itself repair configuration or authority).

## Consequences
Installation and persistent settings become part of normal operation. Model
serving remains separate. Folder selection grants only the displayed configured
capabilities, and private runtime inputs remain outside that folder. Full-history
continuity stays with INT-0026; current follow-ups cite the preceding answer.

## Transition history
- 2026-09-12: created as `proposed` after the user's actual first-use failures.
- 2026-09-12: `proposed → planned`; the user explicitly requested selection upon entry, a command callable anywhere and a Codex CLI-like experience.
- 2026-09-12: `planned → active`; T-108 implements the global entry, human session and bounded workspace operation repair as a direct follow-up to PR #11.
