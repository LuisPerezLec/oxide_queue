# Chapter 03 — Outcomes, Illegal States, and Owning the Domain

> This chapter assumes `docs/00-roadmap.md`, `docs/01-introduction.md`,
> `docs/02-modeling-a-task.md`, and the artifacts you have produced so far:
> `docs/glossary.md`, `docs/scope.md`, `docs/version-zero.md`, and ADRs `0001` and
> `0002`. It also assumes the **working Version Zero code** you wrote for Chapter 02
> (`src/lib.rs` and `src/main.rs`). Future chapters are not written yet and will not
> be referenced in detail here.
>
> Chapter 02 asked you to build the *right small thing*: a library/binary split, a
> `Runnable` trait, a `Task` wrapper, a `VecDeque` queue, and a synchronous run loop.
> You did. It compiles, it runs two heterogeneous tasks in FIFO order, and it has
> tests. That is a real milestone — congratulations.
>
> This chapter is not about adding a big new capability. It is about something more
> valuable and more often skipped: **going back to look honestly at what you built,
> finding the design debts you took on to get it working, and paying the cheap ones
> down now — while they are still cheap.** We are still Version Zero. Still one
> process, one thread, in memory. No async yet. But by the end, your *domain* will be
> solid enough to build a real system on top of.

---

## First: A Candid Review of Your Chapter 02 Code

You cannot improve a design you have not looked at squarely. So before any new
thinking, let's read your own code back to you and name what is actually there. This
is exactly the review a senior engineer would give you in a pull request — strengths
first, then the debts.

Here is the shape you shipped (paraphrased from your `src/lib.rs`):

```rust src/lib.rs
// (your Chapter 02 code, abbreviated)
pub enum ExecutionState { Pending, Failure, Success }

pub struct Task {
    pub state: ExecutionState,
    pub task: Box<dyn Runnable>,
}

pub trait Runnable {
    fn run(&self) -> Result<(), TaskError>;
}
```

**What you got right — and should feel good about:**

- You chose **trait objects** (`Box<dyn Runnable>`) and recorded *why* in ADR 0002,
  tying it to the scope line about users defining their own tasks. That is the
  decision working exactly as `architecture-rules.md` intends.
- You separated **data + behavior** cleanly: a concrete type (`Calculator`,
  `Messager`) owns its data and its logic; `Runnable` is the one uniform operation the
  worker needs. You *named the one operation*. That was the whole point of Chapter 02.
- Your `Task::run` already **updates the status based on the outcome** and your test
  asserts that a failing task ends in `Failure`. That is behavior-focused testing —
  your QA instincts showing.

**Now the debts — stated plainly, because naming them is the job:**

1. **Your status enum lets illegal states exist.** `Failure` carries *no error*, and
   `Success` carries *no result*. Chapter 02 explicitly set the milestone "make at
   least one illegal state unrepresentable," and the milestone box is checked — but
   the code does not actually satisfy it. A `Failure` with no reason attached is
   representable today. That is precisely the footgun Chapter 02 warned about.

2. **`run` throws the answer away.** Look at your `Runnable::run` signature:
   `-> Result<(), TaskError>`. Your `Calculator` computes `1 + 2 = 3`, prints it, and
   then... returns `Ok(())`. The `3` is gone. Nothing above the task can ever learn
   what it produced. Your `docs/scope.md` says OxideQueue will eventually *"preserve
   task execution results for later retrieval"* and *"report failures to callers."*
   With today's signature, both are impossible — not "not yet built," but
   *structurally impossible*, because the value never leaves the function.

3. **`Task` fields are `pub`.** `state` and `task` are public. Anyone can reach in and
   write `task.state = ExecutionState::Success` without ever running anything. The
   status is supposed to be a *consequence* of running; right now it is just a
   writable flag. That is the "testing/observing internals" trap from Chapter 02's
   Common Mistakes, baked into the type.

4. **The queue lives in the binary.** Your `VecDeque<Task>` sits in `main.rs`.
   Chapter 02 flagged this as a real decision to make on the record — and you have not
   yet recorded it, nor decided whether the queue belongs to the reusable *domain*.

5. **`TaskError` is a single opaque variant.** `ExecutionFailed` cannot say *why*
   something failed. That is fine for a first pass, but it is directly connected to
   debt #1 and #2: if a failure cannot carry information, callers cannot be told
   anything useful.

None of these are disasters. All of them are *cheap to fix now* and *expensive to fix
later*, once async code, a network, and a result backend are all leaning on this
domain. That asymmetry — cheap now, expensive later — is the entire reason this
chapter exists. This is `methodology.md`'s "complexity must be justified" turned
around: we justify **paying down** debt by pointing at the concrete limitation we just
hit, not at a hypothetical future.

---

## Objectives

By the end of this chapter you should be able to:

1. Explain why *running a task must produce a value that outlives the run*, and
   redesign the `run` contract so it does.
2. Redesign your task status/outcome type so that **illegal states are
   unrepresentable** — for real this time, verifiable by "try to construct the bad
   state and watch the compiler refuse."
3. Decide, deliberately, whether the **queue belongs to the library domain** or the
   application, and record it.
4. Give your **error type enough structure** to carry a reason, without over-building
   it.
5. Tighten your module's **encapsulation** (field visibility, constructors,
   accessors) so that the only way to reach a state is the legitimate one.
6. Strengthen your **tests** to assert on outcomes and results, and add the
   integration test Chapter 02 asked for.

Still no async. Still one thread. We are making the *baseline trustworthy* before we
make it concurrent.

---

## Concepts to Learn

These emerge directly from the five debts above, not from a syllabus:

- **Type-driven state modeling ("make illegal states unrepresentable").** Using an
  `enum` whose variants *carry the data that only makes sense in that state* — e.g.,
  the error lives *inside* the failure variant, the result lives *inside* the success
  variant. Rust's enums are **sum types with payloads**; this is their superpower and
  you are under-using it.
- **Associated types vs. generics on a trait**, and why returning "a result" from a
  heterogeneous, `dyn`-compatible trait is *harder than it looks*. This is the deep
  Rust lesson of the chapter, and it is forced by debt #2. (Foreshadowing: this is
  where your `Box<dyn Runnable>` decision from Chapter 02 starts charging interest.)
- **Object safety (a.k.a. `dyn` compatibility).** Why some trait shapes can be turned
  into a `Box<dyn Trait>` and some cannot, and what that rules out.
- **Type erasure with `Box<dyn Any>` (or an owned bytes/`String` outcome)** as the
  pragmatic escape hatch — and its costs.
- **Encapsulation in Rust:** module privacy, private fields with constructors, and
  read-only accessors — enforcing invariants at the type boundary.
- **Richer error modeling with `thiserror`:** giving a variant fields and a good
  `Display` message.

You know the *syntax* for all of these. What is new is being *forced to choose between
them by a constraint you created for yourself in Chapter 02*.

---

## Why These Concepts Matter

Everything downstream in the roadmap depends on one question you have not yet
answered: **what does the system learn when a task runs?**

- A **result backend** (in your scope) stores *outcomes*. It cannot store what `run`
  refuses to return.
- **Reporting failures to callers** (in your scope) requires a failure that *carries a
  reason*.
- Later, when a worker runs in a *different place* than the producer, the only thing
  that can cross that gap is **data** — an outcome value, serialized. A `println!`
  inside `run` cannot cross a network. So the shape of "what `run` returns" quietly
  determines whether your later distributed design is even possible.

In other words: the modest-looking change "make `run` return something meaningful" is
actually the hinge the whole rest of the project swings on. That is why we stop and
get it right before touching concurrency.

---

## The Central Tension of This Chapter: What Does `run` Return?

Sit with the problem before reaching for a tool.

Your worker holds a `Box<dyn Runnable>`. It knows *nothing* about the concrete type
inside — that was the whole point of choosing trait objects. It just calls `run`. Now
you want `run` to hand back a *result*. But:

- The `Calculator`'s natural result is an `i32` (`3`).
- The `Messager`'s natural result is... what? Nothing? A `()`? A confirmation string?
- A future `ResizeImage`'s result might be a file path, or bytes.

So different tasks have **different result types**, yet the worker must call `run`
through *one* uniform interface that hides the concrete type. **What single return
type can express "any of these"?** This is the *exact same heterogeneity problem* you
solved for tasks in Chapter 02 — except now it is on the *output* side, and it is
sharper.

Do not read ahead until you have honestly tried to answer: *if the caller doesn't know
the concrete type, what can it possibly do with an `i32` versus a `String`?*

### Path A — return a fixed, uniform outcome type

Pick one type that every task's result can be squeezed into. Candidates:

- `Result<(), TaskError>` — what you have now. Uniform, but carries *no* result value.
  Rejected by debt #2.
- `Result<String, TaskError>` — every task returns a *string* describing its outcome.
  Uniform and dead simple. A `Calculator` returns `"3"`; a `Messager` returns
  `"sent"`. The cost: you have thrown away the *type* of the result — everything is
  stringly-typed, and a consumer must re-parse.
- `Result<Vec<u8>, TaskError>` — every task returns *bytes*. This looks arbitrary now,
  but notice it is exactly what a value must become to cross a network or land in a
  database. (Hold that thought; do not act on it yet — bytes-without-a-schema is its
  own trap for a later chapter.)

```rust example_only/uniform_outcome.rs
// Illustration ONLY. Not OxideQueue code.
pub trait Runnable {
    // Every task collapses its result into one shared, type-erased shape.
    fn run(&self) -> Result<String, TaskError>;
}
```

### Path B — a generic associated result on the trait

Let each implementer declare its own result type:

```rust example_only/assoc_result.rs
// Illustration ONLY. Not OxideQueue code.
pub trait Runnable {
    type Output;                       // each task picks its own result type
    fn run(&self) -> Result<Self::Output, TaskError>;
}
```

This is beautiful and type-safe — and it **breaks your Chapter 02 design.** Ask
yourself *why* before reading the answer:

> Can you still write `Box<dyn Runnable>` if `Runnable` has an associated type
> `Output` that differs per implementer?

The answer is no — not without pinning the associated type (`Box<dyn Runnable<Output =
i32>>`), which forces *every* task in the queue to have the *same* output type, which
destroys the heterogeneity you built the trait for in the first place. This is the
concept called **object safety** / **`dyn` compatibility**: a trait can only become a
trait object if the compiler can build a single, uniform vtable for it, and an
unconstrained associated type makes that impossible.

This is the moment Chapter 02's decision shows its teeth. You chose an *open set* of
tasks via `dyn`. An open set means the worker genuinely cannot know the result type at
compile time. **Openness on the input side forces uniformity on the output side.**
That is not a Rust wart; it is the logical consequence of the abstraction you picked.

### Path C — type erasure with `Box<dyn Any>`

Keep `dyn Runnable`, but let the *result* itself be type-erased:

```rust example_only/any_outcome.rs
// Illustration ONLY. Not OxideQueue code.
use std::any::Any;
pub trait Runnable {
    fn run(&self) -> Result<Box<dyn Any>, TaskError>;
}
```

Now any task can return any type, and a consumer who *knows* the concrete type can
`downcast` it back. This is maximally flexible and maximally *unsafe-feeling*: the
consumer must guess the type correctly at runtime, and a wrong guess is a runtime
`None`, not a compile error. You have traded compile-time safety for flexibility —
sometimes worth it, often not.

### The decision is yours — but decide *for a reason*

I will not pick for you. But I will insist you connect the choice to your scope and to
where you are in the journey:

- For **Version Zero**, do you actually need to *consume* results programmatically
  yet? Or do you just need to *preserve* an outcome so the run loop can report
  "task 2 produced X, task 3 failed because Y"? If the latter, the simplest uniform
  outcome (Path A with a `String`, or even a small purpose-built outcome enum) may be
  exactly enough — and honest about it.
- Whatever you pick, write down the **criterion** and what would make you revisit it.
  This is a genuine, hard-to-reverse decision: it belongs in an ADR.

> Prior art for your peripheral vision (do not go read source yet): async runtimes
> face this exact problem. A `tokio::task::JoinHandle<T>` is generic over the output
> `T` precisely because a spawned task's result type *is* known at the call site there
> — the opposite of your `dyn` queue. Meanwhile, systems that store heterogeneous
> results *do* erase them to bytes or JSON and rely on a schema to recover meaning.
> You are sitting exactly on that fault line. Notice which side your `dyn` choice pushes
> you toward.

---

## Making Illegal States Unrepresentable — For Real

Now the second debt. Your current status:

```rust src/lib.rs
pub enum ExecutionState { Pending, Failure, Success }
```

The problem: the *interesting data* — the error, the result — lives *outside* the
enum (or nowhere at all), so nothing ties "this task failed" to "here is why." Rust
enums are **sum types**: each variant can carry its own payload. Use that.

The design question to answer *in code*:

> Where should the error live so that a "failed with no reason" state cannot be
> constructed, and where should the result live so that a "pending task with a result"
> state cannot be constructed?

A shape to *react to* (not copy — react to; it may be wrong for you):

```rust example_only/outcome_state.rs
// Illustration ONLY. Not OxideQueue code. React to it; do not paste it.
enum ExecutionState {
    Pending,
    Succeeded(/* the result — whatever you decided in Path A/B/C */),
    Failed(TaskError),   // a failure literally cannot exist without a reason
}
```

Now interrogate *this* shape too, because it is not automatically right:

- With the result living *inside* `Succeeded(...)`, can a `Pending` task hold a result?
  (No — good.) Can a `Failed` task exist with no error? (No — good.)
- Does your `Task` still need a *separate* result field somewhere? (If the outcome
  lives in the state enum, a second copy is a bug waiting to happen — a source of the
  contradictions you are trying to forbid.)
- Which states can a **single-threaded synchronous loop actually produce**? You cannot
  observe a task as "running but not done" in a blocking loop — so should a `Running`
  variant exist *yet*? (Chapter 02's rule: add states when the *system* can produce
  them, not before. Deleting a premature state is as valid a design act as adding one.)

The acceptance test for this section is concrete and satisfying: **try to write the
illegal state and let the compiler stop you.** In a comment or a `// does not compile`
note, record one construction the type now forbids that the old design allowed. That
is the milestone, provably met this time.

---

## Encapsulation: Make the Only Path the Right Path

Debt #3 was `pub` fields. If `Task.state` is public and writable, then "the status
reflects what actually happened" is not an *invariant* — it is a *hope*. Invariants
you merely hope for are the ones that break at 3 a.m.

The Rust tools here are ones you already know, now applied with intent:

- **Private fields** + a **constructor** (`Task::new`) that establishes the starting
  invariant (a fresh task is `Pending`, with no result).
- A **read-only accessor** (`fn state(&self) -> &ExecutionState`) so callers can
  *observe* status without being able to *forge* it.
- A **method that is the only way to change state** — your `run` — so a transition to
  `Succeeded`/`Failed` can *only* happen as a consequence of actually running.

Questions to settle:

- Should the run loop be able to read a task's outcome *after* `run`? If so, the
  accessor is part of your public API — design its signature deliberately (return a
  reference? a clone? an `Option`?).
- Does `run` take `&mut self`, `&self`, or **consume** `self`? Chapter 02 left this
  open. Now that `run` produces an outcome the queue may want to *keep*, think: after a
  task has run and produced its result, is the `Box<dyn Runnable>` inside it still
  useful, or is it spent? Your answer changes whether the queue stores "tasks" or
  "tasks-and-their-outcomes." (There is no single right answer — but there is a right
  *process*: decide what the caller needs *after* the run, then pick the signature
  that provides exactly that and no more.)

---

## Where Does the Queue Live? (Deciding Debt #4)

Chapter 02 asked and you deferred. This chapter, decide it, because the answer changes
what your *library* even is.

Reason it through with a single concrete future in mind (from your own roadmap and
scope): *"support execution across multiple machines"* and *"execute multiple tasks
concurrently."* Both imply, eventually, a **separate worker program** and possibly a
**separate producer program**.

- If the queue is part of the **reusable domain (library)**, both future programs can
  depend on the same queue type. The seam is already cut.
- If the queue lives in the **binary**, the day a second program appears you will be
  moving code across a crate boundary — a mechanical but avoidable chore, and a
  contradiction of the "library holds the domain" instinct you already recorded.

Notice this does *not* mean building the concurrent, shared, thread-safe queue now
(that complexity is not justified yet — there is no second thread to share it *with*).
It means: **decide the queue's home**, give it the two operations Version Zero needs
(submit / take-next), and keep it dead simple. A `VecDeque` wrapped in a small type
with two methods is plenty. The point is *where the type is declared*, not how fancy it
is.

> A subtle naming question for your glossary: is this type the **queue** or the
> **broker**? Your glossary defines the broker as "the actor that manages the task
> collection and provides them to workers." A thin type with `submit`/`take_next` is
> arguably the seed of your broker. You need not rename anything today — but note the
> tension, because a later chapter will separate them and you will be glad you saw it
> coming.

---

## Giving `TaskError` a Reason (Deciding Debt #5)

Your error is:

```rust src/lib.rs
#[derive(Error, Debug, PartialEq)]
pub enum TaskError {
    #[error("Task execution failed")]
    ExecutionFailed,
}
```

A single, message-less variant cannot tell a caller *why*. You do not need a taxonomy
of twenty error kinds — that would be over-modeling. You *do* need failures to carry
information, because "report failures to callers" is in your scope. The smallest
honest improvement is a variant that carries a reason:

```rust example_only/error_with_reason.rs
// Illustration ONLY. Not OxideQueue code.
#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("task failed: {0}")]
    ExecutionFailed(String),
}
```

Two things to reason about before you do even this:

- Adding a `String` payload will likely **break `#[derive(PartialEq)]`** ergonomics in
  your tests (comparing error strings is brittle). Ask: do your tests need to assert on
  the *exact* error, or just that the task *failed*? Testing "it failed" (the variant)
  rather than "it failed with this precise string" keeps tests robust. Let your QA
  instincts choose the assertion granularity.
- Do you want to preserve an underlying error *source* (the thing that caused the
  failure)? `thiserror`'s `#[source]` / `#[from]` exist for exactly this and will
  matter later. You do not need them yet — but know they are the tool when a real
  underlying error (an I/O failure, a parse failure) appears.

Resist building a rich error enum now. One variant that carries a reason is the
justified step. More variants arrive when a real failure mode appears that the current
type cannot express — same rule as everything else.

---

## Architectural Discussion: The Domain vs. The Application

Step back. What you are really doing this chapter is drawing a clean line between two
things that got a little tangled in the rush to make Chapter 02 work:

```
   The DOMAIN (library crate) — pure, reusable, knows nothing about "how it's driven"
   ┌───────────────────────────────────────────────────────────┐
   │  Runnable          — the one operation, returning an outcome │
   │  TaskError         — a failure that carries a reason          │
   │  ExecutionState    — Pending | Succeeded(result) | Failed(err)│
   │  Task              — private fields, run() is the only mutator│
   │  Queue/Broker(seed)— submit / take_next over a VecDeque       │
   └───────────────────────────────────────────────────────────┘
                              ▲
                              │ depends on
                              │
   The APPLICATION (binary crate) — the specific Version Zero program
   ┌───────────────────────────────────────────────────────────┐
   │  concrete tasks (Calculator, Messager, ...)                  │
   │  the run loop: submit some tasks, drain, run, report outcomes│
   └───────────────────────────────────────────────────────────┘
```

The test for "does this belong in the library?" is unchanged from Chapter 02: *would a
future second program need it?* A worker binary would need `Runnable`, `Task`,
`ExecutionState`, `TaskError`, and the queue. It would *not* need your specific
`Calculator`. That tells you where the line goes.

---

## Exercises

These are yours. The mentor will not solve them. Prefer small, frequent commits, one
per debt.

1. **Write the review yourself.** Before changing code, write a short
   `docs/03-review-notes.md` (or a section in your notes) listing, in your own words,
   the debts in your Chapter 02 code and which ones you intend to pay down now. Owning
   the critique is half the skill.

2. **Decide the `run` return contract, on the record.** Choose Path A, B, or C (or a
   defensible hybrid). Write an ADR (`adr/0003-...`) capturing the choice, the
   alternatives, the **object-safety reasoning** that rules some options out, and the
   criterion that decided it. Tie it explicitly to the scope lines about *preserving
   results* and *reporting failures*.

3. **Redesign the outcome/state type.** Reshape `ExecutionState` so the result lives in
   the success variant and the error lives in the failure variant. In a comment,
   record **one illegal state the new type forbids** that the old one allowed — and,
   if you can, a `// does not compile:` line demonstrating it.

4. **Encapsulate `Task`.** Make its fields private. Provide a constructor that starts a
   task `Pending`, a read-only accessor for its state/outcome, and ensure `run` is the
   only way the state can transition. Decide `&self` vs `&mut self` vs `self` for `run`
   and justify it in a comment.

5. **Move the queue into the library (or justify not).** Implement a minimal queue type
   in the library with exactly `submit` and `take_next`. If you decide it stays in the
   binary, write the *one-sentence* justification you'd defend in review. Either way,
   note the queue-vs-broker naming tension in your glossary.

6. **Give `TaskError` a reason.** Add a payload (e.g., a message and/or a `#[source]`)
   so a failure can explain itself. Adjust your tests to assert on the *variant/outcome*
   rather than an exact string where that makes them more robust.

7. **Strengthen tests + add the integration test.** Add/adjust tests so they assert
   that a successful task's **result is preserved and readable**, that a failing task's
   **reason is available**, and that FIFO order still holds. Add at least one test in
   `tests/` that drives the whole flow **through the library's public API** (submit →
   drain → run → inspect outcomes) — the integration test Chapter 02 asked for.

8. **Update `main.rs` to report outcomes.** Change the run loop so that instead of
   `println!`-ing *inside* tasks and discarding the result, it takes each task's
   returned outcome and reports it from the loop (e.g., `task 2 produced 3`, `task 3
   failed: ...`). Notice how this pulls the *observable behavior* out of the tasks and
   into the driver — where a real worker will eventually live.

---

## Milestones

You have completed this chapter's milestone when:

- [ ] `run` returns a meaningful outcome (not `()`), by a contract you chose and
      recorded in an ADR, with the object-safety tradeoff articulated.
- [ ] Your state/outcome type makes at least one previously-representable illegal
      state **fail to compile**, and you can point at it.
- [ ] `Task`'s fields are private; its status can only change by running it; there is a
      read-only way to observe the outcome.
- [ ] The queue's home (library vs. binary) is decided and, if moved, implemented with
      exactly `submit`/`take_next`.
- [ ] `TaskError` can carry a reason.
- [ ] Tests assert on preserved results and available failure reasons, FIFO order
      still holds, and at least one integration test drives the library API.
- [ ] The binary reports each task's outcome from the run loop rather than only from
      inside the tasks.

This is a **consolidation milestone**, not an expansion one. If your line count barely
grows — or even shrinks as you delete a premature status variant — that is a sign you
did it right.

---

## Reflection Questions

Sit with these; they seed the concurrency chapters ahead.

1. Chapter 02's `dyn` decision made the *input* side open. This chapter you felt it
   constrain the *output* side (object safety, uniform return). Was that a good trade?
   What would returning to a generic/enum design cost you now versus in Chapter 02?
2. Which `ExecutionState` variants can a single-threaded blocking loop actually
   produce today? Did you *remove* any premature ones? What concrete change to the
   system would make a `Running` state finally observable?
3. You decided `run`'s signature (`&self`/`&mut self`/`self`). After a task runs and
   yields its outcome, what — if anything — does the queue still need from it? Does
   your signature match that need exactly, or over/under-provide?
4. If your outcome type is type-erased (a `String`, bytes, or `dyn Any`), what does a
   consumer have to *know* to make sense of it? Where would that knowledge come from in
   a distributed system? (You do not have to solve this — just locate the future pain.)
5. You pulled observable behavior out of the tasks and into the run loop. In one
   sentence: how did that make the system easier to *test*? (Your SDET hat.)

---

## Recommended Reading

- **The Rust `std::any` module** (`Any`, `downcast_ref`) — *only if* you are weighing
  Path C. Read for the mental model of type erasure and its runtime-checked recovery,
  not to adopt it reflexively.
- **The Reference on object safety / "dyn compatibility."** Read the list of what
  makes a trait *not* dyn-compatible; find your Chapter 02 `Runnable` in that list and
  understand exactly why an associated `Output` would break it.
- **`thiserror` docs** — the `#[error(...)]`, `#[from]`, and `#[source]` attributes.
  Read for how to attach a reason and a cause to an error variant cleanly.
- **The "Encapsulation that Hides Implementation Details" section of *The Book*
  (Ch. 17/18 depending on edition)** — a refresher on private fields + public methods
  as invariant enforcement, now that you have a real invariant to protect.
- Prior art, peripheral vision only: skim how `std::thread::JoinHandle<T>` and
  `tokio::task::JoinHandle<T>` are *generic over the output*. Ask why they can be, and
  your `dyn` queue cannot. That contrast is the whole lesson of this chapter in two
  type signatures.

---

## Common Mistakes

- **"Fixing" illegal states by adding an assertion or a comment.** The goal is to make
  the bad state *not compile*, not to guard against it at runtime. If you can still
  construct it, you have not fixed it.
- **Reaching for `Box<dyn Any>` because it is the most flexible.** Flexibility is not
  free; it moves type errors from compile time to runtime. Choose it only if you can
  say *why* the uniform or generic paths don't fit — and where the `downcast` will live.
- **Over-modeling `ExecutionState` or `TaskError`.** Adding `Running`, `Retrying`,
  `Cancelled`, or ten error variants the system cannot yet produce. Model what today's
  loop causes; add the rest when a later chapter creates the possibility.
- **Keeping the result in two places.** If the outcome lives in the state enum *and* in
  a separate `Task` field, you have re-created the contradiction you set out to remove.
- **Leaving `pub` fields "for the tests."** If a test needs to reach into a private
  field, that is a hint your public API is missing an accessor the *real* caller also
  wants. Add the accessor; test through it.
- **Moving the queue to the library and immediately making it thread-safe.** There is
  no second thread yet. `Arc<Mutex<...>>` now is unjustified complexity. Move the type;
  keep it single-threaded and simple.

---

## Design Decisions I Must Make Myself

Record the significant ones as ADRs.

1. **The `run` return contract.** Uniform outcome vs. generic associated type vs. type
   erasure. The defining decision of this chapter; tie it to object safety and scope.
2. **Outcome/state representation.** Which variants exist now, what data each carries,
   and which illegal combinations are forbidden at the type level.
3. **Queue home and shape.** Library vs. binary; `submit`/`take_next` only; and its
   naming relationship to "broker."
4. **`Task` mutation model.** `&self` vs `&mut self` vs consuming `self` for `run`, and
   what the caller can observe afterward.
5. **Error richness.** How much structure `TaskError` carries now (a reason? a source?)
   and the rule for when to add more.

---

## Project Status

- **Phase:** Foundations → hardening the domain. Still Version Zero: single process,
  single thread, in memory. No async, no network, no persistence.
- **Repository:** Chapters 01–02 written; glossary, scope, version-zero sketch, and
  ADRs 0001–0002 in place. This chapter drives a domain-consolidation pass over the
  working Chapter 02 code and a new ADR for the `run` contract.
- **Code:** A working "list plus a loop" gains a meaningful outcome type, illegal
  states removed at the type level, encapsulated `Task`, a library-owned queue (if you
  so decide), a reason-carrying error, and stronger tests including an integration
  test.

## Current Architecture

```
   Version Zero (hardened this chapter)

   ┌───────────────────────────────────────────────────────────────┐
   │                          one process                           │
   │                                                                │
   │   [ binary: application ]              [ library: domain ]      │
   │   Calculator / Messager   ──uses──▶  Runnable (-> outcome)      │
   │   run loop reports         ──uses──▶  Task (private, run-only)  │
   │     each outcome           ──uses──▶  ExecutionState            │
   │                            ──uses──▶    Pending                 │
   │                            ──uses──▶    Succeeded(result)       │
   │                            ──uses──▶    Failed(TaskError)       │
   │                            ──uses──▶  Queue: submit / take_next │
   │                                                                │
   │   submit ──▶ [ queue: T1 T2 T3 ] ──take_next──▶ run ──▶ outcome │
   │                                                    │            │
   │                                                    ▼            │
   │                                        loop reports outcome     │
   └───────────────────────────────────────────────────────────────┘

   Single thread. In memory. No network. No persistence.
```

## Open Questions

- If the outcome type is erased (string/bytes/`dyn Any`), what carries the *schema*
  that makes it meaningful again — especially once producer and worker are separate?
- Is your `submit`/`take_next` type the *queue* or the seed of the *broker*? When do
  they split?
- After `run` yields an outcome, is the `Box<dyn Runnable>` inside the task spent? Does
  the queue keep tasks, outcomes, or both?
- Which status transitions are even *observable* in a blocking loop — and which are you
  only modeling because a future chapter will need them?

## Pending Design Decisions (with alternatives)

- **`run` return contract.**
  - *Alternative A:* uniform outcome (`Result<String, TaskError>` / small outcome enum)
    — simplest, keeps `dyn`, erases the result's type.
  - *Alternative B:* generic associated `Output` — fully type-safe, **not
    dyn-compatible**, forces a homogeneous queue.
  - *Alternative C:* `Result<Box<dyn Any>, TaskError>` — flexible, runtime-checked
    downcast, no compile-time result safety. *You decide, tied to scope + object safety.*
- **Queue home.**
  - *Alternative A:* library (reusable domain) — ready for a future second program.
  - *Alternative B:* binary (this application) — simplest today, a move later. *You decide.*
- **`run` mutation model** — `&self` vs `&mut self` vs `self`. *You decide.*
- **Error richness** — reason-only vs. reason + `#[source]`. *You decide the minimum.*

## Knowledge Checklist

- [ ] I can explain why `run` returning `()` blocks two of my own scope goals.
- [ ] I can explain object safety and why an associated `Output` breaks
      `Box<dyn Runnable>`.
- [ ] I made at least one illegal task state fail to *compile* and can point to it.
- [ ] I encapsulated `Task` so its status can only change by running it.
- [ ] I decided the queue's home for a reason tied to a future second program.
- [ ] My tests assert on preserved results and failure reasons, and one integration
      test drives the library API.

## Suggested Commits

- `docs: add Chapter 03 — outcomes, illegal states, owning the domain`
- `refactor: make ExecutionState carry result and error (illegal states unrepresentable)`
- `feat: return a meaningful outcome from Runnable::run`
- `refactor: encapsulate Task (private fields, run-only state transition)`
- `feat: move FIFO queue into the library (submit / take_next)`
- `feat: let TaskError carry a failure reason`
- `test: assert preserved results and failure reasons; add integration test`
- `refactor: report task outcomes from the run loop, not from inside tasks`
- `adr: record the run return contract (uniform vs generic vs type-erased)`

## Suggested Git Tags

- `v0.2.0-ch03` — Version Zero domain hardened: meaningful outcomes, illegal states
  removed at the type level, encapsulated `Task`, library-owned queue, reason-carrying
  errors, and behavior-focused tests including an integration test.

## Suggested GitHub Milestone

- **Milestone: "Chapter 03 — Hardening the Domain: Outcomes & Illegal States"**
  - Deliverables: an ADR for the `run` return contract (with object-safety reasoning),
    an outcome/state type that forbids illegal states at compile time, an encapsulated
    `Task`, a library-owned FIFO queue, a reason-carrying `TaskError`, and tests
    (unit + integration) asserting on preserved results, failure reasons, and FIFO
    order.
