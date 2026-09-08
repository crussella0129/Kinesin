# Synthetic live task observations

These files retain every sample from the 2026-09-08 read-only task evaluation.
See [the interpretation and reproduction commands](../../../docs/live-evaluation.md).
They are measured outputs, not golden answers to copy into prompts or training.

- `prompt-probes.json`: nine selected-scope CLI diagnostics, including unsuccessful
  checked instructions, all four freeform prompt candidates, the first checked
  pass, and the two initial explicit-action capability probes.
- Sixteen baseline card files: two samples each, from one frozen instruction
  generator and the pinned llama.cpp b6500/Qwen2.5-Coder-7B profile.
- `recovery-explicit-actions.json` and `comparison-explicit-actions.json`: two
  later samples each after explicit action wording was selected. These are a
  separate diagnostic batch; they do not replace baseline failures.
- `operator-guidance-interrupted.json`: eight completed samples plus the
  interruption record from a later general-instruction probe. Severe model
  slowdown prevented reaching the original recovery/comparison tasks; this
  profile was not promoted to the example configuration.

Each card records synthetic source bytes and hashes, the actual prompt, a rubric,
exact final candidate, separate execution/acceptance outcomes, receipt bindings,
model/tool observations, prepared-request hashes/byte counts, and measured time.
The manual rubric review is an evaluation annotation, never runtime authority.
Configuration filesystem paths and private authority snapshots were omitted.
No model weights, credentials, or unrelated workspace content are included.
Every JSON file is below 64 KiB. Full SQLite journals and generated workspaces
remain under ignored `validation-output/`.

The independently scripted cases in `tests/live_evaluation.rs`,
`tests/runner_tools.rs`, and `src/verification.rs` establish deterministic
behavior. Ordinary test runs do not require or contact a live model.
