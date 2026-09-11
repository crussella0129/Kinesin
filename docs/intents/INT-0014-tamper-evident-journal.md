# INT-0014 — Tamper-evident audit journal

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0014
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent
Make the SQLite journal tamper-evident so that offline modification of the
database, or a compromised controller, cannot silently forge or alter a run's
recorded history or acceptance verdict. Hash-chain events per run (each event
commits to the previous event's digest) and sign the terminal receipt with a
deployment key, so verification (and replay) can detect any insertion, deletion,
reordering, or edit after the fact. Non-goals: preventing a live compromised
controller from writing new malicious-but-consistent history (append-time trust
is a separate problem); external timestamping/transparency-log integration
(a later enhancement); encrypting the journal at rest (orthogonal).

## Acceptance criteria
- Each run's events form a verifiable hash chain; a verifier detects any inserted,
  deleted, reordered, or modified event and names the break.
- The terminal receipt carries a signature over the run's final digest; a forged
  or altered receipt fails signature verification.
- Replay and `inspect` surface a tamper-check result; a clean run verifies and a
  deliberately mutated fixture is reported as broken (proven by tests).
- The scheme adds no prompt/intermediate-text disclosure and preserves the
  existing owner-scoping and metadata-only capture guarantees.

## Rationale
adversarial-review.md states plainly that "digests are identity checks, not
signed attestations" and that a modified database or compromised controller can
invalidate a verdict — the largest gap between the current immutable-by-convention
journal and an NSA-grade, audit-defensible record. Hash-chaining + receipt signing
is the standard, well-understood mechanism.

## Alternatives
Rely on filesystem permissions + the immutable-by-code convention (current:
detects nothing after an offline edit). A full external transparency log / Merkle
inclusion proofs (stronger, heavier; revisit for a shared multi-tenant service).
WORM storage (operational, not portable).

## Consequences
A per-event digest column and chain-verification path; deployment key management
for receipt signing (ties into INT-0013's signing infrastructure); replay/inspect
gain a verification step; a schema migration for existing journals.

## Transition history
- 2026-09-11: created as `proposed` (sprint 5 roadmap, theme A — security hardening); closes the adversarial-review "signed attestations" gap.
