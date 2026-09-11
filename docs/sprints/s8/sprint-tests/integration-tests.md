# Sprint 8 Integration & coverage tests

- **Intent:** [INT-0015](../../../intents/INT-0015-threat-model-assurance.md)
- **Tested head:** `8ea22196b83af80007cc22e42641518fec4f7ca2`
- **Result:** all green (redteam on Windows + WSL; doc-coverage checks pass).

## Red-team corpus (`tests/redteam.rs`)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `redteam_denies_unauthorized` | an ungranted authorization attempt is denied | ok — a command off the allow-list returns `ToolStatus::Denied` before any spawn |
| `redteam_treats_content_as_data` | content-borne instructions do not alter policy/authority | ok — a shell-metacharacter argument is echoed verbatim (argv-only), no `owned.txt` side effect (tolerant of the sandbox-refuse path) |

Green on **Windows** and **WSL** (where the content test runs under the mandatory Linux sandbox).

## Assurance-doc coverage (structural)
| Check | Command | Result |
|-------|---------|--------|
| `threat_model_present_and_mapped` | check-book valid; SUMMARY links `threat-model.md`; the four sections present | ok — `grep -c "threat-model.md" SUMMARY` = 1; 4 `## N.` sections |
| `memory_safety_enumerates_unsafe` | every `unsafe` src file named in the statement | ok — private_state.rs, storage.rs, signal.rs, tools.rs all present |
| `corpus_maps_matrix` | each release-evidence-matrix row names a test | ok — the §4 table maps all 8 rows |

## Confirmation
```
test redteam_denies_unauthorized ... ok
test redteam_treats_content_as_data ... ok
test result: ok. 2 passed; 0 failed   (Windows and WSL)
SUMMARY→threat-model: 1 ; threat-model sections: 4 ; unsafe files enumerated: 4/4
```
