# INT-0006 — Skills and progressive disclosure

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

> **Roadmap:** theme D (SotA capability) — see [the roadmap](../roadmap.md) (INT-0011).

## Intent
Support operator-installed, on-disk skills — a directory per skill holding an
instruction file with a name and description — whose instructions load into a
run as framed reference data. Installation by the operator is the consent, the
way an installed package is trusted while an ingested web page is not; a file's
name or location grants no authority. Optionally load a skill's body on demand
(progressive disclosure) within the run's context budget rather than always.
Non-goal: letting a workspace file named `SKILL.md`, `AGENTS.md`, or `rules.md`
become installed policy or grant a tool.

## Acceptance criteria
- An operator-installed skill's instructions reach a run as data, marked by
  origin, and grant no permission.
- A skill file discovered inside a tool workspace remains untrusted data and
  cannot install policy or a tool.
- If on-demand loading is built, it respects the existing history/request
  budgets and the frozen-input rule.
- Tests cover installation-as-consent, the untrusted-workspace-file case, and
  any budget interaction.

## Rationale
Reusable operator-authored instructions are a field norm (Maka, Animus_Ferric),
and the paper review already worked out the trust model as "file-backed task
recipes."

## Alternatives
Keep instructions in `kinesin.toml` only (current). A folder scanner or profile
registry (rejected in the paper review as unnecessary machinery).

## Consequences
Interacts with the context budget and the frozen-input rule; the genuinely hard
part is on-demand loading, which trades relevance against a fixed context.

## Transition history
- 2026-09-08: created as `proposed`.
