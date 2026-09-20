# Sprint 14 Meta

- **Sprint number:** 14
- **Book schema version:** 2
- **Start timestamp:** 2026-09-20T05:46:52Z
- **End timestamp:** (filled at Loop Phase)
- **Model:** gpt-6-astra
- **Bundle version:** 0.22.0
- **Exit status:** in-progress
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Failed low-intervention workflow experiment; preserve eight failures and root-cause research handoff.
- **Intents:** [INT-0032](../../intents/INT-0032-low-intervention-local-workflows.md)
- **Completion evidence:** (filled at Loop Phase)

## Unresolved live gate

All eight retained fresh attempts failed the independent usability gate. The
structured-action experiment regressed to zero effects and was removed from
new-run defaults; its captures remain supported for replay. See the
[outcome scorecard](sprint-tests/e2e-tests.md) and
[recorded implementation decisions](sprint-research/implementation-adjustment.md).
T-115–T-117 remain queued. Official unit/integration verification is deferred
until useful live behavior is demonstrated. The next design work must address
actual observations driving bounded repair and completion decisions; another
passing compile or model completion claim cannot close this sprint.
