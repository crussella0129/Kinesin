# Architecture and design

## What Kinesin is

Kinesin has three parts:

- **Kineserve** starts a local model server and points it at your model file. It
  uses `llama-server` from the llama.cpp project.
- **Koil** is the channel between the harness and the model server. It also
  records every message that crosses it.
- **K-Core** is the brain. It runs the ReAct loop (reason, then act), it routes
  function calls, and it reads your configuration and instructions.

**Where it runs.** Kinesin splits the control work from the compute work. K-Core
is small and can run anywhere. Kineserve needs a machine with enough memory and a
good GPU. The two do not need to be the same machine.

- **Now (local).** All three parts run on one machine. Koil passes messages
  across `127.0.0.1`. One Koil is enough. This is the build target for Phase 1.
- **Later (distributed).** Kineserve runs on a bigger machine somewhere else, and
  serves more sessions at the same time. A second Koil runs beside it. The two
  Koils hold a private link between them, wherever each machine is.

Koil is the same component in both cases. The local setup is the simple case of
the same design, not a different design. [roadmap.md](roadmap.md), Phase 5 covers the
distributed step.

> **Note:** In this document, "user" means a human **or** another agent. Both send
> input to Kinesin in the same way.

The name is a metaphor only. A kinesin is a motor protein that carries cargo
along a track. Kinesin the program carries messages between the harness and the
model. Do not read more into the name than that.

The design rules are in the [README](../README.md). The rule that matters most
here is rule 6: write down the reason next to the choice. The Koil entry in
[components.md](components.md) is the worked example.

---

## Architecture and data flow

**Local (Phase 1, build this now).** One machine. One Koil. No tunnel.

```
K-Core  <->  Koil  <->  Kineserve  <->  llama-server  <->  model.gguf
(ReAct       (channel   (supervisor    (HTTP server      (your GGUF
 loop +       + trace     for the        from             text model)
 routing)     record)     model server)  llama.cpp)
```

**Distributed (Phase 5, later).** Two machines. Two Koils. A private link between
them.

```
  your machine                    |        the bigger machine
                                  |
K-Core  <->  Koil  <===============|===>  Koil  <->  Kineserve  <->  llama-server
(ReAct       (local     private    |      (remote    (supervisor)
 loop)        endpoint)  link      |       endpoint)
                                   |
```

K-Core sees the same interface in both pictures. It sends a request to Koil and
receives an answer. It does not know whether the bytes cross loopback or a
tunnel. [components.md](components.md), Koil explains why this matters more than anything else you build
first.

The message flow for one step is:

1. K-Core builds a request from the instructions and the conversation so far.
2. K-Core sends the request through Koil to Kineserve.
3. Kineserve passes the request to `llama-server` over local HTTP.
4. `llama-server` returns the model's answer.
5. Koil records the request and the answer as traces.
6. K-Core reads the answer. If the answer asks for a function, K-Core calls the
   function and adds the result to the conversation.
7. The loop repeats until a stop condition is true.

**Traces** are the audit log. Each trace is one JSON file in `traces/logs/`. The
file name is a random ID, not a sequential number. `traces/trace_hash.json` is a
lookup table that maps each ID to its file. [traces.md](traces.md) gives the trace schema.
[decisions.md](decisions.md), items 4 and 5, give the ID decision.

---

