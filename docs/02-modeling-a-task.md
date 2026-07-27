# Chapter 02 — Modeling a Task: The First Real Code

> This chapter assumes only `docs/00-roadmap.md` and `docs/01-introduction.md`, plus
> the design artifacts you produced in Chapter 01 (`docs/glossary.md`,
> `docs/scope.md`, and ADR `0001`). Future chapters are not written yet and will not
> be referenced in detail here.
>
> Chapter 01 was deliberately code-free: we built vocabulary and a mental model. This
> chapter is where OxideQueue stops being prose and becomes a program. But notice the
> restraint: we are still only building **Version Zero** — single process, single
> thread, in memory, a "list plus a loop." The goal is not to build much; it is to
> build the *right small thing* and to let the first genuine Rust design tensions
> surface naturally.

---

## Objectives

By the end of this chapter you should be able to:

1. Turn your Chapter 01 vocabulary into concrete Rust types you actually compile.
2. Explain the difference between a task's **description** (its data) and its
   **behavior** (the code that runs), and why that distinction forces a design choice.
3. Reason about **how to store many different kinds of task in one collection** — the
   first real "heterogeneity" problem — and articulate the tradeoffs of the options
   Rust gives you.
4. Model a task's **lifecycle status** as a type that makes illegal states hard to
   represent.
5. Split the project into a **library crate** and a **binary crate**, and justify why.
6. Build a working Version Zero: submit several tasks, run them in FIFO order, observe
   the result — with tests that assert on observable behavior.

We are still not distributed. We are not async. We are not persisting anything. We are
building the honest baseline everything else will be measured against.

---

## Concepts to Learn

These emerge from the problem, not the other way around:

- **Structs vs. behavior**: modeling data (`Task`) separately from what running it
  *means*.
- **The heterogeneity problem**: a queue must hold "some task," but different tasks do
  different work. This is the classic fork in the Rust road between **enums** (a
  closed set of variants) and **trait objects** (an open set of implementers). You
  will weigh both.
- **Trait objects and dynamic dispatch** (`dyn Trait`, `Box<dyn Trait>`) — *if* you
  choose that path — and what they cost at runtime.
- **Enums as a closed alternative** — and what they cost in flexibility.
- **Modeling state with types**: representing task status so that impossible
  transitions are hard to write.
- **Library vs. binary crates**: where the reusable core lives vs. where the program
  that runs it lives.

You already know the *syntax* for all of these (per your background). What this
chapter asks is that you choose *between* them for a real reason.

---

## Why These Concepts Matter

Your `docs/version-zero.md` sketch already committed to two things without fully
interrogating them:

> "We will model a `Task` struct, with some fields (including one for the task status,
> e.g.: Done, Pending, Failed), and the trait that will allow for homogenization of
> the tasks."

and, in the diagram, a list "storing **dyn type**."

Those are *good instincts*, but they are also *decisions* — and this chapter is where
you earn them or revise them. Two questions your sketch quietly raises:

1. If a `Task` is a **struct with fields**, and it also has a **trait** that
   "homogenizes" tasks, what exactly is the relationship between the struct and the
   trait? Is the struct the *data* and the trait the *behavior*? Can one struct
   represent every kind of task, or does each kind of work need its own type?
2. You wrote "list storing dyn type." Why `dyn`? What did you rule out to get there?
   If you can't answer that yet, this chapter exists to make you answer it *before*
   you type `Box<dyn ...>` out of habit.

Getting this layer right matters more than any later chapter, because every future
component — the async runtime, the network protocol, the persistence layer — will
wrap around whatever `Task` abstraction you choose here. A wrong turn now is the
expensive kind to undo.

---

## The Central Tension: Data vs. Behavior

Let's make the problem concrete. Imagine two tasks OxideQueue should eventually run:

- `SendEmail { to: String, subject: String, body: String }`
- `ResizeImage { path: String, width: u32, height: u32 }`

These have **different data**. They also do **different work** when executed. Yet your
queue needs to hold *both* in the same collection and, later, hand either one to a
worker that says only "run whatever this is."

So the worker needs a *uniform* way to say "run this," while each task needs its *own*
data and its *own* logic. That is the tension. Sit with it before reading the options.

> Ask yourself: what is the single operation the worker needs from *any* task,
> regardless of type? If you can name that one operation, you have found the shape of
> the abstraction.

### Two honest paths

Rust gives you two idiomatic ways to hold "many kinds of one thing." Neither is
"correct" — each buys and costs something.

**Path A — an `enum` (a closed set).**
You enumerate every task kind up front:

```rust example_only/closed_set.rs
// Illustration ONLY. Not OxideQueue code.
enum Task {
    SendEmail { to: String, subject: String, body: String },
    ResizeImage { path: String, width: u32, height: u32 },
}

impl Task {
    fn run(&self) {
        match self {
            Task::SendEmail { .. } => { /* ... */ }
            Task::ResizeImage { .. } => { /* ... */ }
        }
    }
}
```

**Path B — a trait + trait objects (an open set).**
You define the *one operation* and let any type implement it:

```rust example_only/open_set.rs
// Illustration ONLY. Not OxideQueue code.
trait Task {
    fn run(&self);
}

struct SendEmail { to: String, subject: String, body: String }
struct ResizeImage { path: String, width: u32, height: u32 }

impl Task for SendEmail  { fn run(&self) { /* ... */ } }
impl Task for ResizeImage { fn run(&self) { /* ... */ } }

// The queue would then hold `Box<dyn Task>` — "some boxed thing that can run."
```

Before you pick, reason through these — they are the real decision criteria, not
style preferences:

- **Who defines new task types?** If only *you*, in this crate, an `enum` is cheap and
  totally exhaustive — the compiler forces you to handle every variant. If you want
  *users of your library* to define their own task types without editing your enum,
  the `enum` cannot express that; a trait can.
- **Exhaustiveness vs. openness.** A `match` on an enum is checked for completeness at
  compile time — add a variant, the compiler shows you every place to update. A trait
  object gives up that safety net in exchange for extensibility.
- **Cost.** A trait object (`Box<dyn Task>`) means a heap allocation and **dynamic
  dispatch** (a pointer indirection through a vtable at each call). An enum is a
  single stack value dispatched by a `match`. For a task queue the per-call cost is
  almost always negligible next to the work the task does — but you should *know* you
  are paying it and be able to say why it's fine.
- **Where your project is going.** Recall your own scope note: OxideQueue will
  eventually "allow users to define executable tasks." Read that sentence again and
  ask which path it points toward. (I am not going to answer this for you — but I am
  going to insist you connect this decision back to `docs/scope.md`.)

This is exactly the kind of decision `architecture-rules.md` wants captured as an
ADR. Whichever you choose, write down *why*, and what would make you reverse it.

> Prior art, for your peripheral vision (do not go read the source yet): the broader
> Rust ecosystem uses **both** patterns constantly. Error libraries like `std::error`
> lean on `dyn Error` trait objects precisely because errors are an *open* set nobody
> can enumerate. Many state machines use enums precisely because the states are a
> *closed*, known set. Your task queue sits somewhere on that spectrum — the question
> is *where*, and *why*.

---

## Modeling Status Without Footguns

Your `version-zero.md` lists a status field: `Done`, `Pending`, `Failed`. Good — that
is naturally an `enum`, a closed set of states. But a field on a struct invites a
subtle question:

> Can a task be `Done` *and* also carry the reason it `Failed`? Can it be `Pending`
> while also holding a return value that only a finished task should have?

If status is just a flag sitting next to other fields, nothing stops those
contradictions from being *representable*. A core idea of idiomatic Rust is **making
illegal states unrepresentable** — letting the type system refuse to even compile a
nonsensical combination.

Some questions to drive your design (answer them in code, not prose):

- Should a `Failed` status *carry* the error, so a "failed with no error" state cannot
  exist? (Think `Failed(SomeError)` rather than a bare `Failed`.)
- Does a result value only make sense once a task is `Done`? If so, where should the
  result live — in the status enum, or floating in the struct where a `Pending` task
  could also (wrongly) have one?
- For Version Zero, how many of these states do you actually *need*? Do not model a
  lifecycle richer than what a single-threaded "push, pop, run" loop can even produce.
  (Hint to *think* about: in a synchronous loop, is there ever a moment where you can
  observe a task as "running but not yet done"? What does that tell you about which
  states are real *yet*?)

Resist over-modeling. You can always add states when a later chapter creates the
possibility of them (for example, when work actually happens elsewhere and "in
progress" becomes observable). Add states when the *system* can produce them, not
before — that is `methodology.md`'s "complexity must be justified" applied to types.

---

## Architectural Discussion: Where Does the Code Live?

Your Version Zero diagram already made a strong claim:

```
Producer (in the bin crate, using lib types & traits)
   → List (living in binary)
   → Execution/worker (inside binary)
```

So you've intuited a **split**: reusable *types and traits* in a library, the
*program that wires them together* in a binary. That is the right instinct, and it's
worth making explicit and justifying, because it will shape the whole workspace.

Why split at all, this early?

- The **library crate** (`src/lib.rs`) holds the *domain*: what a `Task` is, the trait
  or enum that unifies tasks, the queue type, the status. It knows nothing about
  *how* it's run or by whom.
- The **binary crate** (`src/main.rs`) holds the *application*: it creates concrete
  tasks, pushes them, and drives the loop that runs them.

The payoff is not abstract tidiness. It is that later, when there is *more than one*
program (imagine a separate producer and a separate worker — not now, but on the
horizon), they can both depend on the same library without duplicating the core
types. You are drawing the seam now, while it's free, instead of cutting through
working code later.

> A question to settle: should the queue itself live in the library or the binary?
> Your diagram put the list "in the binary." Is the queue part of the reusable
> *domain*, or part of this particular *application*? There's a defensible answer
> either way — pick one and be able to defend it. (Consider: will a future worker
> program need a queue type too?)

A minimal workspace shape to consider (you own `Cargo.toml`; I will not write it):

```
oxide_queue/
├── Cargo.toml        # you decide: single package with lib+bin, or a workspace
├── src/
│   ├── lib.rs        # the OxideQueue domain: Task, status, queue
│   └── main.rs       # the Version Zero application that uses the library
```

For Version Zero, a **single package exposing both a library and a binary** is likely
enough — Cargo supports `src/lib.rs` and `src/main.rs` in one package, and the binary
can use the library by its crate name. A full multi-crate *workspace* is more than
this chapter needs; reach for it only when a second binary actually appears. (That is
a decision for you, and a candidate ADR: "single package with lib+bin" vs. "workspace
of crates.")

---

## The Empty-Queue Question, Revisited

Chapter 01 planted a question via `VecDeque::pop_front` returning `Option`:

> What does it mean for a worker to ask for work when there is none?

Now you must *answer it in code*. In Version Zero's single-threaded loop, the answer
is probably simple: `pop_front()` returns `None`, the loop ends, done. But name the
decision anyway. Is "empty" the *normal* termination condition of your run loop? Is it
an error? For a synchronous drain-the-queue-and-stop program, "empty means stop" is a
perfectly good answer — but you should choose it deliberately, because in a later
chapter (when the producer keeps producing while the worker runs) "empty" will stop
meaning "stop" and start meaning "wait." You don't have to solve that now. You *do*
have to notice that today's answer is temporary.

---

## Exercises

These are yours. The mentor will not solve them. Prefer small, frequent commits.

1. **Name the one operation.** In one sentence, write down the single thing a worker
   needs to do to *any* task, regardless of its type. This sentence is the signature
   of your core method. Put it in a comment or note before you write the trait/enum.

2. **Choose your abstraction, on the record.** Decide between an `enum` of task kinds
   and a `trait` with trait objects for OxideQueue's tasks. Write an ADR
   (`adr/0002-...`) capturing the choice, the alternatives, and — crucially — the
   *criterion* that decided it. Connect it explicitly to the line in `docs/scope.md`
   about users defining their own tasks.

3. **Model `Task` and status.** Implement your `Task` representation and a status type
   in the **library crate**. Make at least one illegal state unrepresentable (e.g.,
   ensure a "failed" task cannot exist without a reason, or that a result cannot
   attach to an unfinished task). Write a comment explaining which illegal state you
   eliminated and how.

4. **Build the queue.** Implement a minimal FIFO queue type wrapping an appropriate
   standard-library structure. Decide, and note, whether it lives in the library or
   the binary and why. Give it just two operations for now (submit / take-next) and
   nothing more.

5. **Wire Version Zero.** In the **binary crate**, create two or three *concrete*
   tasks of different kinds, submit them, then drain and run them in FIFO order.
   Prove FIFO order is preserved by observable output.

6. **Test observable behavior (your QA strength).** Write tests that assert on what is
   *observable*, not on internals: e.g., that tasks run in submission order, that a
   task that is designed to fail is reported as failed, that draining an empty queue
   does the thing you decided it should. Aim for at least one unit test in the library
   and one integration test that exercises the binary's behavior through the library
   API.

7. **Reflect on cost.** If you chose trait objects, write two sentences on where the
   heap allocation and dynamic dispatch happen and why that cost is acceptable here.
   If you chose an enum, write two sentences on what you'll do the day a user wants a
   task type you didn't foresee.

---

## Milestones

You have completed this chapter's milestone when:

- [x] Your project builds with a **library crate** and a **binary crate**, the binary
      using the library.
- [x] A `Task` representation and a status type exist in the library, and you can
      point to one illegal state you made unrepresentable.
- [x] A minimal FIFO queue exists with exactly the operations Version Zero needs.
- [x] The binary submits several heterogeneous tasks and runs them in FIFO order,
      producing observable output.
- [x] Tests assert on observable behavior (order, success/failure, empty-queue
      handling).
- [x] An ADR records your enum-vs-trait-object decision and its deciding criterion.

This is the **first chapter with an implementation milestone.** It is intentionally
small: a correct "list plus a loop" with well-chosen types, nothing more.

---

## Reflection Questions

1. You committed (or reconsidered) "list storing `dyn` type." After this chapter, can
   you defend the `dyn` — or did you change your mind? What decided it?
2. In a single-threaded synchronous loop, which of your status values can actually
   *occur*? Which did you model "for the future" — and does that violate
   "complexity must be justified"?
3. If the queue lives in the library, what does that imply the day a separate worker
   program appears? If it lives in the binary, what will you have to move later?
4. Your worker currently *is* the loop in `main`. What is the smallest change to the
   system that would make the worker want to become its own thing?
5. How did your QA instincts shape the *shape* of your types? (For example, did
   testability push you toward returning a result from `run` rather than having it
   print directly?)

---

## Common Mistakes

- **Reaching for `Box<dyn Task>` reflexively.** Trait objects are idiomatic and often
  right — but "often right" is not "right here, because." Make the criterion explicit.
- **One giant `Task` struct with a `kind: String` field.** Encoding the task type as a
  string and branching on it throws away everything the type system offers. If you
  find yourself matching on strings, you've reinvented a worse enum.
- **Over-modeling status.** Adding `Running`, `Retrying`, `Scheduled`, etc. before the
  system can produce those states. Model what a synchronous loop can actually cause;
  add the rest when a later chapter creates the possibility.
- **Putting behavior in the binary that belongs in the library** (or vice versa). The
  test: if a *future second program* would need it, it belongs in the library.
- **Testing internals instead of behavior.** Asserting on private fields couples your
  tests to your implementation. Assert on order, outcomes, and observable effects.
- **Skipping the ADR because the decision "feels obvious."** The obvious decisions are
  exactly the ones whose reasons you forget by Chapter 06.

---

## Design Decisions I Must Make Myself

Record the significant ones as ADRs.

1. **Enum vs. trait objects for tasks.** The defining decision of this chapter. Tie it
   to your scope statement.
2. **Status representation.** Which states exist *now*, and which illegal combinations
   you forbid at the type level.
3. **Where the queue lives** — library or binary — and the workspace shape (single
   package with lib+bin vs. a multi-crate workspace).
4. **The `run` contract.** Does running a task return a result/outcome, or produce a
   side effect? Does it take `&self` or consume the task? (Think about what the queue
   still needs after a task runs.)
5. **Empty-queue semantics for Version Zero.** "Empty means stop" vs. anything else —
   chosen deliberately, knowing it is temporary.

---

## Project Status

- **Phase:** Foundations → first implementation. Version Zero is being built:
  single process, single thread, in memory.
- **Repository:** Chapter 01 artifacts complete (glossary, scope, ADR 0001). This
  chapter drives the first library/binary split and the first real types.
- **Code:** Moves from the starter `Hello, world!` binary to a `Task` model, a FIFO
  queue, and a synchronous run loop, with tests.

## Current Architecture

```
   Version Zero (being built this chapter)

   ┌───────────────────────────────────────────────────────────┐
   │                       one process                          │
   │                                                            │
   │   [ binary crate ]                     [ library crate ]   │
   │   creates concrete tasks  ──uses──▶  Task model + status   │
   │   drives the run loop     ──uses──▶  FIFO queue type       │
   │                                                            │
   │   producer code ──submit──▶ [ queue: T1 T2 T3 ] ──take──▶  │
   │                                                    run(T)  │
   └───────────────────────────────────────────────────────────┘

   Single thread. In memory. No network. No persistence.
```

## Open Questions

- Does `run` return an outcome, or only cause side effects? What does the queue need
  from a task *after* it runs?
- Does the queue belong to the reusable domain (library) or this application (binary)?
- Which task states are real *today* versus modeled for a future the system can't yet
  reach?
- When the producer and worker stop sharing a single synchronous loop, what breaks
  first?

## Pending Design Decisions (with alternatives)

- **Task abstraction.**
  - *Alternative A:* `enum` of task kinds — exhaustive, cheap, closed set; cannot be
    extended by library users.
  - *Alternative B:* `trait` + `Box<dyn Task>` — open set, user-extensible; costs a
    heap allocation and dynamic dispatch. *You decide, tied to `docs/scope.md`.*
- **Workspace shape.**
  - *Alternative A:* single package with `src/lib.rs` + `src/main.rs`. Simplest;
    likely sufficient now.
  - *Alternative B:* multi-crate Cargo workspace. More structure than Version Zero
    needs yet. *You decide.*
- **`run` signature** — `&self` vs. consuming `self`; returns outcome vs. side effect.
  *You decide.*

## Knowledge Checklist

- [x] I can state the single operation a worker needs from any task.
- [x] I can explain when an `enum` beats trait objects and vice versa, in terms of
      openness, exhaustiveness, and cost.
- [x] I can explain what `Box<dyn Task>` costs at runtime and why it's acceptable
      here (if I chose it).
- [x] I made at least one illegal task state unrepresentable and can name it.
- [x] I can justify the library/binary split in terms of a future second program.
- [x] I chose deliberate empty-queue semantics and know why they're temporary.

## Suggested Commits

- `feat: split into library and binary crates`
- `feat: model Task and task status in the library`
- `feat: add minimal FIFO queue`
- `feat: wire Version Zero run loop in the binary`
- `test: assert FIFO order and success/failure behavior`
- `adr: record task abstraction decision (enum vs trait objects)`
- `docs: add Chapter 02 — modeling a task`

## Suggested Git Tags

- `v0.1.0-ch02` — Version Zero implemented: task model, FIFO queue, synchronous run
  loop, and tests.

## Suggested GitHub Milestone

- **Milestone: "Chapter 02 — Version Zero: Task Model & Run Loop"**
  - Deliverables: library/binary split, `Task` + status types, minimal FIFO queue, a
    working synchronous run loop over heterogeneous tasks, behavior-focused tests, and
    an ADR recording the task-abstraction decision.
