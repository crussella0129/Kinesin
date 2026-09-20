# Plan Critique — Sprint 12

## Concerns

### C-001: Encoded-request admission needs an explicit MCP boundary
- **Where:** build-plan.md T-111 / test-plan.md Unit Tests
- **Quote:** "Initial history/encoded request bounds are honored."
- **Failure mode:** hidden-dep
- **Why it matters:** MCP schemas are discovered after CLI authorization; the
  normal personal profile uses compiled tools and supports a smaller scope.
- **Suggested response:** fix-in-plan — specify compiled-tool request admission.
- **Resolution:** both plans now explicitly scope fitting to compiled schemas;
  dynamic MCP fitting stays with INT-0026 while runtime limits remain enforced.

## Confidence
proceed-with-caveats
