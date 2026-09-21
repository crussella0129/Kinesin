# Bounded diagnostic evidence archive

Requests 1–4: **80 files, 280,184 bytes** before this index and its manifest.
[archive-1-4-manifest.json](archive-1-4-manifest.json) records every retained
file's size and SHA-256. Its SHA-256 is
`1f27274e1267bba7a1bcd922f1cea4fd9943f1a4679b52be569b741ab2388058`.

Files were copied byte-for-byte from `target/s15-live/` after independent file
scoring. They contain synthetic fixture data and local execution provenance.
Each call retains its freeze/start record, available finish record, capture,
relayed wire requests/responses/timings, stderr, effective profile and final app.
The common fixtures retain original inventory/shipping seeds, exact prompts and
the original seed manifest. That manifest also lists later repair/fallback
fixtures which are outside this requests-1–4 archive subset.

SQLite databases/locks, executables/models, duplicate expected-first request
bodies and stdout/export copies were excluded. The captures retain journal
events; each relayed first request was independently compared to its frozen
expected body before archiving. Raw lab originals remain unchanged.

The slot-1 driver postprocessing error did not trigger a new request. Its manual
capture export is retained; absent completion metadata must not be interpreted
as a rerun, a wall-time measurement or an additional diagnostic slot.

## Appended requests 5–6

The separate [archive-5-6-manifest.json](archive-5-6-manifest.json) records
**36 files, 88,265 bytes**, before the manifest itself. Its SHA-256 is
`26120a2cf5e0cf9baab79db95a6218ca6e0d6634b17b03eb56c4a8ab15d71593`.
The requests-1–4 files and manifest were left unchanged.

This append retains both repair requests' freezes, starts, finishes, process and
terminal records, captures, wire bodies/timings, stderr, profiles, exact accepted
prompt files and unchanged final app copies. The authentic starting browser
observation, seed and pre-dispatch prompt copies remain separately in
[repair-preparation/](../repair-preparation/observation.json). No raw lab originals
were modified; no database, binary, model or redundant export was copied.
