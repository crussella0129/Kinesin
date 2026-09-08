# First steps

You have done Rustlings. Rustlings teaches the language. It does not teach how to
work in a real project. That gap is what makes a first project feel hard, and it
is smaller than it looks.

Do the ten steps below in order. Steps 1 to 8 use a throwaway project, not
Kinesin. That is on purpose. Each step teaches one thing you will need later, in a
place where nothing can break.

**What this document leaves out.** It names the commands and the chapters. It does
not give you the Rust. You write that part. If a step tells you what to build but
not how, that is the design of the document, not an omission.

---

## Step 1 — Confirm the tools, and find the offline documents

**Do this.** Confirm that `rustup`, `cargo`, and `rustc` answer with a version.
Add the two tools you will use every day: the formatter and the linter. Then open
the offline documents with `rustup doc`.

**Done when.** All three commands report a version, and the documents open in your
browser.

**Why it matters.** `rustup doc` gives you the book, the standard library
reference, and more, on your own machine, with no network. You are working without
an assistant, so these pages are your first answer to most questions.

---

## Step 2 — Make a scratch project and learn the daily loop

**Do this.** Make a new binary project somewhere outside the Kinesin folder. Call
it `scratch`. Run it. Then learn what these four commands do, and how they differ:

- `cargo check`
- `cargo build`
- `cargo run`
- `cargo test`

**Done when.** You can say which of them compiles without producing a program, and
which one you will use most.

**Why it matters.** `cargo check` is much faster than `cargo build`. Most of your
work is a loop of "edit, check, read the error, edit". Learn to lean on the fast
one.

**Read.** Rust book, Chapter 1.3 "Hello, Cargo!".

---

## Step 3 — Break things on purpose and read the errors

**Do this.** In `scratch`, cause each of these on purpose, one at a time, and read
the whole message before you fix it:

- use a value after you move it;
- borrow a value as mutable while an immutable borrow is alive;
- return the wrong type from a function;
- forget to handle a `Result`.

When the error has a code, for example `E0382`, run `rustc --explain E0382` and
read the explanation.

**Done when.** You can point at the part of an error message that says *what to
do*, not just *what is wrong*.

**Why it matters.** This is the single most useful skill for the way you have
chosen to work. The Rust compiler explains itself better than most. The message is
long because it is teaching, not because it is angry. Read from the bottom: the
`help:` and `note:` lines are usually the answer.

**Read.** The Brown University fork of the Rust book, Chapter 4.3 "Fixing
Ownership Errors": https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html
That section exists for exactly this step. Chapter 4.5 gives the Read/Write/Own
model that makes borrow errors make sense.

---

## Step 4 — Learn to search the standard library documents

**Do this.** Open the standard library documents. Find the answers to three
questions by reading only the docs:

- What does `String::from_utf8` return, and why is it not just a `String`?
- What is the difference between `Vec::get` and the index form?
- What does `std::fs::read_to_string` give you when the file is missing?

**Done when.** You can read a function signature and say what it takes, what it
gives back, and what can go wrong.

**Why it matters.** Every question you would have asked an assistant, you will now
answer here. The signature usually tells you most of it.

---

## Step 5 — Put code in a second file

**Do this.** In `scratch`, add a second source file. Call a function in it from
`main`. Make one function public and one private, and see what happens when you
call the private one from `main`.

**Done when.** You can explain what `mod`, `use`, and `pub` each do, and where
Rust looks for the file.

**Why it matters.** Rustlings is one file at a time. Kinesin is many files across
four crates. This is the step that bridges the two, and it takes about twenty
minutes.

**Read.** Rust book, Chapter 7, especially 7.2, 7.4, and 7.5.

---

## Step 6 — Write a test, and watch it fail first

**Do this.** Write a small function in `scratch` that does something with a
`String`. Then write a test for it in the same file. **Make the test fail on
purpose first.** Read the failure output. Then make it pass.

**Done when.** You have seen both a red and a green `cargo test` run, and you know
where the test code goes and why it does not ship in the release build.

**Why it matters.** A test you have never seen fail is not evidence of anything.
Get used to the failure output now, while the code is trivial.

**Read.** Rust book, Chapter 11.1. Then [testing.md](testing.md), section 1, for
how this applies to Kinesin.

---

## Step 7 — Read a file from disk

**Do this.** In `scratch`, read a text file that you create by hand, and print it.
Then run it again with the file deleted, and see what happens.

**Done when.** Your program handles the missing file without a panic, and prints
something useful instead.

**Why it matters.** This is the first real work that Kinesin needs, and it is the
first time a `Result` is not optional. Every part of the harness reads something.

**Read.** Rust book, Chapter 12.2.

---

## Step 8 — Replace `unwrap` with `?`

**Do this.** Take the code from step 7. It probably calls `unwrap` or `expect`.
Change the function to return a `Result`, and use the `?` operator instead. Then
decide what `main` does with the error.

**Done when.** No `unwrap` remains, and you can say in one sentence what `?` does.

**Why it matters.** `unwrap` is fine while you learn. It is not fine in a harness
that must keep a record of what went wrong. Doing this conversion once, on code
you already understand, is much easier than learning it later under pressure.

**Read.** Rust book, Chapter 9.2.

---

## Step 9 — Talk to `llama-server` from Rust

**Do this.** First finish Phase 0 in [roadmap.md](roadmap.md), so the server runs
and answers by hand. Then, in `scratch`, open a TCP connection to it, ask the
`/health` endpoint, and print the raw bytes that come back. Do not parse anything
yet. Just print it.

**Done when.** You see the raw HTTP response text in your terminal.

**Why it matters.** This is the real core of Kinesin in its smallest form. Once
you have seen the raw response, the rest of the harness is shaping and recording
what you already know how to fetch.

**Expect this one to take longer than the others.** You must write the request
exactly: a request line, the headers, then a blank line before anything else. A
missing blank line is the usual reason nothing comes back. That is normal, and it
is not a sign you are behind.

**Read.** Rust book, Chapter 21.1, for the same read-and-parse pattern on a
socket. Then MDN "HTTP Messages" for what a correct request looks like.

---

## Step 10 — Build the real workspace

**Do this.** Now open the Kinesin folder. Make the workspace and the member
crates, as [roadmap.md](roadmap.md) Phase 1 describes. Move what you learned in
`scratch` into it. Keep `scratch` — it stays useful as the place to try one thing
without breaking the project.

**Done when.** `cargo build` succeeds at the workspace root, and Phase 1, step 1
is complete.

**Read.** The Cargo Book, "Workspaces". Then follow
[roadmap.md](roadmap.md) from Phase 1, step 2.

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

**Ownership, when you pass data between functions.** Expected. Read the Brown fork
Chapter 4 again once you have real code to look at. It reads very differently the
second time.

**Lifetimes, when you store a reference in a struct.** The best advice for now:
**do not.** Give your structs owned data. Use `String`, not `&str`. Clone when you
are unsure. This costs a small amount of speed that you will not notice, and it
removes almost every lifetime error a first project hits. Make it efficient later,
when the harness works and you can measure. This is not cheating; it is the normal
order of work.

**Error types, when `?` will not convert.** You will meet this in step 8. The
short version is that `?` needs a way to turn one error type into another. Read
Chapter 9 for how, and do not be surprised if your first attempt is clumsy.

---

## One last thing

You are more ready than you feel. The design work for this project is already
done, and it is written down: the decisions, the schema, the loop, and the order
to build in. That is usually the part that stops people. What remains is typing
Rust, one small piece at a time, with a compiler that tells you when you are
wrong.

Steps 1 to 8 should take one or two sittings. They are meant to be easy. The point
is that by the time you reach the real project, the tools are boring and the
compiler is a colleague rather than a wall.
