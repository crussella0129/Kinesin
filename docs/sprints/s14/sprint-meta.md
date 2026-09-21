# Sprint 14 Meta

- **Sprint number:** 14
- **Book schema version:** 2
- **Start timestamp:** 2026-09-20T05:46:52Z
- **End timestamp:** 2026-09-20T13:40:28Z
- **Model:** gpt-6-astra
- **Bundle version:** 0.22.0
- **Exit status:** failed
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Failed low-intervention workflow experiment; preserve eight failures and root-cause research handoff.
- **Intents:** [INT-0032](../../intents/INT-0032-low-intervention-local-workflows.md)
- **Completion evidence:** Eight live attempts failed INT-0032 AC3; findings and unverified source archived in T-118, unfinished T-115-T-117 returned to backlog, official verification deferred; see docs/sprints/s14/failure-report.md.
- **Checkpoint:** https://github.com/crussella0129/Kinesin/pull/14

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
