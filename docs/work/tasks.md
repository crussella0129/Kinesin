# Agent Tasks (Persistent Backlog)

- [ ] T-003 (sprint 10) [intent: INT-0022]: Repair freeform read compaction and continuation admission — touches: src/core.rs; src/policy.rs; src/runner.rs; src/replay.rs; src/verification.rs; tests/runner_tools.rs; tests/replay.rs; tests/live_evaluation.rs
- [ ] T-005 (sprint 10) [intent: INT-0022]: Close Linux sandbox truncation, network and private-state gaps — touches: src/tools.rs; src/config.rs; tests/sandbox_linux.rs; src/bin/cmd-fixture.rs; docs/security.md
- [ ] T-006 (sprint 10) [intent: INT-0022]: Bound MCP protocol and own server process lifecycle — touches: src/mcp.rs; src/bin/mcp-fixture.rs; tests/mcp.rs; Cargo.toml if required
- [ ] T-007 (sprint 10) [intent: INT-0022]: Move MCP preparation under bounded run ownership and enforce both gates — touches: src/scheduler.rs; src/service.rs; src/cli.rs; src/runner.rs; src/policy.rs; src/replay.rs; tests/mcp.rs; tests/service.rs
- [ ] T-008 (sprint 10) [intent: INT-0022]: Enforce confidential remote model origins — touches: src/config.rs; tests/runner_tools.rs; tests/model_protocol.rs; docs/configuration.md; docs/integration.md; tests/live_evaluation.rs
- [ ] T-009 (sprint 10) [intent: INT-0021, INT-0022]: Repair dependency gate and refresh assurance/roadmap — touches: deny.toml; docs/threat-model.md; docs/roadmap.md; docs/security.md; docs/loop-and-tools.md; docs/configuration.md; docs/supply-chain.md; README.md
- [ ] T-010 (sprint 10) [intent: INT-0022]: Make workspace move preserve an existing destination atomically — touches: src/tools.rs; tests/runner_tools.rs; tools unit tests; docs/security.md
- [ ] T-011 (sprint 10) [intent: INT-0021, INT-0022]: Verify integrated contracts and finish the sprint checkpoint — touches: tests and docs/sprints/s10/sprint-tests; docs/work; sprint-meta; affected documentation
