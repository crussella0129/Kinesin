# Sprint 3 Meta

- **Sprint number:** 3
- **Book schema version:** 2
- **Start timestamp:** 2026-09-11T04:40:30Z
- **End timestamp:** 2026-09-11T05:42:45Z
- **Model:** claude-opus-4-8
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Reuse llama.cpp's cached prompt prefix via `cache_prompt` (config-toggled), preserving immutability/replay; measure the reduction on a live server (INT-0004).
- **Intents:** [INT-0004](../../intents/INT-0004-kv-cache-reuse.md) — realized
- **Completion evidence:** INT-0004 realized: cache_prompt reuse shipped; live benchmark measured ~18.4x prompt-eval reduction (933ms->51ms) on a ~2.6k-token shared prefix, 241 offline tests green, proceed-with-caveats critique accepted
