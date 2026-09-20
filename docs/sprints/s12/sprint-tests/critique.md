# Test Critique — Sprint 12

## Concerns

### C-001: JSON notice routing is not exercised
- **Where:** T-111 command/output EARS; integration-tests.md JSON row;
  explicit_json_session_keeps_stdout_as_structured_receipts.
- **Quote:** "JSON output remains machine-readable and context notices do not become result receipts."
- **Failure mode:** weak-assertion
- **Why it matters:** The original one-reply test never triggered a context
  notice and could not detect notice text contaminating stdout.
- **Suggested response:** tighten-assertion — exercise an oversized answer and
  failed entry, parse every stdout receipt and assert notices only on stderr.

## Confidence
block
