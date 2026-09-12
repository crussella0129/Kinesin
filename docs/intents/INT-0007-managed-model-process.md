# INT-0007 — Managed local model-process supervision (Kineserve)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0007
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

> **Roadmap:** theme C (operability) — see [the roadmap](../roadmap.md) (INT-0011).

## Intent
Optionally start and supervise the local `llama-server` process instead of
requiring an operator to launch it by hand (today's attach-only profile). When
the operator selects a managed model, Kinesin spawns `llama-server` with the
configured model file and flags, gates run admission on `/health` readiness,
and owns the child's lifecycle: whole-process-tree cleanup on shutdown, and a
defined outcome when the child dies or never becomes ready. This is the
"Kineserve" responsibility named in the README, currently unimplemented.
Non-goal: managing a remote server's process (that host owns its own process);
downloading or building models; supervising more than the configured local
backends.

## Acceptance criteria
- With a managed model configured, `kinesin` starts `llama-server`, waits for
  `/health`, and admits runs only once it is ready; a server that never becomes
  ready fails admission with a defined error rather than hanging.
- On shutdown (including Ctrl+C) the whole `llama-server` process tree is
  reaped, using the same tree-kill machinery INT-0003 added; no orphan survives.
- An externally started server on the configured endpoint is never stopped;
  attach mode remains available and is the default for a remote endpoint.
- Tests cover ready-gating, a never-ready failure, child death mid-run, and
  clean tree teardown.

## Rationale
Serving locally is the common case, and the manual launch documented in
`docs/model-preflight.md` is a real papercut (the KV-cache benchmark had to
hand-launch the pinned server). Managed supervision closes the "serve locally"
half of the model-serving architecture goal without weakening the attach path.

## Alternatives
Keep attach-only (current; smallest surface, but manual). A shell/PowerShell
launcher script outside the binary (the original scaffold's `run-harness.ps1`;
rejected because process ownership and cleanup then live outside Rust). Relates
to INT-0008, which owns the remote-endpoint half.

## Consequences
Reintroduces `Command`/`Child`, readiness, and owned-cleanup ownership the
attach profile deliberately deferred; a new failure surface (spawn failure,
crash, restart policy) that must map to defined run outcomes; interacts with the
scheduler's admission gate. Reuse the shared `OwnedProcess` group/Windows Job
lifecycle implementation introduced in sprint 10, extending it only where
managed model ownership requires separately tested behavior.

## Transition history
- 2026-09-11: created as `proposed`.
- 2026-09-12: proposed implementation guidance updated to the shared owned-process primitive after the unused command-group dependency was removed; capability remains proposed.
