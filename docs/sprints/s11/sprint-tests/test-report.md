# Sprint 11 Test Report

## Intent Verification

| INT-0028 acceptance criterion | EARS | Named executed verification | Result |
| --- | --- | --- | --- |
| Product default/help and selected installation | T-001/A | default_product_entry; help_without_configuration; product_only_install | Pass on Windows and native Debian; malformed help combinations reject without files |
| Prominent OS-specific checkout/install/PATH/config entry | T-001/B; T-002/A | usage_walkthrough_windows; usage_walkthrough_linux; platform_smoke_evidence | Pass with installed help outside the checkout and actual first model/session commands |
| Private starter/model/checked/replay setup and expected outcomes | T-001/B | repeat_setup_preserves_existing_files; starter_config_and_checked_replay | Pass, including PowerShell 5.1, unchanged sentinel contents/permissions and disconnected replay |
| Honest fixture purpose and exclusion from product install | T-001/C | fixture_explanation_review; product_only_install | Pass; only kinesin installed, native process/stdio fixtures retained for tests |
| Current references, real platform evidence and Book checkpoint | T-002/A,B | usage_reference_review; platform_smoke_evidence; book_and_test_critic | Pass for TEST: actual provisioning boundaries, valid 28-intent Book, resolved links and independent clean critique; Loop owns realization and final checkpoint |

The [unit record](unit-tests.md), [integration record](integration-tests.md),
[E2E record](e2e-tests.md) and [platform provenance](platform-verification.md)
contain the assertions, commands, per-suite confirmations, source hashes and run
IDs. The final [independent critique](critique.md) is clean. This report is attached
as Test evidence to [INT-0028](../../../intents/INT-0028-first-use-documentation.md)
while it remains active; Loop reconciles its completion and realization evidence.

## Canonical CI

- **Tested implementation:** `c7eb088ad104d381853ede5d2288c8d97e663aa2`.
- **Tested head:** `ee6a26b8c2b662b77511c1f25bfa01a57ea62cd5` (ledger successor).
- **PR CI:** [34710282235](https://github.com/crussella0129/Kinesin/actions/runs/34710282235), success.
- **Branch CI:** [34710279767](https://github.com/crussella0129/Kinesin/actions/runs/34710279767), success.
- **Confirmations:** Windows 304 passed (164 library + 140 integration);
  Ubuntu 316 passed (168 library + 148 integration); zero failures, nine ignored
  per platform. Both format/all-target/all-feature clippy gates and deny/audit pass.
- **Local affected checks:** CLI unit 8 and CLI integration 11 passed, format,
  complete clippy and diff checks passed before implementation commit.

Only sprint verification/ledger documentation differs after the tested head;
no runtime, starter, Cargo, test or usage-document changes followed it. Loop checks
the submitted PR head separately instead of relabeling this CI result.

## Actual OS Walkthroughs

Both hosts installed only the product into a task-owned prefix using the
documented debug option. Setup repetition preserved operator-like sentinel
contents and permissions. Six actual model runs passed their expected completion
or checked-acceptance conditions. Both checked exports replayed consistently
after the verified model process and SSH forward were stopped. Native Nighthawk
Rust tooling stayed within an isolated validation directory.

Actual Windows PowerShell 5.1.26100.9444 also passed setup, repeated preservation
and installed help from outside the checkout. The full Windows model walkthrough
used PowerShell 7. These are separate claims, not a full clean-machine installer test.

## Limitations and Disposition

Fresh system prerequisite installation, interactive/default-home rustup setup,
persistent PATH configuration and release-profile installation were not executed.
The guide provides those steps with primary references; the native tests reused
available prerequisites and debug artifacts. Initial network-sandbox, elevated
private-capture and PowerShell -File-policy failures remain in the platform record;
documented commands passed within their intended authorized execution contexts.

Two documentation-review findings (health-port consistency and unique temporary
rustup download) were fixed and rechecked. No source contract remained failing,
and this bounded usage sprint adds no new backlog. Existing seven broader
capability tasks remain unchanged. By explicit user instruction, Loop adds this
sprint to existing PR #11 and leaves merging for review.
