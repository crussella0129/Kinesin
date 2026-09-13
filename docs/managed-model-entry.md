# Local model entry follow-up

This direct follow-up T-109 extends PR12 before merge, following the user's
report that interactive entry still exposed a personal remote deployment and
required URL/model-ID knowledge. It does not reopen the completed sprint11.

## Implementation plan

1. Keep mandatory working-folder selection. Discover bounded regular GGUF files
   in per-user model storage and launch-folder `model`/`models` directories;
   offer an arrow/Enter selector, saved selection and custom file choice.
2. Resolve a trusted local llama-server executable outside the working folder.
   Treat GGUF and runtime parent directories as protected runtime resources.
   Persist model changes atomically with a private backup and preserve unrelated
   operator settings. Path overrides apply to human session setup only.
3. Start one owned loopback backend with typed arguments and a unique served
   identity. Gate resource preparation/admission on health, identity, context
   and slot readiness. Bound startup and diagnostic retention; stop admission
   and settle work on child death. Reap the owned tree on all exit paths.
4. Preserve explicitly selected external HTTP(S) serving. SSH authentication
   remains in a foreground SSH terminal and is never collected by the model.
5. Remove personal machine/account/model-alias defaults from active usage
   examples. Normalize sensitive environment labels in prior public evidence
   without changing historical IDs, hashes, timestamps or outcomes.

## Validation plan

- Configuration tests: missing/invalid files, bounds, disjoint model/runtime
  parents, unsupported service scope and profile mismatch.
- Selector/onboarding tests: bounded discovery, canonical deduplication,
  trusted runtime resolution, cancellation, preserved settings and backups.
- Native process tests: delayed readiness, never-ready and identity/context
  mismatch, startup cancellation, child death, bounded logs and descendant
  cleanup, including CLI ownership rather than only isolated primitives.
- Windows and native Debian terminal walkthroughs: arrow/Enter selection,
  startup, actual model tool use, exit and Ctrl+C cleanup. Record actual source
  and artifact identities; distinguish real model evidence from fixture tests.
- Format, all-target/all-feature Clippy, relevant/full tests, dependency gates,
  independent review, Book/link checks and final PR12 CI. Do not merge.

## Results

Implementation and validation are in progress. No new pass is claimed here yet.
