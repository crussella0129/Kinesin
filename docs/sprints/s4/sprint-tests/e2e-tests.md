# Sprint 4 End-to-End Tests

- **Status:** possible (live, manual). The offline integration test is the
  CI-verifiable stand-in — same split as INT-0004.
- **Intent:** [INT-0008](../../../intents/INT-0008-remote-model-over-overlay.md)
- **Tested head:** `a8de1808ee9d74e62b6563c308c9568469441edd`

## Live attach (`tests/live_evaluation.rs`, `#[ignore]`d)
| Test | EARS clause (T-002) | Result |
|------|---------------------|--------|
| `attach_to_non_loopback_backend` | WHEN the live attach test runs against a `llama-server` at the host's real non-loopback address THEN it completes over that address and returns the loopback baseline's answer | **ok** — run 2026-09-11 against the pinned b6500 server bound to `0.0.0.0`; identical answer (`"ready"`) via `127.0.0.1:8080` and the host LAN IP `192.168.86.20:8080` |

The test discovers the host's primary non-loopback IPv4 (via a UDP socket's
local address, sending nothing), asks the pinned server the same fixed prompt at
`127.0.0.1:8080` and at `http://<that-ip>:8080`, and asserts identical answers —
proving "attach to another machine" is the same operation as "attach to
localhost." That a Kinesin config *admits* the non-loopback (private/overlay)
address is covered by the T-001 unit tests; this records reachability and
answer-equivalence over that address.

## CI-verifiable stand-in
The uniform code path and the admissibility of overlay addresses are proven
offline (see unit and integration): the origin policy accepts private/overlay
addresses and rejects public ones, and a run over an overlay `base_url` prepares
byte-identical requests to a loopback run. The live attach is the manual
confirmation of the headline.

## Confirmation
```
attach: loopback="ready"  non-loopback(http://192.168.86.20:8080)="ready"
test attach_to_non_loopback_backend ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
```
Machine: NVIDIA RTX 2080 Ti, Windows 11; server bound to `0.0.0.0` so the same
process is reachable over loopback and the RFC1918 LAN address `192.168.86.20`.

