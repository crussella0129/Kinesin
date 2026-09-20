# Test Critique — Sprint 13

## Concerns

(none — the scoped intent acceptance criteria and every EARS clause have sufficient evidence.)

## Review basis

Independent read-only source/assertion review covered implementation commit `d9547f8865b0703919f3ba3aea1f75abd7432932`, the locked build/test plans, INT-0031, T-112/T-113 completion entries and the [unit](unit-tests.md), [integration](integration-tests.md) and [live E2E](e2e-tests.md) records. The reviewer did not implement product changes or rerun tests. Reported command outcomes came from the implementation operator; journal evidence was independently read and retained. All eleven recorded source/test hashes match the implementation commit.

- AC1 and T-112's first clause have an initially empty, separate application workspace, private control state, recorded model/limits/grants and actual prompts. The evidence explicitly avoids claiming OS isolation.
- AC2 and T-112's second clause have real file-tool observations and hash-identified generated artifacts. Browser operations establish the catalog/cart/checkout result; fabricated model answers remain recorded failures, and precise operator-guided repairs are disclosed.
- AC3 and all five T-113 clauses map to named executed checks and actual browser/lifecycle observations. The real HTTP tests assert response bytes and denied outside bytes, not only status codes. The Windows native symlink fixture actually executed; no `UNAVAILABLE` branch was reported. Service grants and checked authority are rejected, and an ungranted runner call is denied.
- AC4 and T-114 preserve the live-first order, concrete failure/repair evidence, 271 distinct focused passing cases, format/Clippy results and final source identity. Replay tests verify that the new effect is not re-executed and that older denial semantics remain stable.
- Scripted model clients appear only in bounded runner/replay integration checks; the real local model authored and revised the independently operated workload. Loopback requests and temporary fixtures have finite watchdogs and isolated paths.

This acceptance is for one operator-steered static storefront on native Windows. It does not certify unattended model reliability, production storefront quality, Linux behavior, remote CI, arbitrary backend servers or a full unrelated test suite. These limits are disclosed and do not leave a promise in this sprint's scoped intent unproved.

## Confidence

clean
