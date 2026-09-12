# Sprint 11 Installation and Setup Tests

Intent: [INT-0028](../../../intents/INT-0028-first-use-documentation.md).
Implementation: `c7eb088`; source provenance and execution boundaries are in
[platform-verification.md](platform-verification.md).

| Named verification | EARS | Executed assertion and result |
| --- | --- | --- |
| product_only_install | T-001/A,C; T-002/A | On both hosts, `cargo install --locked --debug --path . --bin kinesin --root TASK_ROOT/install` succeeded. Each bin directory contained only kinesin (kinesin.exe on Windows). Installed help succeeded from outside the source checkout. Windows process-local PATH resolved the task installation from the user's home; no persistent PATH was changed. Pass. |
| repeat_setup_preserves_existing_files | T-001/B | Executed each guide setup block twice in isolated directories. Windows config/project sentinel hashes and existing state SDDL remained unchanged. Linux new directories/files were 700/600; sentinel contents and existing 750/755 directory and 640/644 file modes remained unchanged, as did the caller's umask. Pass. |
| usage_walkthrough_windows | T-001/B; T-002/A | Starter-relative workspace/state, installed entry, single prompt, checked task, inspect/export/replay and bare session-to-EOF all succeeded using the SSH-forwarded Nighthawk model. Pass. |
| usage_walkthrough_linux | T-001/B; T-002/A | Same installed entry and walkthrough on Debian nighthawk, outside the source checkout with absolute config. Only model port changed 8080 to 18080. Pass. |
| starter_config_and_checked_replay | T-001/B | Both native runs accepted project=Kinesin and language=Rust with evidence. Replay capture was selected before execution; inspect and new-file export succeeded. Replay was consistent with 11 events, 3 model requests, 1 tool observation and verification_replayed=true, including after server shutdown. Pass. |

The debug installation option is documented and keeps this verification bounded
by reusing compilation output. Default release-profile installation and fresh
OS prerequisite provisioning were not executed; this distinction also appears
in the platform record. No global installation or existing operator config was replaced.
