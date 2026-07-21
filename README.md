# OxideQueue

A **distributed task queue inspired by Celery**, built from scratch in idiomatic
Rust — as a deep, long-form learning project.

OxideQueue is **not** a feature-for-feature clone of Celery. It is a ground-up
exploration of how a production-quality task queue works: how clients submit units of
work, how workers execute them asynchronously, and how a real system handles
serialization, networking, persistence, concurrency, observability, and reliability.

The project is built **one chapter at a time**, guided by an AI mentor, with every
significant architectural decision recorded as an ADR.

---

## Learning Methodology

This project intentionally uses an AI mentor.

The AI is instructed to:

- never generate implementation code on its own
- never solve exercises
- progressively introduce concepts
- act as a technical mentor

All architecture decisions, implementation, debugging, and code are my own.

---

## Why This Project Exists

The goal is to transition into Rust backend / systems programming by building
something substantial. A distributed task queue is an excellent vehicle because it
naturally forces you to confront:

- **async execution** (Tokio)
- **inter-process communication** (serialization + networking)
- **concurrency** (worker pools, synchronization)
- **durability** (persistence / storage)
- **observability** (tracing, structured logging)
- **reliability** (acknowledgements, retries, scheduling, failure handling)

Each of these emerges from a concrete problem encountered while building, never as an
abstract lesson.

---

## Repository Structure

```
oxide_queue/
├── README.md                 # You are here
├── Cargo.toml                # Build configuration (owned by me)
├── src/                      # Source code (written by me)
├── docs/                     # The learning guide — one chapter per file
│   ├── 00-roadmap.md         # Evolving high-level plan + chapter index
│   └── NN-*.md               # Chapters, added one at a time
├── adr/                      # Architecture Decision Records
│   └── 0000-template.md      # ADR template (reference)
└── design-process/           # Meta-docs: how this project is mentored
    ├── mentor-system.md      # Mentor role
    ├── mentor-user.md        # My developer profile
    ├── methodology.md        # How the mentor teaches
    ├── constraints.md        # What the mentor must avoid
    ├── architecture-rules.md # ADRs and architectural evolution
    ├── roadmap-rules.md      # How chapters are written
    └── repository-rules.md   # Repo/documentation conventions
```

---

## Information Architecture (How to Navigate)

The documentation is organized by **intent**, so you can find what you need quickly:

| I want to know...                     | Look in            |
|---------------------------------------|--------------------|
| How is this project taught / mentored?| `design-process/`  |
| How do I build it, step by step?      | `docs/`            |
| Why was a particular decision made?   | `adr/`             |

- **`design-process/`** — the *meta* layer: the operating contract for the mentor and
  the developer profile.
- **`docs/`** — the *learning* layer: the narrative guide that builds OxideQueue
  chapter by chapter. Start with `docs/00-roadmap.md`.
- **`adr/`** — the *decision* layer: dated, immutable records of concrete
  architectural choices. Superseded decisions are preserved and marked, never
  silently rewritten.

---

## Current Status

- **Phase:** Scaffolding complete.
- **Chapters written:** None yet (only the roadmap exists).
- **Code:** Starter binary only.
- **Next step:** Begin with Chapter 01 (Introduction).

See [`docs/00-roadmap.md`](docs/00-roadmap.md) for the evolving plan and chapter
index.

---

## A Note on Authorship

The narrative guide, ADRs, and mentoring instructions are produced collaboratively
with an AI mentor whose role is strictly limited (see **Learning Methodology** above).
The **implementation** — all Rust code, architecture decisions, debugging, and
exercises — is mine.
