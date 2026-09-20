# Retained operator glue

These are byte-for-byte snapshots of the ignored live preparation/driving
scripts, stored as text for provenance. They are not product commands or an
evaluation framework. Do not rerun them: their original paths and exclusive
attempt records are specific to this completed experiment.

The original slot-1 postprocessing expected a nested `record` field and failed
after inference had completed. The retained `run_slot.py.txt` is the corrected
version used for subsequent one-shot requests; slot 1 was exported separately,
never rerun. `run_session_slot.py.txt` was used for requests 5/6 and held their
stdin open until evidence collection requested closure, without extra prompts.
The relay saved unmodified request/response bodies and refused any mismatch of
the actual first body against the frozen expected hash.

`manifest.json` covers copied scripts and runtime metadata; this index was added
afterward. All scripts operated solely on the disposable lab. These are
historical evidence copies, not officially tested tooling.
