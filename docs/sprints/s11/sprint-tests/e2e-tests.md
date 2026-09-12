# Sprint 11 Real First-Use Tests

Intent: [INT-0028](../../../intents/INT-0028-first-use-documentation.md).
Named verification: `platform_smoke_evidence` (T-002/A), supporting T-001/B.
E2E is possible and was executed on both operating systems. These are bounded
manual model checks, separate from deterministic offline CI.

| Host / command | Run ID | Observed result |
| --- | --- | --- |
| Windows greeting | a5c2d136-da01-4244-8915-e69e1d21fcf2 | completed, unchecked, task_accepted=false; --allow-unchecked gives exit 0 |
| Windows practice-fields | 6cab74be-fc2f-41dc-a158-5d9b15b789bf | completed, passed; both evidence-backed field criteria pass; exit 0 |
| Windows bare session, one prompt then EOF | 29d5a50b-a4be-44bb-91e5-b6373755b6bc | completed, unchecked; output names Kinesin and Rust; exit 0 |
| Debian greeting | 72a32213-8c1b-407c-8eca-c12df3185d37 | completed, unchecked; exit 0 |
| Debian practice-fields | 55c70bb1-adaf-4d0d-9468-9e5d8baa7660 | completed, passed; both evidence-backed field criteria pass; exit 0 |
| Debian bare session, one prompt then EOF | 2388a5ee-80b3-46a5-a47e-23d6926c5346 | completed, unchecked; output names Kinesin and Rust; exit 0 |

Inspect, export and replay exited 0 on each host. Both checked exports replayed
consistently after the owned model server and Windows SSH forward were stopped.
Replay success proves recorded internal consistency, not independent model quality
or freshness. No particular greeting text was required.

The server used retained llama.cpp b6500 / Qwen2.5-Coder-7B Q4_K_M files,
Vulkan0 on the RTX 2070 Super, a 4096-token context and one slot. Linux and
Windows live sequences were serialized. The server bound only 127.0.0.1:18080;
Windows connected at 127.0.0.1:8080 through Tailscale SSH as charles@nighthawk.
The process was stopped only after its executable and start time matched.
The final health probe failed to connect and the listener disappeared.

See [platform-verification.md](platform-verification.md) for commands, hashes,
log locations, scoped provisioning and unexecuted prerequisites.
