# First steps

You have done Rustlings. Rustlings teaches the language. It does not teach how to
work in a real project. That gap is what makes a first project feel hard, and it
is smaller than it looks.

There is one warm-up sitting, then ten steps. **Each of the ten steps builds a
real part of Kinesin.** Every one of them ends with something that runs. When you
finish step 10, Phase 1 of [roadmap.md](roadmap.md) is complete.

**What this document leaves out.** It names the commands, the `std` modules, and
the chapters to read. It does not give you the Rust. You write that part. If a
step tells you what to build but not how, that is the design of the document, not
an omission.

---

## Step 0 — Warm-up (one sitting, before the real work)

Do these four things in a throwaway project, not in Kinesin. Make a new binary
project somewhere else and call it `scratch`. Keep it afterwards. It stays useful
as the place to try one thing without breaking the project.

**A. Confirm the tools.** Check that `rustup`, `cargo`, and `rustc` answer with a
version. Add the formatter and the linter. Then open the offline documents with
`rustup doc`. Those pages are the book and the whole standard library reference,
on your machine, with no network. They are your first answer to most questions.

**B. Learn the daily loop.** Find out what `cargo check`, `cargo build`,
`cargo run`, and `cargo test` each do, and how they differ. `cargo check` is much
faster than `cargo build`. Most of your work is a loop of "edit, check, read the
error, edit". Lean on the fast one. *(Rust book 1.3.)*

**C. Break things on purpose and read the errors.** This is the part not to skip.
Cause each of these deliberately, one at a time, and read the whole message before
you fix it:

- use a value after you move it;
- borrow a value as mutable while an immutable borrow is alive;
- return the wrong type from a function;
- ignore a `Result`.

When an error has a code such as `E0382`, run `rustc --explain E0382`.

**Why this one matters most.** You have chosen to work without an assistant. The
compiler is the thing that takes its place, and it explains itself better than
most. The message is long because it is teaching. **Read it from the bottom:** the
`help:` and `note:` lines are the answer more often than the first line is.
*(Read the Brown fork, Chapter 4.3 "Fixing Ownership Errors":*
https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html *— that
section exists for exactly this.)*

**D. Search the standard library documents.** Answer three questions by reading
only the docs: what does `String::from_utf8` return and why is it not just a
`String`; how does `Vec::get` differ from the index form; what do you receive from
`std::fs::read_to_string` when the file is missing. You want to reach the point
where a function signature tells you what it takes, what it returns, and what can
go wrong.

**Done when.** The four commands are boring, you have read a real borrow error to
the end, and you can find a type in the documents without help.

---

## Step 1 — Prove the environment

**Do this.** No Rust in this step. Work by hand, exactly as Phase 0 of
[roadmap.md](roadmap.md) describes: build or install `llama-server`, put a model
of 8B parameters or more in `models/`, start the server with `--jinja`, check
`/health`, send one chat request, send one request with a `tools` array, and read
the server log to confirm it uses a **native** tool format and not the generic
fallback.

**Save the request bodies that worked, and the exact command line.** Put them in a
file you keep. You will use them again in step 6 as the target your code must
reproduce, and again in step 8 as the thing you send.

**Done when.** You have received an answer from the model by hand, and you have a
saved request body that works.

**Why first.** If the model cannot do tool calls, that changes the design of the
whole harness. Finding out now costs an afternoon. Finding out after step 10 costs
the loop.

---

## Step 2 — Make the workspace run

**Do this.** In the Kinesin folder, create the Cargo workspace: a root manifest
that lists members, a binary crate for `k-core`, and a library crate for `common`.
Define one small type in `common` and use it from `k-core`. Print something.

**Done when.** `cargo build` succeeds at the workspace root, `cargo run` prints
your line, and the type it prints is defined in the other crate.

**What this teaches.** The difference between a workspace and a package, the
difference between a library crate and a binary crate, and what `pub` controls.
Kinesin has four crates, so this is the shape you work in from now on.

**Read.** The Cargo Book, "Workspaces". Rust book, Chapter 7, especially 7.2 and
7.5.

---

## Step 3 — Read `kinesin.toml` and split it

**Do this.** Write `kinesin.toml` by hand first, using the skeleton in
[configuration.md](configuration.md). Then make `k-core` read the file as text and
split it at the `+++ instructions +++` divider. Print the two halves separately.
Do not parse the settings yet.

Then run it again with the file renamed, so the file is missing.

**Done when.** It prints the settings half and the instructions half separately,
and it handles the missing file without a panic.

**What this teaches.** `std::fs`, and the first time a `Result` is not optional.
If you reached for `unwrap`, change it now: make the function return a `Result`
and use the `?` operator instead. Decide what `main` does with the error.

**Read.** Rust book, Chapter 12.2 (read a file) and Chapter 9.2 (`?`).

---

## Step 4 — Test what you wrote, and make it fail first

**Do this.** Write unit tests for the splitter from step 3, in the same file, in a
`#[cfg(test)] mod tests` block. **Make one fail on purpose first** and read the
output. Then make it pass.

Cover the awkward cases: no divider in the file; the divider on the first line;
extra blank lines around it; an empty instructions half.

**Done when.** You have seen `cargo test` red and then green, and you know why the
test code does not ship in the release build.

**Why here, and not later.** The splitter is a pure function: text in, text out.
It touches no socket and no clock, so testing it is easy. Start the habit on the
easy case. From now on, when you write something pure, write its tests with it.

**Read.** Rust book, Chapter 11.1. Then [testing.md](testing.md), sections 1 and 2.

---

## Step 5 — Parse the settings

**Do this.** Turn the settings half from step 3 into a real settings type in
`common`. Support only what the skeleton uses: section headers, and keys with
string, number, and boolean values. Ignore comments and blank lines.

Write the tests first this time. You know the rules before the code exists.

**Done when.** Your program reads `kinesin.toml` and prints the model path, the
port, and `max_steps` from the parsed type, not from the raw text. Every setting
has a default, so an older file still works.

**Read.** Rust book, Chapter 8.2 (strings) and Chapter 13 (iterators). The TOML
specification for the small subset you support: https://toml.io/en/v1.0.0

---

## Step 6 — Build the JSON request

**Do this.** Write the JSON encoder by hand, in `common`. Then use it to build the
request body for the chat endpoint.

**Your target is the body you saved in step 1.** Make your code produce it. Write
the tests first, from the JSON rules.

**Done when.** Your encoder produces a body that matches the one that worked by
hand. Strings with quotes, backslashes, and newlines are escaped correctly.

**Why this is the right place to try test-first work.** The rules of JSON are
fixed and known before you write a line. That is the ideal case for writing the
test first, and you have a real target to check against.

**Read.** The JSON grammar: https://www.json.org/ It is smaller than you expect.

---

## Step 7 — Ask the server whether it is healthy

**Do this.** Open a TCP connection to `llama-server` and ask the `/health`
endpoint. Print the raw bytes that come back. Do not parse anything.

**Done when.** You see the raw HTTP response text in your terminal.

**Expect this to take longer than the steps before it.** You must write the
request exactly: the request line, the headers, then **a blank line**. A missing
blank line is the usual reason nothing comes back. That is normal, and it is not a
sign that you are behind.

Start with `/health` and not the real request, because it is a GET with no body.
There is no `Content-Length` to get wrong yet.

**Read.** Rust book, Chapter 21.1, for the same read-and-write pattern on a
socket. MDN "HTTP Messages" for the shape of a correct request:
https://developer.mozilla.org/docs/Web/HTTP/Messages

---

## Step 8 — Send the real request and see an answer

**Do this.** Now POST the body you built in step 6. This one has a body, so you
must send `Content-Type` and a `Content-Length` that is the exact byte length.
Print the raw response.

**Done when.** The model's answer appears in your terminal, produced entirely by
your own code.

**This is the milestone.** Everything after it is shaping and recording something
you already know how to fetch.

**A warning.** `Content-Length` counts bytes, not characters. If your prompt has
any character outside ASCII, a length in characters is wrong and the server will
wait for data that never comes.

---

## Step 9 — Read the answer properly, and record it

**Do this.** Two parts.

First, parse the response: find the blank line that ends the headers, take the
body after it, and decode enough JSON to reach the answer text. Print only that
text, cleanly.

Second, write your first trace. Use the schema in [traces.md](traces.md). Write
one file for the request and one for the answer, into `traces/logs/`, and add both
to `trace_hash.json`. Write the file first and the index entry second. Indent the
JSON so that you can read it.

**Done when.** Your program prints a clean answer, and you can open the two trace
files by hand and read what was sent and what came back.

**Why both in one step.** The decoder and the trace writer are the two halves of
the same idea: understand the answer, then keep a record of it. This is the first
moment the harness can tell you what it did.

---

## Step 10 — Make it a harness

**Do this.** Two changes that turn a program into a harness.

First, start the server yourself. Use `std::process::Command` to launch
`llama-server` with the settings from `kinesin.toml`, then poll `/health` with
your code from step 7 until it answers. This is Kineserve.

Second, put the sending behind a trait. Everything that sends a request to
Kineserve goes through one interface, with one implementation for now: direct,
over `127.0.0.1`. This is the Koil transport seam.

**Done when.** You run one command, your program starts the server, waits for it,
sends a request, prints the answer, and writes the traces. You did not start
anything by hand.

**Take care with the seam.** It is the one piece of Phase 1 that is expensive to
change later. `k-core` must not know whether the bytes cross loopback or a tunnel.
Get that right and Phase 5 adds the remote machine without touching `k-core`. Read
the Koil entry in [components.md](components.md) before you write it.

**Read.** Rust by Example, "Child processes":
https://doc.rust-lang.org/rust-by-example/std_misc/process.html

---

## After step 10

Phase 1 is complete. Go to [roadmap.md](roadmap.md), Phase 2, and read
[process.md](process.md) for what "done" means before you move on.

---

## How to get unstuck without an assistant

In order, these usually work:

1. **Read the whole error, from the bottom.** The `help:` and `note:` lines are
   the answer more often than the first line is.
2. **Run `rustc --explain` on the error code.**
3. **Look up the type in the standard library documents.** Read the signature.
4. **Make the smallest example that still fails.** Cut everything else away. Half
   the time the answer appears while you cut.
5. **Write a comment that says what you expect to happen.** Being exact often
   shows you the wrong assumption.
6. **Stop for a while.** Borrow errors are unusually responsive to a night of
   sleep. This is not a joke.
7. **Ask a person.** The Rust users forum is patient with beginners:
   https://users.rust-lang.org/

---

## Three places you will get stuck, and what to do

**Ownership, when you pass data between functions.** Expected, and it starts
around step 5. Read the Brown fork, Chapter 4, again once you have real code to
look at. It reads very differently the second time.

**Lifetimes, when you store a reference in a struct.** The best advice for now:
**do not.** Give your types owned data. Use `String`, not `&str`. Clone when you
are unsure. This costs a small amount of speed that you will not notice, and it
removes almost every lifetime error that a first project meets. Make it efficient
later, when the harness works and you can measure it. This is not cheating. It is
the normal order of work.

**Error types, when `?` will not convert.** You will meet this around step 5, when
one function can fail in two different ways. `?` needs a way to turn one error
type into another. Read Chapter 9 for how, and do not be surprised if your first
attempt is clumsy. Everybody's is.

---

## One last thing

You are more ready than you feel. The design work for this project is already
done, and it is written down: the decisions, the schema, the loop, and the order
to build in. That is usually the part that stops people.

What remains is typing Rust, one small piece at a time, with a compiler that tells
you when you are wrong. Ten steps, each one ending with something that runs.
