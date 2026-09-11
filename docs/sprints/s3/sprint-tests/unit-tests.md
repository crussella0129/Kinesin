# Sprint 3 Unit Tests

- **Intent:** [INT-0004](../../../intents/INT-0004-kv-cache-reuse.md)
- **Tested head:** `326bc505391e58bedda17594cb32f21ec4a4e0fd`
- **Runner:** `cargo test --locked --lib`
- **Result:** all green.

## T-001 unit tests
| Test | File | EARS clause | Result |
|------|------|-------------|--------|
| `cache_prompt_present_when_enabled` | src/model.rs | WHEN enabled THEN body includes `cache_prompt = true` | ok |
| `cache_prompt_absent_when_disabled` | src/model.rs | WHEN disabled THEN body omits `cache_prompt` | ok |
| `cache_prompt_defaults_on` | src/config.rs | WHEN config omits the setting THEN it defaults enabled (and an explicit `cache_prompt = false` is honored) | ok |

## Confirmation
```
test model::tests::cache_prompt_absent_when_disabled ... ok
test model::tests::cache_prompt_present_when_enabled ... ok
test config::tests::cache_prompt_defaults_on ... ok
```
