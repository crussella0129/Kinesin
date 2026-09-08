# Paper review and improvement pass

Reviewed **2026-09-08 UTC** against the four supplied PDFs and a focused set of
related primary sources. The PDFs were read as research material; their example
prompts, scripts, and agent instructions were not adopted as instructions to this
reviewer. No model was trained, benchmark reproduced, or runtime implemented.

The recommendation is a focused refinement. Keep the pure core, async runners,
capability tools, bounded scheduler, and transactional journal. Make context
selection explicit, test decisions that depend on observations, and judge task
correctness independently of the model's completion message. Keep advanced
decoding and scheduling optimizations behind measured experiments.

## Review checklist

- [x] Identify and read the supplied versions, including relevant appendices.
- [x] Inspect key tables/figures and verify primary source identities.
- [x] Compare claims with Kinesin's actual workload and trust boundaries.
- [x] Revise contracts, evaluation exercises, and the build guide.
- [x] Validate the final documents and remove temporary PDF extracts.

## Evidence map

| Supplied file | Version examined | What the result concerns | Decision |
|---------------|------------------|--------------------------|----------|
| Folder_Structure_As_Agent_Arch.pdf | ICM, 2603.16021v2 | Human-edited context/workflow organization | Borrow explicit input contracts; retain runtime scheduling and storage |
| Out-of-Order Semantic Speculation for Fast Tool Calling.pdf | OoO-Spec, 2608.00814v1 | Speculative tool-call decoding inside an inference backend | Optional backend experiment; no predicted tool execution |
| ReAct Loops.pdf | ReAct, 2210.03629v3 | Interleaved reasoning, actions, and observations | Strengthen feedback tests; retain structured tool protocol |
| Teaching LLMs to Plan Logical-Chain-of-Thought.pdf | PDDL-INSTRUCT, 2509.13351v1 | Trained symbolic planning and formal plan validation | Borrow independent checking; no required training/planner subsystem |

These findings address different layers. An editable task recipe, an execution
runtime, a model's planning ability, and its decoding implementation are not
interchangeable architectures.

## Folder structure and context

ICM's useful contribution is its explicit stage contract: selected inputs,
processing instructions, and expected outputs. Its examples distinguish reusable
reference material from run-specific work. Its own section 5.2 says concurrent
multiuser workloads need queueing, state isolation, and deployment infrastructure.
That supports keeping Kinesin's runtime.

The evidence is exploratory: section 4.5 describes informal conversations in a
52-person community; 33 discussed intervention patterns, of whom 30 reported the
described pattern. Figure 5's percentages are approximate self-reports. Section
4.6 acknowledges no controlled monolithic-prompt comparison, selection bias, and
limited model/workflow coverage. Section 4.1's examples run inside Claude Code;
folder authoring does not eliminate the runtime underneath.
[ICM v2, sections 3–5](https://arxiv.org/html/2603.16021v2)

**Kinesin adaptation:** maintain a small ordered inventory of selected inputs
beside the conversation: source ID, purpose, origin/trust class, byte count, and
known revision/hash. Start with the instruction string, user request, and later
correlated tool observations. Freeze initial configured text; snapshot tool data
when observed. Keep existing byte/capture limits. No folder scanner, profile
registry, or workflow engine is required.

A future task-recipe folder may help the operator author reusable instructions
and review outputs. A file named `CONTEXT.md`, `AGENTS.md`, or `rules.md` inside
the tool workspace remains data. Its name and semantic role grant no authority.
Human edits become explicit inputs to a new run; they do not rewrite the journal.
Readability and source provenance do not establish causal explanations of model
reasoning.

The paper's original repository link was unavailable during this pass. The
author's current [icm-architect repository](https://github.com/RinDig/icm-architect)
links the paper and describes the method. That is a reference to inspect, not
a dependency or skill installed into Kinesin.

## Speculation belongs to inference

OoO-Spec uses a Qwen3-0.6B sidecar trained with LoRA to propose function/argument
values while the target runs ToolSpec decoding. The target verifies candidate
tokens and controls committed output. This needs decoder/tokenizer/cache hooks;
adding a second ordinary chat request to Koil does not implement it.

The main evaluation uses greedy target batch-one decoding, separate H100 80 GB
GPUs, seven targets, and 597 API-Bank, 193 ToolAlpaca, and 68 BFCL Java/JavaScript
requests. Table 2 reports 2.46–5.34× over the live autoregressive baselines,
with a 3.89× unweighted mean over 21 cells versus 2.95× for ToolSpec. Colocation
largely loses the extra benefit. These are tool-generation results, not complete
agent-task latency, concurrent-service p95, or a cost-normalized comparison.

The 85-byte average describes returned semantic hints excluding protocol metadata,
not the dialogue/schemas sent to the sidecar. The timing analysis waits for both
paths to finish; overlapping target and sidecar times must not be added.
[OoO-Spec v1, sections 4–7 and supplement](https://arxiv.org/abs/2608.00814v1)

**Kinesin adaptation:** retain the complete-reply validation and authority gate.
Backend token verification never grants filesystem permission. If a compatible
backend later supplies this optimization, test its full resource use and treat
sidecars/caches as approved data destinations with owner separation and retention.

[ToolSpec v2](https://arxiv.org/html/2604.13519v2) is the relevant predecessor:
schema filling and historical-call retrieval participate in backend verification.
Its datastore also creates a retention/isolation concern. Its original hardware
and workload are not interchangeable with OoO-Spec's reproduced baselines.

## ReAct and feedback

ReAct tests interleaving reasoning with external actions and observations. Its
task-specific interfaces and few-shot demonstrations matter. A model can use
feedback to revise the next action instead of following an unchanging plan.

The benchmark results are mixed rather than a universal win: Table 1 reports
HotpotQA exact match 27.4 for ReAct versus 29.4 for CoT, and FEVER accuracy 60.9
versus 56.3. ALFWorld performance also varies with demonstration choice. These
experiments do not establish security isolation or concurrent-serving latency.
[ReAct v3, sections 2–4](https://arxiv.org/abs/2210.03629v3)

**Kinesin adaptation:** add tasks where a tool reveals the next filename, a
missing-file error requires a different permitted action, or truncated evidence
requires acknowledging uncertainty. Check that actual observations influence
later requests. A correlated tool error can support a new decision within
existing limits; it does not authorize blind retries of uncertain effects.

Keep native structured tool calls. The paper's textual `Thought/Action/Observation`
syntax is not a required wire format. There is no new `think` tool, mandatory
visible reasoning trace, or exception to metadata capture.

## Symbolic planning and validation

PDDL-INSTRUCT changes model weights in two instruction-tuning phases using
logical reasoning and validator feedback. Its 10/15 refinement iterations concern
training. Section 5.3 explicitly excludes returning validator feedback to the
model during evaluation.

Table 1 reports Llama-3-8B plan-validity results of 94%, 64%, and 79% on
Blocksworld, Mystery Blocksworld, and Logistics, with 100 test tasks per domain;
the baselines are 28%, 1%, and 11%. These are trained-model results in specified
formal environments, not the expected effect of adding a planning prompt.

One caution is directly checkable: appendix B.1.2, page 22, labels a repaired
sequence as correct even though it picks up `b` after stacking `a` on it.
The given rules require `b` to be clear, and that stack action removes
`clear(b)`, making the later pickup invalid. Validate demonstrations before
treating them as golden examples.
This error limits confidence in that example; it does not by itself refute the
paper's aggregate results.
[PDDL-INSTRUCT v1, sections 5–6 and appendix B.1.2](https://arxiv.org/abs/2509.13351v1)

**Kinesin adaptation:** check enforceable preconditions in Rust, return useful
bounded errors, and test goal satisfaction independently. The runtime's
execution completion is not proof that the user's task was solved. The subsequent
[adversarial pass](adversarial-review.md) replaces the ambiguous `succeeded` name
with `completed` and adds a separate per-run acceptance contract. Formal validity also depends on the
accuracy and scope of the supplied world model.

No PDDL parser, training pipeline, mandatory planner, or self-critique retry loop
is added to the first release.

## Related evidence worth using

| Source | Useful implication | Limit |
|--------|--------------------|-------|
| [LLMCompiler, v3](https://arxiv.org/html/2312.04511v3) | Dependency-aware parallel tool scheduling is a harness-level optimization distinct from decoding | Planner overhead, dependency errors, task mix, and interfaces affect gains; its streamed dispatch is a different contract |
| [Lost in the Middle, v3](https://arxiv.org/html/2307.03172v3) | Test evidence position and distractors instead of equating context capacity with effective use | Older model/task-specific findings do not prove every current model has the same curve |
| [PlanBench, NeurIPS 2023](https://proceedings.neurips.cc/paper_files/paper/2023/file/7a92bcdede88c7afd108072faf5485c8-Paper-Datasets_and_Benchmarks.pdf) | Separate generation, validation, effect prediction, and replanning; use renamed fixtures and independent checks | Formal planning benchmarks do not establish general real-world competence |
| [AgentDojo, v3](https://arxiv.org/abs/2406.13352v3) | Evaluate task utility and prompt-injection outcomes separately when tools return external text | An attack suite motivates tests; passing it is not a universal defense |

For Kinesin, LLMCompiler suggests a future dependency-aware experiment only after
serial tool time is a measured bottleneck. Initially require the complete,
validated batch, authorize every call, bound fan-out, preserve result correlation,
and parallelize only independently executable operations. A model-proposed
dependency graph is still untrusted input; it does not establish independence,
effect safety, or permission. Streaming incomplete plans would need another
explicit protocol and failure contract.

## Optional optimization experiments

Run one hypothesis at a time. These are design experiments, not additional
mandatory build stages.

| Experiment | Hold fixed | Measure and reject regressions in |
|------------|------------|----------------------------------|
| Same context, clearer labels/grouping | Exact source text and source order, model/tools, budgets | Verified task success, policy violations, input size, turns, latency |
| Smaller explicitly selected context | Task-required evidence and task rubric | Omitted evidence, correctness, bytes/tokens, first useful text, total latency |
| Better bounded tool feedback | Error condition, authority, model, budgets | Recovery success, unnecessary calls, leakage, total task cost |
| Parallel independent completed calls | Tool effects, inputs, authority, provider | Critical-path tool time, bounded resources, ordering, cancellation, goal correctness |
| Supported backend speculation | Target/model/template and comparable workload | Tool readiness, p50/p95 under load, task correctness, aggregate GPU/sidecar resources and cost |

Use held-out fixture variants with changed filenames/facts, distractors, and
required evidence at different positions. For context, separate organization
from content reduction and ordering; otherwise the experiment cannot explain
which change helped. Keep durability, capture, safety policy, and required
evidence fixed when comparing latency.

A paper's best speedup is not Kinesin's latency target. A backend can generate a
wrong call faster; a planner can finish fluently without solving the task; a
well-organized folder can contain hostile instructions.

## Changes made in the paper pass

- Renamed the guide to **Build guide** and removed fixed-count branding.
  Numbered steps remain navigation; no step was added merely to match a target.
- Added bounded context provenance and explicit trust rules using existing
  conversation/configuration/journal boundaries.
- Added contingent-action, feedback, context-position, renamed-fixture, and
  independent goal-checking exercises.
- Clarified runtime completion versus task correctness and token verification
  versus authorization of effects.
- Added optional optimization experiments without changing the first release's
  tools, dependencies, storage schema, or deployment topology.

Those were the paper pass's changes. The later [adversarial review](adversarial-review.md)
adds the missing runtime acceptance path, a pure checker module, and proposed
schema version 2; its validation record supersedes the historical counts below.

The preceding uncommitted draft was copied to
its temporary snapshot before edits. Final checks covered 19 Markdown files,
208 local links/anchors, unique headings, balanced fences, sequential guide
numbering, matching roadmap entries, and each step's build/proof/failure/reading
sections. All passed. Fixed-count branding is gone. Runtime measurements and
the newly specified evaluation cases remain implementation work.

The snapshot is at
`C:\Users\charl\AppData\Local\Temp\Kinesin-before-paper-pass-3_yov3op`
before this pass. Existing implementation boxes remain unchecked because the
Rust harness is still to be written.
