# Test Critique — Sprint 4

Adversarial read-only screen of the sprint 4 test evidence against the locked
plans and INT-0008.

## Concerns

### C-001: the uniform-attach test uses a scripted client
- **Where:** `integration-tests.md` / `uniform_attach_prepares_identically_across_local_and_overlay_backends`.
- **Quote:** "runs the same scripted checked run against a loopback `base_url` and a CGNAT-overlay `base_url`."
- **Failure mode:** weak-assertion.
- **Why it matters:** a scripted client never opens the socket, so the test proves the address does not perturb request preparation — not that a real server at an overlay address behaves.
- **Suggested response:** accept as layered. Offline can only prove address-independence of the prepared request (location transparency) and the admissibility of the overlay origin (T-001). "A real server at that address answers the same" is exactly what the `#[ignore]`d live `attach_to_non_loopback_backend` records; the split mirrors INT-0004's accepted layering.

### C-002: `unreachable_backend_reports_not_ready` binds-then-drops a port
- **Where:** `integration-tests.md` / `unreachable_backend_reports_not_ready`.
- **Failure mode:** flake-risk.
- **Why it matters:** between dropping the listener and `ready()` connecting, another process could theoretically claim that ephemeral port.
- **Suggested response:** defer-with-rationale. The window is microseconds on loopback and any squatter would have to speak the readiness protocol to flip the assertion — effectively impossible; the 5 s timeout bounds the failure to a `false`, never a hang. This is the standard "closed port" idiom the repo already uses for transport tests.

### C-003: the live attach test depends on an external server and a network interface
- **Where:** `e2e-tests.md` / `attach_to_non_loopback_backend`.
- **Failure mode:** e2e-cop-out (screened, not upheld).
- **Why it matters:** the test asserts a non-loopback interface exists and a server answers at it.
- **Suggested response:** defer-with-rationale. It is `#[ignore]`d and manual, exactly the INT-0004 pattern; the CI-verifiable code path is covered offline. Marked `possible`, named, and compiled — not skipped.

## Screen of the remaining failure modes
- **Intent/EARS trace gap:** none — every INT-0008 acceptance criterion maps through the test-plan traceability table to a named executed test (or the named live test for the measured-attach criterion).
- **Stub leakage:** none — the scripted `ModelClient` is the repo's standard double; it does not mirror `validate_origin`, which is exercised directly.
- **Integration drift:** none — `uniform_attach…` runs the full admit→run→capture path (not a unit repeat); `unreachable…` drives the real `ModelClient::http`.
- **Negative-path absence:** none — reject-public (two tests) and unreachable-backend are executed negative tests.
- **Evidence drift:** none — all result records name tested head `a8de180`, carry canonical confirmations, and identify the Test-evidence link the report attaches to INT-0008.

## Confidence
proceed-with-caveats
