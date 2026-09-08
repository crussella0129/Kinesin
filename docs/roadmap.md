# Implementation roadmap

Implementation is being validated against this guide. Observed proofs and
remaining gaps are recorded in [build validation](build-validation.md); a passing
test or one completed model run does not establish every release requirement.
The preceding documentation review is tracked in [research](research.md).

Use the [build guide](build-guide.md) for the actual instructions,
references, and proofs. These numbers match that guide exactly. Check a box
only after its proof and failure exercise pass.

## Checkpoint A: a recorded live turn

- [x] 1. [Write the two operating profiles](build-guide.md#1-write-the-two-operating-profiles).
- [x] 2. [Set up Rust and an ownership scratch project](build-guide.md#2-set-up-rust-and-an-ownership-scratch-project).
- [x] 3. [Create one package and a repeatable checkpoint](build-guide.md#3-create-one-package-and-a-repeatable-checkpoint).
- [x] 4. [Prove the model contract independently](build-guide.md#4-prove-the-model-contract-independently).
- [x] 5. [Define owned domain types](build-guide.md#5-define-owned-domain-types).
- [x] 6. [Make state transitions without I/O](build-guide.md#6-make-state-transitions-without-io).
- [x] 7. [Learn enough async to run the shell](build-guide.md#7-learn-enough-async-to-run-the-shell).
- [x] 8. [Run a scripted model through one runner](build-guide.md#8-run-a-scripted-model-through-one-runner).
- [x] 9. [Validate configuration and construct run authority](build-guide.md#9-validate-configuration-and-construct-run-authority).
- [x] 10. [Persist a run through one SQLite owner](build-guide.md#10-persist-a-run-through-one-sqlite-owner).
- [x] 11. [Map the wire format with saved fixtures](build-guide.md#11-map-the-wire-format-with-saved-fixtures).
- [x] 12. [Add the bounded HTTP adapter](build-guide.md#12-add-the-bounded-http-adapter).
- [x] 13. [Complete the first live command](build-guide.md#13-complete-the-first-live-command).
- [x] 14. [Enforce stopping and cancellation around effects](build-guide.md#14-enforce-stopping-and-cancellation-around-effects).

**Gate:** the scripted path and one live text exchange work. Invalid config,
transport failure, oversized input/output, truncated generation, journal
failure, and cancellation produce the defined outcomes. The journal intent
commits before a model request is sent.
Freeform completion is explicitly unchecked. The demonstration's
`--allow-unchecked` affects only its exit policy; no task is labelled passed.

## Checkpoint B: a useful read-only agent

- [x] 15. [Define the three tool contracts](build-guide.md#15-define-the-three-tool-contracts).
- [x] 16. [Create the workspace capability](build-guide.md#16-create-the-workspace-capability).
- [x] 17. [Implement a bounded file read](build-guide.md#17-implement-a-bounded-file-read).
- [x] 18. [Implement a bounded directory listing](build-guide.md#18-implement-a-bounded-directory-listing).
- [x] 19. [Complete one model/tool/model round trip](build-guide.md#19-complete-one-modeltoolmodel-round-trip).
- [x] 20. [Prove policy, protocol, and loop failure paths](build-guide.md#20-prove-policy-protocol-and-loop-failure-paths).
- [x] 21. [Check file fields before accepting a task](build-guide.md#21-check-file-fields-before-accepting-a-task).
- [x] 22. [Evaluate the useful read-only agent](build-guide.md#22-evaluate-the-useful-read-only-agent). The [baseline](live-evaluation.md) preserves model failures; separate [controlled context comparisons](live-comparisons.md) record equal-byte and selected-source conditions.

**Gate:** an authorized run lists a directory, reads relevant text, and answers.
A greeting can finish without tools. Malformed batches, unapproved tools,
traversal, missing files, repeat loops, and context exhaustion are exercised.
Capability access and byte limits hold; no writes or shell execution are enabled.
The pure `FileFieldsV1` checker rejects wrong values and forged evidence, and
the runner commits its bounded receipt with the result. Freeform stays unchecked;
strict exit zero requires completed execution and passed contract checks.

## Checkpoint C: concurrent personal use

- [x] 23. [Make each concurrent run own its state](build-guide.md#23-make-each-concurrent-run-own-its-state).
- [x] 24. [Bound admission, queues, bytes, and waiters](build-guide.md#24-bound-admission-queues-bytes-and-waiters).
- [x] 25. [Separate active runs from model capacity](build-guide.md#25-separate-active-runs-from-model-capacity).
- [x] 26. [Bound blocking tool work by its real lifetime](build-guide.md#26-bound-blocking-tool-work-by-its-real-lifetime).
- [x] 27. [Shut down and cancel without losing ownership](build-guide.md#27-shut-down-and-cancel-without-losing-ownership).
- [x] 28. [Add a batch command for independent agents](build-guide.md#28-add-a-batch-command-for-independent-agents).
- [ ] 29. [Measure saturation before tuning concurrency](build-guide.md#29-measure-saturation-before-tuning-concurrency). Synthetic warm/curve/slowdown/soak and fixed-arrival HTTP probes pass; live capacity/cold-warm and worst-case payload/checker performance remain open.
- [x] 30. [Inspect stored runs and replay decisions](build-guide.md#30-inspect-stored-runs-and-replay-decisions).
- [x] 31. [Stream output with bounded assembly](build-guide.md#31-stream-output-with-bounded-assembly). Functional proofs and the paired [live first-visible-text comparison](live-comparisons.md) pass within their recorded scope.

**Gate:** independent runs share bounded admission, model, tool, journal, and
observer capacity without mixing state or releasing permits early. Saturation,
slow consumers, cancellation, shutdown, and replay are tested. A pinned live
workload supplies latency and accepted-task-throughput baselines. Evidence and
checking remain bounded. Streaming never dispatches partial tools or publishes
a pass before terminal commit. This is the first target release for your clarified
goal; concurrency is part of it.

## Deployment options

- [ ] 32. [Reach a private remote model](build-guide.md#32-reach-a-private-remote-model).
- [ ] 33. [Optionally supervise an owned model process](build-guide.md#33-optionally-supervise-an-owned-model-process).

Step 32 is conditional on an actual remote model host. Step 33 is optional:
attach mode remains sufficient. Record a justified “not applicable” with evidence
for an optional step rather than implementing a feature solely to tick a box.
Both are **not applicable to this validation profile**: inference is local and
Kinesin attaches to the manually managed loopback server.

**Gate when used:** the same bounded adapter works over the approved private
route. Managed mode proves process ownership and cleanup without killing an
attached server or other runs when one session is cancelled.

## Checkpoint D: a shared service

- [x] 34. [Define and implement private API ingress](build-guide.md#34-define-and-implement-private-api-ingress).
- [x] 35. [Authenticate owners with provisioned credentials](build-guide.md#35-authenticate-owners-with-provisioned-credentials).
- [x] 36. [Persist owner-scoped results and idempotent creation](build-guide.md#36-persist-owner-scoped-results-and-idempotent-creation).
- [x] 37. [Give each owner bounded and fair capacity](build-guide.md#37-give-each-owner-bounded-and-fair-capacity).
- [ ] 38. [Harden the shared filesystem and service process](build-guide.md#38-harden-the-shared-filesystem-and-service-process).
- [x] 39. [Recover interruption without inventing execution history](build-guide.md#39-recover-interruption-without-inventing-execution-history).
- [x] 40. [Operate retention, backups, readiness, and overload](build-guide.md#40-operate-retention-backups-readiness-and-overload).
- [ ] 41. [Pass the shared-service exposure gate](build-guide.md#41-pass-the-shared-service-exposure-gate).

**Gate:** authentication and owner-filtered access cover every operation;
idempotent retries, concurrent first submissions, slow/disconnected clients,
fair admission, overload, storage failure, restart, retention, and backup/restore
have defined and tested behavior. Rehearse a two-owner isolation scenario before
exposure. Results and receipts survive together; retries preserve the original
contract verdict, and restart never invents a pass. See [verification](verification.md),
[security](security.md), and [testing](testing.md).

## After these checkpoints

Consider new features only against a demonstrated task or bottleneck:

| Candidate | Entry evidence | New contract required |
|-----------|----------------|-----------------------|
| More model backends | A second useful backend | Compatibility profile, data destination, separate/shared capacity identity |
| File-backed task recipes | Explicit-context evaluations improve useful tasks | Bounded selected inputs, frozen revisions, trust/capture policy, new runs for edits |
| Parallel tools in one run | Serial tool time is a measured bottleneck | Verified independence, complete-batch validation, bounded fan-out, journal/cancellation ordering |
| Backend speculation | Tool-generation latency dominates and a compatible backend exists | Target verification, complete-call gate, sidecar/cache privacy, aggregate capacity and cost |
| Model-spawned subagents | Independent sessions cannot express the task | Bounded fan-out, inherited authority, shared budgets, information-sharing rules |
| MCP tools | A needed external tool server | Protocol/lifecycle, server trust, schemas, authorization, cancellation |
| Write and edit files (built) | Read-only tools cannot change a workspace the task must edit | Separate writer capability, operator-grant consent, atomic replace, unique-match edits, barred from checked runs |
| Delete/move and shell tools | A demonstrated task needs them | Effect-specific authority, sandboxing as needed, reconciliation and approval policy; shell is a distinct process-spawning trust class |
| Context compaction | Useful tasks repeatedly exhaust context | Evidence preservation, complete tool groups, quality evaluations |
| Resume | Repeating an interrupted task is too costly | Unknown-effect reconciliation, version/policy changes, new execution identity |
| More controllers | A measured single-controller limit or availability requirement | Shared transactional store, leases/fencing, distributed admission and recovery |

Adding controllers is a consistency redesign. Do not put SQLite WAL on a
network share and call that horizontal scaling. More features do not by
themselves make the harness more general.
