# Sprint 11 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | EARS | Named verification |
| --- | --- | --- | --- |
| INT-0028 | Product selection and side-effect-free help | T-001/A | default_product_entry; help_without_configuration; product_only_install |
| INT-0028 | Windows/Linux README and detailed first use | T-001/B | usage_walkthrough_windows; usage_walkthrough_linux; starter_config_and_checked_replay; repeat_setup_preserves_existing_files |
| INT-0028 | Fixture purpose and install selection | T-001/C | fixture_explanation_review; product_only_install |
| INT-0028 | Actual platform and Nighthawk evidence | T-002/A | platform_smoke_evidence |
| INT-0028 | Current references and closed checkpoint | T-002/B | usage_reference_review; book_and_test_critic |

## Unit Tests
- T-001/A: `help_without_configuration` launches the real CLI with --help/-h
  from an empty directory and asserts successful usage output without files.
  Malformed extra arguments remain errors.
- T-001/A: `default_product_entry` invokes cargo run -- --help and verifies
  successful product usage rather than a fixture or Cargo ambiguity.
- T-001/C: `fixture_explanation_review` checks README/guide against Cargo targets
  and the integration tests' CARGO_BIN_EXE usage; this is a named review procedure.

## Integration Tests
- T-001/A,C; T-002/A: `product_only_install` uses cargo install --path . --locked
  --bin kinesin with a task-owned --root and existing build cache where possible;
  verify the bin directory and run the installed executable from outside the repo.
- T-001/B; T-002/A: Windows and Linux walkthroughs validate starter config paths,
  actual executable/session/single/checked commands and result semantics.
- T-001/B: `starter_config_and_checked_replay` uses --capture replay on the
  checked run, then inspect/export/replay; no model is contacted by offline replay.
- T-001/B: `repeat_setup_preserves_existing_files` reruns each OS setup block
  with sentinel configuration/workspace contents and existing directory modes/ACLs;
  assert they are unchanged. Apply private permissions only when creating state.

## End-to-End Tests
- **Status:** possible. A retained pinned server/model on nighthawk supplies the
  actual model path. Record greeting, session and checked results under each
  platform used; do not require a particular model-generated greeting string.
- Toolchain installation and prerequisites are separate from Kinesin behavior.
  Any unexecuted OS provisioning step remains explicit, with its primary source.
- Format, all-target/all-feature clippy and affected `tests/cli_inspect.rs` run
  before the implementation commit. Existing CI supplies final submitted-head
  Windows/Ubuntu/dependency checks. Do not rerun unrelated live/load suites.
- T-002/B: independent critic, valid links and committed Book before closing;
  reuse PR #11 under the user's explicit checkpoint instruction.
