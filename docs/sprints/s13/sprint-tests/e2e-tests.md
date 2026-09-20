# Sprint 13 live storefront evidence

Status: the live confidence gate passed at approximately 04:52 UTC, before official unit/integration checks were authorized. The latest preceding journal operation completed at `2026-09-20T04:52:05.749Z`; browser observations and `/exit` are operator evidence, not separately timestamped run events. This order follows the user's explicit request to operate the real assistant first, discover failures, repair them, and run formal checks afterward. Final-binary reconfirmation remains separate from this initial gate.

## Environment and provenance

- Disposable application workspace: `target/storefront-lab/app`; trusted profile and replay journal: `target/storefront-lab/control/kinesin.toml` and `control/state/kinesin.sqlite`.
- Native Kinesin CLI, managed local Qwen2.5-Coder 7B Q4_K_M model, localhost model endpoint, 16,384-token context, temperature 0, one verified slot. Per run: 240 seconds, 20 model turns, 30 tool calls, 2,400 output tokens.
- Granted workspace operations initially: list/read/search/create/write/edit and `run_command` with `node` as the sole command. The generated app uses HTML, CSS and JavaScript without external dependencies or real payments.
- All application source changes below were dispatched by Kinesin's model through its actual file tools. The operator supplied tasks and observed failures. The disposable directory is not an OS security sandbox.
- Journal observations were read through a read-only SQLite connection. Only synthetic workload details are reproduced here. Times below are UTC on 2026-09-20.
- `completed / answer_candidate` is an unchecked freeform answer, not a claim that the app passed functional acceptance. Browser behavior and actual tool events determine the outcomes recorded here.

## Live sequence

| Start | Run ID | Actual tool evidence | Outcome and next action |
| --- | --- | --- | --- |
| 04:29:34 | `6e4419ec-27f0-4fa3-af04-afef049ea89d` | `create_directory public` succeeded; no file write | Model printed HTML in its final answer. Operator inspected the empty directory and explicitly requested a real write. |
| 04:30:11 | `d7499ed2-d88b-44a7-a81f-bf7dc9eb6ac6` | `write_file public/index.html`, 1,618 bytes | Initial HTML persisted. It lacked checkout form inputs despite the requested checkout behavior. |
| 04:31:09 | `026d5afe-62a5-4a01-86f6-e99447c98ef3` | `write_file public/styles.css`, 2,209 bytes | Styles persisted. |
| 04:34:02 | `018f62f2-c3e6-4fd1-9bff-a3a3a7026e34` | `write_file public/app.js`, 4,235 bytes | Six-product catalog/cart implementation persisted. Model's “fully functional” answer was later contradicted by the live checkout error. |
| 04:35:19 | `4e645f86-6537-4ef5-bbfd-50157f03ebe6` | `write_file server.mjs`, 982 bytes; no command dispatched | Model fabricated a `<tool_response>` claiming the server was running. Operator requested an actual command. |
| 04:35:42 | `eb498f41-018b-4292-86a2-bae1b3cb9193` | `run_command ["node", "server.mjs"]` executed; child exited 1, `success:false` | Node reported `ReferenceError: __dirname is not defined in ES module scope` at `server.mjs:6`. The tool itself completed successfully; its child process failed. |
| 04:35:57 | `06c02e32-280e-41af-b3d9-072d0039433b` | `edit_file server.mjs`, then actual `run_command ["node", "server.mjs"]` | Browser reached `127.0.0.1:4173` while the command occupied the run. Cancellation later ended the run as `cancelled / cancelled`. |
| 04:37:17 | `d04f2efe-6c83-44e3-8323-7a7ae0a04695` | Read HTML followed by five successful `edit_file` calls | Model added required name/email form fields and a confirmation element, changed category option values/labels, and updated the footer. JavaScript still required a corresponding repair. |
| 04:38:05 | `c10e5e44-6b80-4089-8319-78b248e744f0` | `read_file public/app.js` only | Model emitted another fabricated `<tool_response>` containing JavaScript. No edit/write occurred; repair was not yet applied. |

## Browser observations and failures

The operator opened the actual Node server in the internal browser, observed six products, and added two Desk Lamps. The displayed total was $50.00. Clicking Checkout produced `TypeError: Cannot read properties of null (reading 'value')` at `app.js:86`, timestamp `2026-09-20T04:36:25.539Z`. This exposed the missing HTML inputs through normal interaction.

The Node server remained inside the active `run_command`; Kinesin could not accept the next editing turn while that command occupied the run. Ctrl+C cancelled the native session and stopped the owned child process. Reloading the browser then produced connection refused. This establishes the need for a separately owned localhost preview lifecycle without weakening command-child cleanup.

These failures are retained as live evidence. Model answers that merely claim a write, repair or server startup do not count as successful operations.

## Owned preview and further live repairs

At 04:39:30, run `68c2cfc4-de03-4999-91e0-064cfb01dbc8` dispatched the new `start_preview` tool for `public`. Its successful tool result returned a loopback URL on port 50803 with a random path prefix. The operator loaded that URL in the internal browser and continued sending editing turns while the preview stayed available. This resolves the observed command-occupancy wall for static previews; the Node command cleanup contract remains intact.

The browser then exposed more defects in the model's generated application: an invalid regular expression (`SyntaxError`, `2026-09-20T04:40:49.992Z`), a duplicate `const products` declaration after splitting JavaScript, and inline click handlers rejected by the preview's content security policy. The operator fed these observed failures back as increasingly narrow corrections. Initial app generation was model-authored; later changes included precise operator-guided patches. Every applied file change still went through Kinesin's actual file tools.

Representative journal evidence:

- `8c7c9f71-cf8b-4301-b7cc-0c91c2b7332e` wrote a 6,863-byte replacement `app.js`; writing the file did not establish valid JavaScript. Runs `691c829c-76fc-47fe-bd36-19f63579d44c`, `3676739f-5a1e-4bb5-8c84-e99609b6991c` and `b08c5b8f-d606-41c2-b814-d3e893a04569` subsequently wrote separate catalog, cart and checkout files.
- Runs `b11102f5-89b5-4ee8-9a54-8b0371d68d37` and `b73f6cba-8760-4ade-87f7-328222e67545` returned `match_not_found` for exact edits. These attempts did not change their target files.
- At about 04:44 the operator used `/new` to clear accumulated unreliable answers. Run `12a108cb-6b79-4a34-839f-5befe162c950` applied four cart edits, then ended `failed / empty_response`. Its persisted file mutations remain real despite the failed final model response.
- Runs `d5c1d822-18a7-4f27-87a0-5544124d3a04` and `04595010-5f50-4059-b975-1365dd00cdf8` emitted fabricated `<tool_response>` edit claims after a read or with no tool activity. Neither is counted as a repair. Additional explicit instructions against fabricated tool responses did not eliminate this behavior.
- At 04:47:27, run `23fccc80-a173-406e-8330-7ad7757a54f2` performed two successful `edit_file public/app.js` calls for the inline Add to Cart handler correction. Functional confirmation still depends on the following browser pass.

The following browser pass confirmed repaired Add to Cart behavior: two Desk Lamps produced a $50.00 cart. Demo checkout with synthetic customer `Demo Shopper` / `demo@example.test` reached a confirmation, but it displayed `$5000.00`. This exposed a cents-to-dollars formatting defect despite correct cart arithmetic. Run `87975460-8327-495f-b055-8e5d0007a29e` applied the operator's narrow divide-by-100 correction; the successful browser outcome is recorded below.

The profile includes `start_preview` and instructions against fabricated responses; its effective `max_output_tokens` remained 2,400. An attempted increase did not match the profile text and did not take effect. A full stylesheet rewrite (`ce2a9124-ce5e-4687-8b59-fb7c065cbd2e`) stopped at `generation_length`, so no new stylesheet was written. A narrow box-sizing edit succeeded in `08ec2031-7a59-4ea7-a5a1-d715b9dfce6c` and completed at 04:52:05.749 UTC. The exercise establishes basic functional delivery with operator steering, not autonomous design quality or reliable first-attempt model behavior.

## Live confidence gate: passed

The internal browser independently exercised the served final app, after actual tool-applied repairs:

| Interaction | Observed result |
| --- | --- |
| Case/whitespace search with `LAMP`; category `paper`; nonsense search `zzzz` | Desk Lamp matched; Paper showed Bookshelf and Pen Set; nonsense search showed the no-results state. |
| Add two Desk Lamps | Cart total $50.00. |
| Increase Pen Set to two, then reload | Quantity two and $10.00 total survived reload after startup rendering was repaired. |
| Decrease quantity two → one → zero | Correct decrements, then empty cart and $0.00 total. |
| Explicit Remove for a Desk Lamp | Empty cart and $0.00 total. |
| Checkout an empty cart with synthetic customer details | Visible empty-cart error. |
| Checkout two lamps with an invalid email | Native email validation prevented submission and retained the $50.00 cart. |
| Checkout with `Demo Shopper` / `demo@example.test` | Visible `Order confirmed`, customer name and `Total: $50.00`; cart cleared to $0.00. |
| Send follow-up edits while preview runs | Preview stayed available across turns; reload reflected applied edits. |
| `/exit` from native session 53856 | CLI exited cleanly; a subsequent HTTP request to the owned port 50803 URL failed with connection refused. |

Persistence repair included successful `edit_file public/cart.js` operations in `a787f091-b701-4645-abc3-a450d8301dcc`, `aca267f3-c01b-49a3-a29f-90ec261ab0c8` and `9d150bc1-9716-402b-ae8b-fac000833efa`. The first of those also tried to start a different preview root twice, received `preview_already_running`, and stopped at the repeat limit. This did not replace or disrupt the already-running `public` preview.

The five live scenarios in the locked test plan are evidenced by the isolated profile and real journal events, actual file creation/revision, successful browser flows, retained failure/recovery record, and observed owned-server shutdown. Official unit/integration checks may now begin. Later startup-deadline and replay-compatibility harness changes require final-binary reconfirmation, recorded separately; this gate does not claim they were in the earlier running binary.

## Retained workload

[The storefront fixture](fixtures/storefront/public/index.html) contains an exact copy of the final five public files, captured at `2026-09-20T04:54:44.097910Z`. The [SHA-256 manifest](fixtures/storefront/manifest.json) identifies each file's bytes and hash. [Selected journal records](fixtures/storefront/selected-runs.json) retain the initial prompt, preview request, precise operator-guided repairs, effective limits, grants and actual tool observations. The original replay SQLite remains in the disposable local control directory; it is not committed. The generated Node server is omitted because the delivered workflow uses Kinesin's owned preview.
