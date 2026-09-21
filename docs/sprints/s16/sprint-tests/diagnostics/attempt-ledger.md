# Sprint 16 diagnostic results

Three of at most four conditional slots consumed. A, B and C each failed;
the locked decision stops after C failure. Slot 4 was not submitted. All three
calls used independent original-seed app copies and empty normal CLI sessions.
The independently reproduced held-out seed is frozen but was never submitted.

| Slot | Run and evidence | Result and actual costs |
| --- | --- | --- |
| 1 / A | `ff9e7736-5c8f-4dd3-81c3-36bd565b6f65`; [observation](evidence/call-01/evidence/observation.json), [freeze](evidence/call-01/evidence/freeze.json), [capture](evidence/call-01/evidence/capture.json) | FAIL. Asked for files already accessible through tools. 2 turns, 0 tools, 2,783 prompt / 77 completion tokens, 2.281 s. |
| 2 / B | `2eae4e7c-d9a3-40a2-b9c8-973d23fbf1b3`; [observation](evidence/call-02/evidence/observation.json), [freeze](evidence/call-02/evidence/freeze.json), [capture](evidence/call-02/evidence/capture.json) | FAIL. Repeated the request and observed filenames, no operations. 2 turns, 0 tools, 3,007 prompt / 175 completion tokens, 2.328 s. |
| 3 / C | `28955c2f-6d4e-47f2-b48e-14c175abcc5c`; [observation](evidence/call-03/evidence/observation.json), [freeze](evidence/call-03/evidence/freeze.json), [capture](evidence/call-03/evidence/capture.json) | FAIL. Identified the assignment bug and falsely claimed an edit/control success. 2 turns, 0 tools, 5,203 prompt / 183 completion tokens, 2.844 s. |

Token counts sum provider-reported values across each run, including cached
prompt tokens; they are not unique context size or a reliability measure.
Each used the single bounded completion review and then ended with
`answer_candidate` / `unchecked`. No repair nudge, truncation, transport failure,
context stop or budget exhaustion was observed. First requests were 10,391,
10,806 and 14,332 bytes; provider first prompt counts were 1,339, 1,451 and 2,549
respectively (see actual wire responses for authoritative values).

All final original app bytes match the seed. No candidate preview was started
or returned, so candidate browser criteria are unobserved and cannot pass.
The seed browser observations remain preparation evidence only. C's prose
diagnosis is a different observation from applied or useful work; its first
answer proposes deleting the total assignment, which alone would leave $0.
Its second answer instead claims accumulation, still with no edit. No candidate
code was fabricated or manually patched to complete this repair.

B/C are explicitly assisted diagnostic prompts, not normal-request successes.
No mid-run corrective message, tool forcing, app patch or context reset occurred.
Lab/fixture preparation, browser interaction and scoring were operator work;
human active time was not measured. The wire relay preserved exact requests and
only validated their first-request hashes; it did not synthesize model outputs.
Model prompt caching remained enabled as frozen, and fresh sessions are not a
claim of cold inference caches. S16 A exactly matches S15 slot 5's first wire,
but this is another negative observation, not a repeatability estimate.

All owned sessions closed. [Cleanup](cleanup.json) records identity-checked
termination of the model, relay and seed server, each returning 0, and no
remaining listeners on 51640, 51641 or 61458. The temporary seed tab was closed;
user tabs were left alone. No product source/default/install changed.
