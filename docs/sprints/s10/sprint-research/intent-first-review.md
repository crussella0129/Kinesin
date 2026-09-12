# Sprint 10 — intent-only baseline

Recorded before opening implementation code or tests on 2026-09-12. Inputs:
all twenty chapters under `docs/intents/`, their index, and `docs/roadmap.md`.
The findings below are requirements questions, not implementation conclusions.

## Coverage and omissions

The twenty intents cover token reporting (0001), bounded compaction (0002),
commands (0003), prompt caching (0004), local MCP (0005), skills (0006), model
supervision (0007), model transport (0008), overlay development (0009), shared
writes (0010), roadmap governance (0011), Linux isolation (0012), dependencies
(0013), signed history (0014), assurance (0015), telemetry (0016), approvals
(0017), subagents (0018), Windows isolation (0019), and remote MCP (0020).

Missing explicit ownership of foundational harness outcomes:

- Durable run lifecycle and recovery: crash/restart/cancellation, effect
  reconciliation, explicit terminal outcomes, and replay compatibility.
- Session/context continuity: bounded provenance-preserving carry-forward,
  user interruption and branching, and model token-window admission. Byte
  compaction and cache reuse do not specify semantic continuity or token fit.
- Tool interoperability and lifecycle: schema/dispatch agreement, bounded
  discovery, cancellation, process cleanup, schema drift, and honest failure
  outcomes. Extend existing command/MCP contracts where appropriate.
- Model capability contracts and reproducible evaluation: supported response
  formats, malformed/partial responses, task quality and regression workloads,
  replay versus actual model-quality evidence. Caching is only one performance
  dimension; security tests do not establish task competence.
- Private durable data lifecycle: credential isolation, retention, deletion,
  backup/restore, and upgrade compatibility. Signed history is distinct from
  confidentiality and recoverability.
- Workspace change correctness: atomic or explicitly reconciled mutations and
  stale-input conflict detection. Presence leases address concurrency, not every
  mutation's failure semantics.

These are candidate intents for audit and planning, not permission to implement
all proposed roadmap features in this repair sprint. Existing mechanisms may
already satisfy some of them; the second pass will establish that.

## Contract inconsistencies to audit

- INT-0008 requires encrypted traffic between different machines and never a
  public non-overlay backend. Its realization instead allows private plaintext,
  public HTTPS by opt-in, and cites the same server over loopback and its own LAN
  address. Address privacy alone is not encryption; that test is not a two-host
  encrypted-transport proof. Preserve the original requirement and expose the gap.
- INT-0004 promises session cache reuse and honest concurrency/lifetime handling;
  its evidence discusses within-run prefixes and a stable system prefix. Verify
  precisely which session/concurrency claims are actually established.
- INT-0002 promises continued bounded sessions with immutable checked evidence;
  audit whether dropping oldest can preserve progress, complete tool groups, and
  accounting at boundaries rather than merely having a compaction counter.
- INT-0005 promises equivalent byte/time/concurrency gates for dynamic tools;
  discovery, server descriptions, failure cleanup, and checked-run effects need
  explicit examination as well as successful calls.
- INT-0012 promises no path into private state and mandatory isolation; inspect
  broad system grants, overlapping roots, descriptor inheritance, process cleanup,
  and unavailable-kernel behavior.
- INT-0011 and INT-0015 describe maintained artifacts but terminal chapters forbid
  rewriting realized intent. Use follow-on revisions with retained provenance.
- The roadmap's opening snapshot and sequencing lag its own later realized list.
- INT-0020 inherits INT-0008's private-plaintext policy despite handling credentials;
  its proposed acceptance must reflect verified confidentiality, not address class.

## Review order

1. Finish an independent intent-only review and freeze these candidate gaps.
2. Inspect implementation and tests against every original intent, including
   proposed work; record satisfied, partial, absent, and unverified criteria.
3. Create follow-on repair/revision intents for terminal chapters and explicit
   proposed chapters for missing outcomes; avoid silently lowering acceptance.
4. Plan repairs, execute them, test, verify, and submit the sprint checkpoint PR.
