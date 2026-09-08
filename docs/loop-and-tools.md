# The ReAct loop and the tool layer

K-Core runs the ReAct loop. "ReAct" means reason, then act. The model reasons in
text. Then it asks for an action. K-Core runs the action, observes the result, and
gives the result back to the model. The loop repeats until a stop condition is
true.

## The step cycle

One step of the loop does these things in order:

1. Build the prompt. Join the instructions from `kinesin.toml`, the conversation
   so far, and the last observation.
2. Send the prompt to the model through Koil ([integration.md](integration.md), llama-server). Record a `to_model`
   trace.
3. Read the answer. Record a `from_model` trace.
4. Look for an action in the answer.
5. If there is no action, treat the answer as the final answer. Stop the loop.
6. If there is an action, find the function for that action name in the function
   table.
7. If the name is not in the table, make an error observation. Go to step 10.
8. Check the arguments against what the function needs. If the arguments are
   wrong, make an error observation. Go to step 10.
9. Call the function. Capture its result or its error as the observation.
10. Add the action and the observation to the conversation.
11. Add 1 to the step count. Go to step 1.

## The loop states

Model the loop with a small set of states. An `enum` fits well (Rust book,
Chapter 6):

- `Think` — build and send the prompt; wait for the answer.
- `Act` — a valid action is present; call the function.
- `Observe` — record the result and add it to the conversation.
- `Done` — a stop condition is true; return the final answer.
- `Failed` — an error stops the loop; return the error.

## How the model asks for an action

This is the most important decision in the whole harness. There are three ways to
do it. They differ a great deal in how often they work.

**Approach A — Native tool calls (recommended).** Start `llama-server` with
`--jinja`. Send your tools in the `tools` array on `/v1/chat/completions`. The
server formats the tools with the model's own chat template and parses the reply
for you. You receive `tool_calls` in the response, and `finish_reason` is
`"tool"`. **You write no parser for tool calls.**

Why this is the default choice: model makers train the model on their own tool
format. When you inject a different format into the prompt instead, the format
does not match the training, and measured hallucination rates for that mismatch
are very high. The original ReAct method used free text action lines and a regular
expression to read them, and parse failures were one of the main causes of agent
breakage. Native formats also use fewer tokens.

**Approach B — Constrained decoding (use together with A).** Send a `json_schema`
(or `grammar`) field with the request. llama.cpp turns it into a GBNF grammar and
permits only tokens that fit the shape. Malformed output becomes impossible, so a
whole class of failure disappears. Three cautions:

- The schema does not reach the model. Describe the shape in the prompt as well.
- Only a subset of JSON Schema works. Do not mix `properties` with `anyOf` or
  `oneOf`; avoid `prefixItems`, nested `$ref`, and `patternProperties`.
- `additionalProperties` defaults to false, which is what you want.

**Approach C — Hand-parsed JSON on `/completion` (fallback only).** The model
returns one JSON object in `content`, and you parse it:

```json
{
  "action": "list_files",
  "args": { "path": "." }
}
```

This is the simplest request and it works with any model, with no template
support needed. It is also the approach with the worst measured reliability, and
you own the parser forever. Use it only if your model has no tool template. If you
do use it, always add Approach B so that the JSON is at least well formed.

**The recommendation: A + B.** Use native tool calls, and constrain the output.
Keep C written down as the fallback.

> **This does not cost you the learning.** Approach A removes the fragile
> tool-call *extraction* work. It does not remove the JSON work. You still
> hand-write the encoder that builds the request and the decoder that reads the
> response, exactly as [decisions.md](decisions.md), item 2 describes. You give up a parser that
> tends to break, not the part that teaches you the most.

**Check this before you build anything.** Start the server with your model and one
test tool, and read the server log. It states whether it used a native format or
the generic fallback. If it says generic, change the model now rather than after
you write the loop. Phase 0 in [roadmap.md](roadmap.md) makes this a step.

## Stop conditions

The loop must not run forever. Stop when any of these is true:

- The answer has no action. This is the normal, successful end.
- The step count reaches the maximum. Set the maximum in `kinesin.toml`, for
  example 12 steps.
- The same action and the same arguments repeat too many times. This shows a
  stuck loop.
- A function returns a stop result on purpose (for example a `finish` function).
- An error happens that you cannot recover from (for example Kineserve is not
  reachable).

## What to record

Record a trace for every message to and from the model ([traces.md](traces.md)). Also decide
whether to record the function results. A good first rule: put the observation
text inside the next `to_model` trace, because the observation becomes part of the
next prompt. This keeps the full history in the traces.

## Errors in the loop

Return a `Result` from each function (Rust book, Chapter 9). Turn a function error
into an observation, not a crash, so the model can react to it. Stop the loop only
for an error that you cannot recover from. For that case, write a `Failed` trace
with the reason.

Rust study for this section: Chapter 5 (structs for the message and the action),
Chapter 6 (enums and `match` for the states), Chapter 9 (`Result` and `?`), and
Chapter 13 (iterators to walk the conversation). Read the Brown fork, Chapter 4.3
"Fixing Ownership Errors", before you pass the conversation between functions.

---

## How to define a tool

A tool has four parts. Keep the shape below, because it maps straight into the
`tools` array that `llama-server` expects, and it also matches the shape that the
Model Context Protocol uses. That keeps a later move to MCP a mapping job instead
of a rewrite.

| Part | What it is |
|------|-----------|
| `name` | A short, action-based name, for example `read_file`. |
| `description` | Plain words that say what the tool does and when to use it. |
| `parameters` | A JSON Schema for the arguments. |
| handler | The Rust function that runs the tool. |

Rules that come from measured practice:

- **The description carries as much weight as the schema.** The model chooses the
  tool from the description. Write it for a reader who cannot see your code. Say
  when *not* to use the tool as well.
- **Set `additionalProperties` to false.** List `required` fields explicitly. Use
  an enum where the value set is fixed. A tight schema removes guesswork.
- **Give a tool one clear purpose.** Too broad, and the model picks it for the
  wrong job. Too narrow, and you need many tools, which also confuses the model.
- **Keep the tool count small.** Every extra tool makes the choice harder. Start
  with three.
- **Give every tool result the same envelope.** For example a status, a body, and
  a flag that says whether the body was cut. One shape makes the loop simple and
  makes the traces easy to read.

## What goes wrong, and what to do about it

Local models fail at tool calls in four repeatable ways. Plan for each one.

| Failure | What it looks like | What to do |
|---------|-------------------|-----------|
| Eager invocation | The model calls a tool when it does not need one. It answers "Hello" with a tool call. | Say in the instructions that a plain answer is allowed. Give an explicit `finish` tool. |
| Wrong tool | It picks a tool that does not fit the task. | Improve the descriptions. Reduce the tool count. |
| Invalid arguments | A field is missing, or has the wrong type. | Use constrained decoding (8.3, Approach B). Check the arguments before you call the handler, and return the error as an observation. |
| Ignored result | The model does not use the tool output and repeats the call. | Put the result in the conversation in a clear form. Use the `repeat_limit` from "Stop conditions" above. |

Two more points that decide success before you write any code: use a model of at
least 8B parameters, and keep the KV cache unquantized. [components.md](components.md), models/ gives the
detail on both.

## Safety: bound what the model can do

The model chooses the actions, so the harness must set the limits. This is a
requirement, not a feature.

- **Read-only by default.** A tool that changes data goes on an explicit
  allow-list, and asks for a confirmation first.
- **Validate arguments before you run the handler.** Never pass model output
  straight into a path, a command, or a query.
- **Treat every tool result as untrusted text.** A file, a web page, or another
  agent can carry instructions aimed at your model. Do not let a tool result act
  as an instruction. Keep it clearly marked as data when you put it back in the
  prompt.
- **Cap the size of a result** before it enters the conversation.

## Two approaches considered and not chosen

Design rule 6 says to write down the reason. These two are worth knowing about.

**Code as action (CodeAct).** Instead of a JSON action, the model writes a short
program, and the harness runs it. Measurements show it beats JSON actions: about a
20% better success rate and roughly 30% fewer steps, because code carries loops
and conditions, and models see a lot of code in training. The gain is real and it
is largest for open models. **Not chosen** because it needs a sandboxed
interpreter. That is a large dependency and a large security surface, and both
work against the minimal, standard-library goal. Revisit only if Kinesin ever
needs to compose many tools in one step.

**Model Context Protocol (MCP).** MCP is now the common standard for tool
interfaces, with wide industry support. **Not chosen for now** because Kinesin
needs a working local loop first, and MCP adds a protocol and a transport that the
harness does not yet need. The cheap step is the one in "How to define a tool" above: shape the
tool definition like an MCP tool. Then adopting MCP later is a mapping, not a
redesign.

---

