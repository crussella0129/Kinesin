# Development process

This document takes the useful parts of a software development life cycle and
keeps only what helps one person. Most process exists to keep many people in
agreement. You do not need that part. You do need the parts that catch mistakes
and that keep a record of your reasons.

---

## 1. The cycle for one phase

Each phase in [roadmap.md](roadmap.md) goes through the same five steps.

```
Ready  ->  Build  ->  Prove  ->  Record  ->  Merge
```

- **Ready** — the decisions the phase needs are settled.
- **Build** — write the code in small commits.
- **Prove** — the tests for the phase pass.
- **Record** — write down any decision you made and the reason.
- **Merge** — the branch goes to `main`, and you tag it.

---

## 2. Definition of ready

Do not start a phase until all three are true:

1. Every decision that the phase needs is settled in
   [decisions.md](decisions.md).
2. You can say in one sentence what the phase produces.
3. You know how you will prove that it works.

Point 3 is the one people skip. If you cannot say how you will prove it, you do
not yet understand the phase.

---

## 3. Definition of done

A phase is done when all of these are true:

- The code does what the phase goal says.
- Unit tests cover the pure core that the phase added.
- At least one integration test proves the phase from end to end.
- `cargo fmt --check`, `cargo clippy`, and `cargo test` all pass.
- The documents are updated where the phase changed something.
- Any new choice has its reason written next to it (design rule 6).

**What proves each phase:**

| Phase | The proof |
|-------|-----------|
| 0 | The saved request bodies work by hand, and the server log reports a native tool format |
| 1 | Config loads from a real file; a trace round trip rebuilds a session; one real answer arrives |
| 2 | Unit tests cover the loop state machine; the loop runs end to end against a fake model |
| 3 | The context limit and the size caps have tests; a long run finishes without a break |
| 4 | Replay of a recorded session produces the same actions |
| 5 | The harness passes the same tests with the remote transport as with the local one |
| 6 | The golden traces pass, and CI is green on every push |

---

## 4. Branches and tags

You work alone, so keep this light. Do not make it sloppy.

- One branch for each phase, for example `phase-1-foundations`.
- Small commits inside the branch.
- Merge to `main` only when the definition of done is met.
- Tag `main` at the end of each phase, for example `phase-1`.

**Why.** `main` then holds only work that passed its own test. Your history
becomes a list of states that each worked. That is what makes `git` useful to you
in the same way that the trace chain is useful: you can go back to a known point.

---

## 5. Commits

- One logical change for each commit.
- Write the subject in the imperative: "Add the trace writer", not "Added" or
  "Adding".
- Keep the subject short. Put the detail in the body.
- The body says **why**. The diff already says what.
- Never push a broken build to `main`.

---

## 6. Decision records

Design rule 6 says to write down the reason. Make it a habit with one file, not a
system.

Use [decisions.md](decisions.md) as the single log:

- An open item lists the choices and a suggested start.
- When you decide, mark the item settled, and write the reason and the date.
- **Never delete a settled item.** A decision you later reverse is still useful,
  because the reason tells you what you knew at the time.

This is an Architecture Decision Record in its smallest useful form. The Koil
entry is the worked example: the tunnel looked like too much work until the reason
was written down, and then it was clearly correct.

---

## 7. CI is the gate

Run three commands on every push:

```
cargo fmt --check
cargo clippy
cargo test
```

Treat a clippy warning as work to do, not as noise. It is the closest thing you
have to a second reader.

CI runs the fast tests only. The tests marked `#[ignore]`, which need a real
model, stay manual. See [testing.md](testing.md).

---

## 8. What to skip, and why

These are normal in a team and pointless for one person. They exist to keep people
in agreement, and there is one of you.

- Sprints, story points, and stand-ups.
- A formal review board.
- A changelog, until somebody other than you uses the harness.
- A branch protection rule that only you would approve.

**What replaces code review:** read your own diff before you commit. Let `clippy`
and the tests be the second reader. A day between writing and reading also works
well, and costs nothing.

---

## 9. Traceability

Keep a straight line from a goal to the proof:

```
phase goal  ->  the tests that prove it  ->  the commit that closed it  ->  the tag
```

When you return to this project after a break, that line tells you where you
stopped and what was true when you stopped.

---

## Study

- **The Cargo Book, "cargo fmt", "cargo clippy", "cargo test":**
  https://doc.rust-lang.org/cargo/commands/
- **GitHub Actions quickstart** (the CI gate in section 7):
  https://docs.github.com/actions/quickstart
- **Rust API Guidelines** — useful when you decide what to make `pub`, which is
  the same decision as what an integration test can reach:
  https://rust-lang.github.io/api-guidelines/
