# Testing

Kinesin runs a language model, and a language model is not reliable. You cannot
test the model. So you must be able to test everything around it. That is the
reason this document exists: the harness is the part that must be trustworthy.

This document explains what a unit is in Rust, how to mark the units in Kinesin,
how to write the tests, and how to test the parts that touch a real model.

---

## 1. What a "unit" is in Rust

Rust puts tests in three places. The rules decide how you lay out your files, so
learn them before you write the first test.

- **Unit tests** live in the same file as the code they test, at the bottom, in a
  `#[cfg(test)] mod tests` block. They can reach private functions. The
  `#[cfg(test)]` mark keeps the test code out of the release build.
- **Integration tests** live in a `tests/` directory at the crate root. Each file
  there becomes its own test program. They see **only the public API** of the
  crate, exactly as another crate would see it.
- **Doc tests** are the examples inside `///` comments. Cargo compiles and runs
  them. They cannot go stale, because a wrong example fails the build.

**The consequence to remember:** in Rust, the unit boundary and the public API
boundary are the same decision. If you want an integration test to reach an item,
that item must be `pub`. So "how do I mark units" is the same question as "what do
I make public".

---

## 2. How to mark the units in Kinesin

Use one rule: **separate pure logic from input and output.**

A pure function takes values and returns values. It reads no file, opens no
socket, starts no process, and does not ask the time. A pure function is simple to
test, because the same input always gives the same output.

So build every part in two layers:

- a **core** of pure functions — test these closely with unit tests;
- a **shell** that does the input and output and calls the core — test this
  lightly with integration tests.

This is the pure core of Kinesin. Each row is a unit worth its own tests.

| Unit | Input to output | Why it needs tests |
|------|-----------------|--------------------|
| JSON encoder | value to text | Everything depends on it |
| JSON decoder | text to value | It must survive bad input without a panic |
| TOML subset reader | text to settings | The same |
| Config splitter | file text to settings and instructions | The divider rule has edge cases |
| HTTP request builder | parts to request bytes | `Content-Length` must be exact |
| HTTP response parser | bytes to status and body | It must find the blank line correctly |
| Trace writer | trace to JSON text | The audit record must be correct |
| Tool argument check | schema and arguments to ok or error | This is a safety boundary |
| **Loop state change** | state and event to new state and action | The heart of the harness |

The last row is the important one. The next section explains it.

---

## 3. Make the loop a pure function

Write the ReAct loop as a pure function: it takes the current state and one event,
and it returns the new state and the next action. The shell around it does the
sending and the writing.

**Then you can test the whole loop with no model, no server, and no network.**

This one choice gives you three things at the same time:

- **Unit tests** for the loop. Feed it an event, and check the action it returns.
- **Replay** ([roadmap.md](roadmap.md), Phase 4). Feed it the recorded events from
  a trace file.
- **Rewind** ([traces.md](traces.md)). Restore the state at step N and continue.

Testing, replay, and rewind are the same property. Design for it once, in Phase 1,
and you receive all three.

Cases to cover for the loop:

- the normal path, where the model answers with no action;
- a valid tool call;
- a tool that returns an error;
- a tool name that is not in the table;
- arguments that do not match the schema;
- the step limit is reached;
- the repeat limit is reached;
- the answer cannot be parsed.

---

## 4. Test doubles with no libraries

There is no mocking library in `std`, and you do not need one. The Rust answer is
a trait with two implementations: the real one, and a fake one for tests.

Put a trait at each boundary:

| Trait | Real | Fake for tests |
|-------|------|----------------|
| `ModelClient` | Sends HTTP to Kineserve | Returns answers from a fixed list |
| `Transport` (the Koil seam) | Loopback or tunnel | Records what was sent |
| `TraceSink` | Writes files | Collects traces in a `Vec` |
| `Clock` | Reads the system time | Returns a fixed time |

A fixed `Clock` matters more than it looks. With it, two traces of the same run
compare exactly, so a golden test can use a plain string comparison.

**Note what happened here.** The seams you need for tests are the same seams you
need for the distributed build. The transport seam from Phase 1 is also the test
seam. Good structure and testability are the same thing in this project.

---

## 5. Integration tests

Put these in a `tests/` directory inside each crate. They use only the public API.

Three are worth writing early:

1. **Config load.** A real file on disk gives the expected settings and the
   expected instructions.
2. **Trace round trip.** Write traces to a temporary directory. Then rebuild the
   session from the lookup table and the `prev` chain. This proves the audit log
   works, which is the point of the whole trace design.
3. **The loop with a fake model.** No network, but the real loop, the real tools,
   and the real trace writer.

---

## 6. The tests that need a real model

You cannot run an 8B model in CI. Keep these tests, but keep them out of the
normal run.

- Mark them with `#[ignore]`. Then `cargo test` skips them, and
  `cargo test -- --ignored` runs them.
- Run them by hand after any change to the Kineserve link, and before you call a
  phase done.
- These tests prove that the Phase 0 findings still hold: the server starts, the
  model answers, and the tool format is still native.

---

## 7. Golden traces

Once replay works (Phase 4), record real sessions and keep them as fixtures.
Replay each one after a change and compare the result. A difference means the
behavior changed. That is either a fault or a change you meant to make, and either
way you want to know.

Keep them small and keep them few. A golden test that nobody understands gets
deleted instead of repaired.

---

## 8. The shape of the suite

Many fast pure tests. Fewer integration tests. Very few tests that use a real
model.

If `cargo test` becomes slow, the shape is wrong. Usually it means logic that
should be pure is sitting in the shell, where a test must do input and output to
reach it.

---

## 9. Practical notes

- `assert_eq!` needs `#[derive(Debug, PartialEq)]` on your own types.
- A test function can return `Result`, so you can use `?` instead of `unwrap`.
- Use `#[should_panic]` only for cases that must panic. Prefer a returned error.
- Name a test for the behavior it checks: `rejects_trailing_comma`, not
  `test_json_2`. The name is what you read when it fails.
- **Write the tests first for the JSON codec.** The rules of JSON are known before
  your code exists, so you can write the tests from the specification. This is the
  best place in the project to try test-first work.

---

## Study

- **Rust book, Chapter 11 "Writing Automated Tests":**
  - 11.1 How to Write Tests (`assert!`, `assert_eq!`, `#[should_panic]`)
  - 11.2 Controlling How Tests Are Run (`--ignored`, single-thread runs)
  - 11.3 Test Organization (unit versus integration, and the `tests/` directory)
- **Rust book, Chapter 12.4** — test-driven development in the I/O project. This
  is the closest worked example to the JSON codec task.
- **The Cargo Book, "cargo test":**
  https://doc.rust-lang.org/cargo/commands/cargo-test.html
- **The rustdoc book, "Documentation tests":**
  https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html
