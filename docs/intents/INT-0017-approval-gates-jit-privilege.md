# INT-0017 — Human-approval gates & just-in-time privilege

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0017
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Add an operator-configurable **human-approval gate** for irreversible or
high-impact effects (a command, a write/delete/move, a network-bearing tool),
and **just-in-time privilege**: a run holds only the authority its current step
needs, elevated per-action rather than for the whole run. A gated effect pauses
with a described, bounded approval request and executes only on an explicit,
per-action approval that cannot be granted by model output or observed content.
Non-goals: replacing the capability model or the checked-run bar (this is an
additional gate above them); an interactive TUI (the mechanism is a
pause/approve/deny protocol usable by a human or an outer trusted controller);
approvals sourced from untrusted content.

## Acceptance criteria
- An effect whose tool/impact class is configured as gated does not execute until
  an explicit approval arrives; a denial ends the effect with a defined,
  journalled outcome and never a partial application.
- Approval is per-action and per-run; one approval never generalizes to later
  actions, and no approval can be derived from model output, tool results, or
  workspace content.
- Just-in-time privilege: a step cannot exercise authority beyond its declared
  need, proven by a test that a gated tool is unavailable until elevated.
- Gate decisions (requested, approved, denied, timed-out) are journalled and
  visible to `inspect`, without disclosing prompts.

## Rationale
Human oversight for high-impact actions and just-in-time/least-privilege are core
NIST AI-agent controls and the primary OWASP mitigation for "excessive agency."
The runtime already scopes capability and bars mutating tools in checked runs, but
has no per-action human-in-the-loop gate or step-scoped privilege — the missing
control for high-consequence autonomous operation.

## Alternatives
All-or-nothing per-run grants (current: no mid-run gate; an approved run can do
anything its grant allows). External wrapper approval (loses journalling and the
frozen-input guarantee). Always-manual (defeats autonomy; the gate must be
selective by impact class).

## Consequences
A pause/resume protocol threaded through the runner and journal; latency for gated
actions; a timeout/default-deny policy; interaction with replay (an approval is
recorded input, not a pure decision — journalled like a control observation) and
with INT-0010 (a paused holder must not deadlock coordination).

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme A — security hardening).
