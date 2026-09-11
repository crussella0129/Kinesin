# Sprint 4 Meta

- **Sprint number:** 4
- **Book schema version:** 2
- **Start timestamp:** 2026-09-11T13:46:58Z
- **End timestamp:** 2026-09-11T14:21:06Z
- **Model:** claude-opus-4-8
- **Bundle version:** 0.22.0
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Make attaching to a remote llama-server as easy as localhost — a uniform backend seam with an address-privacy policy (private/overlay HTTP allowed, public rejected by default), encrypted fabric out-of-band (Tailscale now, Koil later) (INT-0008).
- **Intents:** [INT-0008](../../intents/INT-0008-remote-model-over-overlay.md) — realized
- **Completion evidence:** INT-0008 realized: uniform local/remote model transport shipped; origin policy admits private/overlay HTTP and rejects public by default; live attach executed (identical answer over loopback and LAN 192.168.86.20); full suite green, proceed-with-caveats critique accepted
- **Checkpoint:** https://github.com/crussella0129/Kinesin/pull/5 (the open dev → main PR from sprint 3, still awaiting merge, now also carries sprint 4 — the adapter opens at most one dev → main PR at a time)
