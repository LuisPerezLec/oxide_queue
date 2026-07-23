# Chapter 01 — Introduction: What Are We Actually Building?

> This is the first chapter of the OxideQueue guide. It assumes only the roadmap
> (`docs/00-roadmap.md`) exists. Future chapters are not written yet, and this chapter
> will not reference them in detail.
>
> This chapter is deliberately **light on code and heavy on thinking**. Before we
> write a single line of the system, we need a shared vocabulary and a mental model.
> A distributed task queue is a systems project; the hardest parts are rarely the
> syntax. They are the *decisions*. So this chapter is where we build the map before
> we start walking.

---

## Objectives

By the end of this chapter you should be able to:

1. Explain, in your own words, what a task queue is and what problem it solves.
2. Name the core roles in a task queue system and describe how work flows between
   them.
3. Distinguish between *synchronous* and *asynchronous* execution at the
   **architectural** level (not the `async`/`await` keyword level — that comes much
   later, and only when we need it).
4. Articulate why "distributed" changes everything, and why we will **not** be
   distributed on day one.
5. Write down the **glossary** and **scope** for OxideQueue in your own repository, as
   your first real design artifacts.
6. Make and record your first small architectural decisions.

We are **not** writing the queue yet. We are defining what "done" even means.

---

## Concepts to Learn

- The **task queue** pattern and the problem category it belongs to.
- The **producer / broker / worker / result backend** vocabulary (borrowed loosely
  from Celery, but we will define our own terms).
- **Latency vs. throughput**, and **synchronous vs. asynchronous** request handling
  as an architectural tradeoff.
- **Coupling** in time (temporal decoupling) — why a queue exists at all.
- The difference between a **problem statement**, a **scope**, and an
  **architecture** — and why confusing them is the most common early mistake.

These are systems-design concepts, not Rust concepts. That is intentional. You cannot
choose good Rust abstractions for a system you cannot yet describe in plain language.

---

## Why These Concepts Matter

You come from a web + QA background. You have almost certainly *used* something like a
task queue without building one: sending an email "in the background," resizing an
uploaded image after the HTTP response returns, retrying a flaky third-party call.
The queue is the invisible machinery behind "we'll handle that shortly."

If we skip the vocabulary, three bad things happen:

1. We invent inconsistent names for the same thing and confuse ourselves in later
   chapters.
2. We reach for a fancy solution (a database! a network protocol! threads!) before we
   understand the problem it solves, violating our own guiding principle that
   *complexity must be justified*.
3. We can't tell whether the thing we built actually works, because we never defined
   what it was supposed to do.

So: words first, then a mental model, then — later — code.

---

## The Problem, Told as a Story

Imagine a web endpoint that lets a user upload a photo. Naively:

```
Client ──HTTP request──▶ Web handler
                           │
                           ├─ save original file
                           ├─ generate thumbnail  (slow: 1.5s)
                           ├─ run virus scan       (slow: 3s)
                           ├─ notify followers      (slow: network calls)
                           │
Client ◀──HTTP response──┘   (user waited ~5s staring at a spinner)
```

Everything happens **inline**, while the user waits. The response is only as fast as
the slowest step. This is *synchronous* request handling.

Now ask yourself some questions before reading on:

- Which of those steps does the user actually need to have *finished* before we can
  honestly say "your upload succeeded"?
- What is the cost, to the user and to the server, of doing the slow steps inline?
- If the "notify followers" step fails, should the whole upload fail?

The insight behind a task queue is: **some work does not need to happen now, in the
same place, as part of the same request.** We can *accept* the work, acknowledge it,
and let it happen elsewhere, later.

```
Client ──HTTP request──▶ Web handler
                           │
                           ├─ save original file
                           ├─ enqueue "make_thumbnail"  ─┐
                           ├─ enqueue "virus_scan"       ├─▶  (a queue)
                           ├─ enqueue "notify_followers" ─┘
                           │
Client ◀──HTTP response──┘   (user waited ~0.2s)

                (queue)
                   │
                   ▼
             Worker process ──▶ actually does the slow work, later
```

The web handler's job shrank to: *validate, persist the essentials, and hand off the
rest.* Something else — a **worker** — picks the work up and runs it.

That "something else," and the machinery that reliably gets work from the handler to
the worker, is what we are building.

---

## The Core Vocabulary

We will loosely borrow Celery's mental model but define our own terms so we are not
bound to Celery's exact semantics. Here are the roles. Read each one and, before you
continue, try to describe it back to yourself without looking.

- **Task**: a named unit of work plus the data it needs to run. *"Make a thumbnail of
  file X at size Y."* A task is a *description*, not the running of it.
- **Producer** (a.k.a. client / caller): whoever creates a task and submits it. In our
  story, the web handler.
- **Queue**: the ordered holding area where submitted tasks wait to be executed.
- **Broker**: the component responsible for *accepting* tasks from producers and
  *handing them out* to workers. Early on, our "broker" and "queue" may be the same
  simple thing. They separate later, if and when we need them to.
- **Worker**: a process that pulls tasks from the queue and executes them.
- **Result backend** (optional, and *not* for now): a place to store the outcome of a
  task so a producer can later ask "did my task succeed, and what did it return?"

```
        ┌──────────┐   submit    ┌───────────────────┐   deliver   ┌──────────┐
        │ Producer │ ──────────▶ │  Broker  +  Queue │ ──────────▶ │  Worker  │
        └──────────┘             └───────────────────┘             └──────────┘
                                                                        │
                                                                        │ (maybe)
                                                                        ▼
                                                                  ┌───────────┐
                                                                  │  Result   │
                                                                  │  backend  │
                                                                  └───────────┘
```

Notice what this diagram does **not** yet say:

- Are the producer, broker, and worker in the *same process*? Different processes?
  Different machines?
- Is the queue in memory? On disk? In another service?
- What happens if a worker crashes halfway through a task?

Those are the interesting questions. We are going to answer the *simplest* version of
each first, on purpose.

---

## "Distributed" — and Why Not Yet

The roadmap says *distributed* task queue. That word is doing a lot of work, and it is
worth being honest about what it costs.

A system is **distributed** when its parts run in separate failure domains —
different processes, and eventually different machines connected by a network — such
that any part can fail, restart, or become unreachable independently of the others.

Here is the uncomfortable truth about distribution, and a question to sit with:

> If a producer submits a task over a network and never hears back, does it know
> whether the task was received? Whether it ran? Whether it ran *twice*?

You cannot answer that cleanly. The network can lose the request, lose the response,
or delay either arbitrarily. This single fact — that you cannot distinguish "it
failed" from "it succeeded but the acknowledgement was lost" — is the source of an
enormous fraction of distributed-systems complexity (retries, idempotency,
at-least-once vs. at-most-once delivery, acknowledgements, timeouts).

We will absolutely get there. But if we start distributed, we will spend our early
energy fighting the network instead of learning the core structure of a task queue.
So our very first architectural decision is a deliberate simplification:

> **We will begin with everything in a single process, in memory.** No network. No
> disk. No separate worker binary yet. The producer, the queue, and the "worker" all
> live in one program.

This is not the goal; it is the *starting point*. It lets us build a working mental
model we can trust, and every later chapter will introduce distribution one honest
step at a time — and each step will be forced by a concrete limitation we actually
hit, not by ambition.

This is our guiding principle in action: **complexity must be justified, and it must
emerge from a limitation we discover — never added proactively.**

---

## Architectural Discussion: What "Version Zero" Looks Like

Let's reason about the smallest thing that is still recognizably a task queue.

At minimum we need:

1. A way to *describe* a task (a name + some data).
2. A place to *put* tasks (the queue).
3. A way to *take* tasks back out, in some order.
4. Something that *runs* a task once it's taken out.

Ask yourself:

- If everything is in one process and one thread, and the producer submits three
  tasks and then immediately runs them itself, is that a task queue... or just a
  list and a loop?
- What is the *first* property that would make it feel like a real queue rather than
  a fancy `for` loop?

Here's the honest answer to hold onto: at Version Zero, a task queue really can be
"a list plus a loop." That is *fine*. The value of building it this way is that you
will personally feel the moment it stops being enough — when you want the producer to
*not wait* for the work, when you want *more than one* thing running at once, when you
want the work to *survive a restart*. Each of those moments is a future chapter, and
each one will justify the next layer of machinery.

For now, resist the urge to build for those futures. Build the list and the loop.
Give it clean names. Make it easy to change.

```
   Version Zero (single process, single thread)

   ┌──────────────────────────────────────────────┐
   │                 one program                    │
   │                                                │
   │   producer code ──push──▶ [ queue: T1 T2 T3 ] │
   │                                │               │
   │                             pop │               │
   │                                ▼               │
   │                         run the task           │
   └──────────────────────────────────────────────┘
```

---

## A Small, Isolated Rust Aside (Concept Only)

Per our own rules, the only code the mentor shows you is a *tiny isolated example
whose sole purpose is to illustrate a concept*, never part of the OxideQueue
implementation. Here is one, purely to anchor a question you'll need to answer
yourself:

```rust example_only/queue_shape.rs
// Illustration ONLY. Not OxideQueue code. Do not paste this into src/.
// A queue is fundamentally "put things in one end, take them out the other."
// The standard library already has a double-ended queue:
use std::collections::VecDeque;

fn demo() {
    let mut q: VecDeque<&str> = VecDeque::new();
    q.push_back("task-1");   // producer side
    q.push_back("task-2");
    let next = q.pop_front(); // worker side; FIFO order
    // `next` is an Option<&str> — why an Option? What does None mean for a queue?
    let _ = next;
}
```

The example is not a design; it's a prompt. When you look at `VecDeque`,
`pop_front` returns an `Option`. That `Option` is quietly asking you a design
question: *what does it mean for a worker to ask for work when there is none?* Hold
that question — you'll answer it with code soon.

If you want to see how experienced Rustaceans think about queue-shaped data, the
`std::collections::VecDeque` documentation is the right first read. You are **not**
committing to `VecDeque` as your design — you are studying the shape.

---

## Exercises

These are for *you*. The mentor will not solve them. Do them in your repository —
several are documentation, not code, and that's the point.

1. **Glossary (write it down).** Create `docs/glossary.md`. Define, in *your own
   words*: task, producer, queue, broker, worker, result backend. Where your
   definition differs from how you imagine Celery uses the term, note the difference.
   You are establishing OxideQueue's vocabulary.

2. **Scope statement.** In a file of your choosing (e.g., `docs/scope.md`), write two
   short lists: "OxideQueue *will* eventually..." and "OxideQueue will *not*...". Be
   ruthless. Every item you add is future work you're signing up for.

3. **Version Zero sketch.** On paper or in an ASCII diagram in your notes, draw the
   single-process Version Zero. Mark exactly where the producer, queue, and execution
   live. Circle the point where, later, a network might have to cut through the
   diagram.

4. **The "why a queue at all" essay (½ page).** Answer in writing: what does a queue
   buy you that a plain function call does not? Name at least two distinct benefits
   and one cost. (Hint to *think* about, not an answer: consider *time*, and consider
   *what happens under load*.)

5. **Read and annotate.** Skim the `std::collections::VecDeque` docs. Write three
   sentences: what operations does it offer, what are their costs, and what does its
   `pop_front` returning `Option` imply for a queue that might be empty?

6. **Record a decision.** Using `adr/0000-template.md`, create your first ADR — for
   example `adr/0001-start-single-process-in-memory.md` — capturing the decision to
   begin single-process and in-memory, with the alternatives (start distributed;
   start with a database) and the consequences. This is real practice for the habit
   we'll rely on all project long.

---

## Milestones

You have completed this chapter's milestone when:

- [x] `docs/glossary.md` exists and defines the core roles in your words.
- [x] A scope statement exists (will / will-not).
- [x] You have an ADR recording the single-process, in-memory starting point.
- [x] You can explain out loud, without notes, the path a task takes from producer to
      execution in Version Zero.
- [x] You have *not* written any queue implementation code yet — and you understand
      why that's the correct state to be in.

There is intentionally **no implementation milestone** this chapter. The deliverables
are understanding and design artifacts. Implementation begins once the shape is clear.

---

## Reflection Questions

Sit with these. They seed later chapters.

1. In Version Zero, the producer, queue, and worker share one process. Which of the
   three do you think will be the *first* to want to move out on its own, and why?
2. What is the difference between a task *failing* and a task *never being picked up*?
   How would a producer tell them apart?
3. If a worker takes a task out of the queue and then the whole program crashes, what
   happened to that task? Is that acceptable for Version Zero? For version one?
4. Celery separates the *broker* (transport) from the *result backend* (storage of
   outcomes). Why might a system want those to be two different things?
5. You're a QA/SDET by trade — a strength here. How would you *test* that Version Zero
   "works"? What's the observable behavior you'd assert on?

---

## Recommended Reading

- `std::collections::VecDeque` — the standard-library docs. Focus on the operations
  and their complexity, not on adopting it as your design.
- Celery's high-level "Introduction to Celery" documentation — read it for the
  *mental model and vocabulary* (producer/broker/worker/result backend), not for
  implementation details. We are borrowing concepts, not copying design.
- Optional, conceptual: any short write-up on "why use a message queue" /
  "temporal decoupling." You're reading for the *why*, not for a specific product.

**Prior art to keep in your peripheral vision (do not study deeply yet):** in the Rust
ecosystem, crates like `deadqueue` and the queue types inside async runtimes solve
"queue-shaped" problems. It's too early to read their source — but it's useful to know
that the problem you're modeling by hand is one the ecosystem has also modeled. We'll
point at specific modules to explore once you've built your own version and have
something to compare against.

---

## Common Mistakes

- **Building for the distributed future on day one.** Reaching for a network protocol
  or a database now means debugging infrastructure instead of learning the queue's
  shape. Resist it.
- **Confusing scope with architecture.** "It should be reliable" is a scope/quality
  goal. "It uses acknowledgements and retries" is an architecture. Don't smuggle
  solutions into your problem statement.
- **Inventing five words for one concept.** Decide now whether you say "job" or
  "task," "client" or "producer," and be consistent. Future-you will thank you.
- **Skipping the glossary because it feels trivial.** It is the cheapest artifact with
  the highest long-term payoff. The confusion it prevents is invisible precisely
  because you prevented it.
- **Treating an `Option` as an annoyance.** When `pop_front` hands you an `Option`,
  the language is asking you a real design question about emptiness. Answer it on
  purpose.

---

## Design Decisions I Must Make Myself

You — not the mentor — decide these. Record the significant ones as ADRs.

1. **Terminology.** What do you call a unit of work? A submitter? The holding area?
   Lock in the vocabulary for the whole project.
2. **Version Zero scope boundary.** Exactly how minimal is your first target? (E.g.,
   "submit N tasks that print a message, run them in order, in one process.")
3. **What 'a task' contains, conceptually.** Just a name? A name plus data? Do you
   even need data for Version Zero? (Don't over-design this yet — decide the minimum.)
4. **How you'll know it works.** Define the observable behavior you'll verify. Your
   QA instincts are an asset; use them to define acceptance now, before you build.

---

## Project Status

- **Phase:** Foundations. Vocabulary and mental model established; no implementation
  yet (by design).
- **Repository:** Documentation skeleton in place. This chapter adds the first
  narrative chapter. You will add `docs/glossary.md`, a scope note, and your first
  ADR as part of the exercises.
- **Code:** Untouched starter binary only. No queue code yet — this is the intended
  state at the end of Chapter 01.

## Current Architecture

There is, deliberately, no implemented architecture yet — only a chosen **starting
shape**:

```
   Version Zero (target of the next chapter, not built yet)

   ┌──────────────────────────────────────────────┐
   │                 one process                    │
   │   producer ──push──▶ [ queue ] ──pop──▶ run    │
   └──────────────────────────────────────────────┘
```

Single process, single thread, in memory, no network, no persistence. This is the
baseline every future decision will be measured against.

## Open Questions

- What does it mean for a worker to ask for work when the queue is empty?
- Where is the boundary between "broker" and "queue" for us — are they one thing at
  the start?
- How will we observe and verify correct behavior before any real logging exists?
- What is the smallest useful definition of "a task" for Version Zero?

## Pending Design Decisions (with alternatives)

- **Starting deployment shape.**
  - *Chosen (proposed to you):* single-process, in-memory.
  - *Alternative A:* start with separate processes over a network — rejected for now
    (fights the network before learning the core structure).
  - *Alternative B:* start with a database-backed queue — rejected for now (adds
    persistence and a dependency before we've felt the need).
- **Project vocabulary** — "task" vs "job", "producer" vs "client", etc. *You decide.*
- **Task representation** — name only, or name + data. *You decide the minimum.*

## Knowledge Checklist

- [x] I can explain what a task queue is and the problem it solves.
- [x] I can name and define producer, queue, broker, worker, and result backend.
- [x] I can explain temporal decoupling — why hand work off instead of doing it
      inline.
- [x] I can explain why "distributed" introduces the "was it lost or did it succeed?"
      ambiguity, and why we defer distribution.
- [x] I understand why Version Zero is intentionally a "list plus a loop."
- [x] I understand what `VecDeque::pop_front` returning `Option` is asking me to
      decide.

## Suggested Commits

- `docs: add Chapter 01 — introduction and mental model`
- `docs: add project glossary`
- `docs: add scope statement (will / will-not)`
- `adr: record decision to start single-process and in-memory`

## Suggested Git Tags

- `v0.0.2-ch01` — Chapter 01 written and foundational design artifacts committed.

## Suggested GitHub Milestone

- **Milestone: "Chapter 01 — Foundations & Vocabulary"**
  - Deliverables: glossary, scope statement, first ADR, and a written, defensible
    description of Version Zero. No queue implementation is part of this milestone.
