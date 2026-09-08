# Trace schema

A trace is the record of one message that crosses Koil. Koil writes one trace as
one JSON file in `traces/logs/`. The file name is the trace ID. The file is
immutable after Koil writes it.

This section gives a first schema. Change it to fit your needs. But keep the field
names stable after you start, because the lookup table and any later tool depend
on them.

## Fields of one trace

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `id` | string | yes | The random trace ID. It is also the file name. |
| `time` | string | yes | The time Koil wrote the trace, in ISO 8601 (for example `2026-09-07T14:03:22Z`). |
| `session` | string | yes | The ID of the run that this trace belongs to. One run has many traces. |
| `step` | number | yes | The step number inside the session. It starts at 0 and counts up. |
| `direction` | string | yes | `to_model` or `from_model`. |
| `source` | string | yes | The part that sent the message (for example `k-core`). |
| `destination` | string | yes | The part that received the message (for example `kineserve`). |
| `endpoint` | string | for `to_model` | The `llama-server` path, for example `/completion`. |
| `model` | string | yes | The model file name from `models/`. |
| `payload` | object | yes | The exact JSON body that crossed the channel (see below). |
| `timings` | object | for `from_model` | The timing block from `llama-server`, if it is present. |
| `prev` | string or null | yes | The ID of the trace before this one in the session, or null for the first. |

The `payload` field is the key to the phrase "works with `llama-server`". For a
`to_model` trace, the payload is the request that you sent to `llama-server`, with
no change. For a `from_model` trace, the payload is the answer that `llama-server`
returned, with no change. So a trace holds the real `llama-server` body plus the
metadata around it. The `prev` field links the traces in order, so you can replay
a session from first to last.

## Example trace to the model

```json
{
  "id": "b1c4f9a2e8d74630",
  "time": "2026-09-07T14:03:22Z",
  "session": "9f2a77c0",
  "step": 0,
  "direction": "to_model",
  "source": "k-core",
  "destination": "kineserve",
  "endpoint": "/completion",
  "model": "your-model.gguf",
  "payload": {
    "prompt": "System instructions...\nUser: list the files\n",
    "n_predict": 256,
    "temperature": 0.2,
    "stream": false
  },
  "prev": null
}
```

## Example trace from the model

```json
{
  "id": "7d0e5a13c9b28f44",
  "time": "2026-09-07T14:03:24Z",
  "session": "9f2a77c0",
  "step": 1,
  "direction": "from_model",
  "source": "kineserve",
  "destination": "k-core",
  "model": "your-model.gguf",
  "payload": {
    "content": "{\"action\": \"list_files\", \"args\": {\"path\": \".\"}}",
    "stop_type": "eos"
  },
  "timings": {
    "prompt_n": 42,
    "predicted_n": 18,
    "predicted_ms": 640.5
  },
  "prev": "b1c4f9a2e8d74630"
}
```

## The lookup table `trace_hash.json`

`traces/trace_hash.json` maps each trace ID to its file. It gives quick access
without a scan of the directory. A simple shape is one object. Each key is a trace
ID. Each value holds the file path, the session, the step, and the time.

```json
{
  "b1c4f9a2e8d74630": {
    "file": "traces/logs/b1c4f9a2e8d74630.json",
    "session": "9f2a77c0",
    "step": 0,
    "time": "2026-09-07T14:03:22Z"
  },
  "7d0e5a13c9b28f44": {
    "file": "traces/logs/7d0e5a13c9b28f44.json",
    "session": "9f2a77c0",
    "step": 1,
    "time": "2026-09-07T14:03:24Z"
  }
}
```

Rules for the recorder:

- Write the trace file first. Add the lookup entry second. This order makes sure
  the table never points to a missing file.
- Do not change a trace after you write it. To correct a record, write a new trace
  and link it with `prev`.
- The ID scheme (random bytes or a hash) is a separate decision. See [decisions.md](decisions.md),
  items 4 and 5.

## Record the tool events too

The first purpose of the trace log is to find out what went wrong. That purpose
sets a requirement that the fields above do not yet meet.

Koil sees only the messages between K-Core and Kineserve. **A tool runs inside
K-Core, so a tool call never crosses Koil.** With the fields above, a tool result
survives only as text inside the next prompt. That is the wrong shape for
debugging, because "what went wrong" is often "the tool returned something
unexpected".

So let K-Core write tool events into the same chain, with the same schema. Add two
values to `direction`:

- `tool_call` — K-Core is about to run a tool. The payload holds the tool name and
  the arguments.
- `tool_result` — the tool finished. The payload holds the status, the result or
  the error, the duration, and a flag if the result was cut.

The log then holds the whole session, not only the model traffic. The `prev` chain
still puts every event in order.

## Keep the log readable by hand

Manual reading is a valid way to use this log, and it needs no extra program. Two
small choices keep it that way:

- **Write the JSON with indentation, not on one line.** A trace is then readable
  as soon as you open it.
- **Make the lookup table do the sorting.** `trace_hash.json` already holds
  `session` and `step`. Sort on those two fields and you have the file order for a
  whole session, with no parser at all.

That covers one trace and one session. A reader program only earns its place when
you want replay ([roadmap.md](roadmap.md), Phase 4), because replay must rebuild the loop state,
not just show the files.

**If you want rewind, watch this constraint.** Rewind means: return to step N and
continue differently. It works only if every input to the loop state appears
somewhere in the chain. If K-Core holds state that never reaches a trace, rewind
breaks at that point. The tool events above close the largest hole. Keep the rule
in mind as you add fields: **if it changes the loop, record it.**

---

