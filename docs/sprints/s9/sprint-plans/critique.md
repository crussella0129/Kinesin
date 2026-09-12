# Plan Critique — Sprint 9

## Concerns

### C-001: "Unapproved call denied" does not assert the server is never spawned
- **Where:** `build-plan.md` T-007 EARS 2 / `test-plan.md` Intent Traceability row 2 → `mcp_unapproved_call_denied`
- **Quote:** "WHEN a run calls a server/tool not approved, THEN the call SHALL be denied **without spawning/contacting the server**."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the denial test proves `ToolStatus::Denied`, but the security-relevant half — that no child process is spawned and no bytes are sent to a server — is the stronger claim the intent's two-gate boundary rests on. Without an explicit no-spawn assertion the clause is only half-verified.
- **Suggested response:** fix-in-plan — strengthen the `mcp_unapproved_call_denied` entry to also assert no MCP child is spawned / no `tools/call` is issued for the denied call.

### C-002: Lightweight arg validation may accept args a strict server rejects
- **Where:** `build-plan.md` T-005
- **Quote:** "args fail a **lightweight required-fields/object check** derived from `input_schema` → `invalid_arguments`."
- **Failure mode:** intent-drift (potential over-claim on "same … gates as a compiled tool", since compiled tools use the strict `TypedToolArgs` `deny_unknown_fields`).
- **Why it matters:** an MCP call with extra/mistyped fields could pass the gate and reach the server, unlike a compiled tool. This is weaker validation than the compiled path.
- **Suggested response:** defer-with-rationale — the gate's security property (a tool the run was not granted cannot run) is independent of argument strictness; the server is the authority on its own schema and rejects bad args; full JSON-Schema validation is a named follow-up in the intent's out-of-scope. The MVP requires the top-level object shape + required keys, which is enough to route a well-formed call. Recorded, not fixed this sprint.

### C-003: Discovery at run start is unbounded I/O — a bad server could hang the run
- **Where:** `build-plan.md` T-003
- **Quote:** "the runner **SHALL** connect to its declared server, discover the tool's `{description, input_schema}` … before admission."
- **Failure mode:** missing-risk
- **Why it matters:** spawning a child + `initialize` + `tools/list` can block indefinitely if the server misbehaves, hanging run start with no bound — a denial-of-service / operability gap inconsistent with the harness's bounded-everywhere posture.
- **Suggested response:** fix-in-plan — bound discovery (connect + initialize + list) with a timeout; on timeout or transport error, fail run start with the same defined error as the missing-tool case. Fold into T-003's third acceptance-relevant clause.

## Confidence
proceed-with-caveats

C-001 and C-003 are fixed in the plans below before finalization; C-002 is
accepted as a documented, deferred scope decision (full JSON-Schema arg
validation is out of scope for the MVP and named in INT-0005 out-of-scope).
