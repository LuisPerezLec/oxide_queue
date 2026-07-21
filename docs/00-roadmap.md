# OxideQueue — Roadmap

> This is the evolving, high-level plan for the OxideQueue learning guide. It is a
> **map, not a contract**. The roadmap is intentionally allowed to change as we learn
> more (see `design-process/architecture-rules.md`).
>
> Chapters are written **one at a time**, on request. Only `docs/00-roadmap.md`
> exists until a chapter is explicitly requested. Nothing below should be read as a
> detailed spec of a future chapter — it is a direction of travel.

## What We Are Building

A **distributed task queue inspired by Celery**, written in idiomatic Rust, named
**OxideQueue**.

We are **not** cloning Celery feature-by-feature. We are building a system that lets
clients submit units of work ("tasks") to be executed asynchronously by one or more
**workers**, with reliability, observability, and production concerns introduced
progressively — only when a concrete limitation forces them.

## Guiding Principles

- **Learning first, production second.** Prefer a simpler implementation when it
  teaches more. Adopt production-grade libraries only when they become the better
  engineering choice.
- **Concepts emerge from problems.** No concept is introduced before the project has
  a concrete reason to need it.
- **Complexity must be justified.** Each new component must be motivated by a
  limitation discovered in the previous step.
- **Preserve history.** Decisions are recorded as ADRs; superseded decisions are kept
  and marked, never silently rewritten.

## Conceptual Learning Arc (subject to change)

This arc shows the *kinds* of problems we expect to encounter and the concepts they
will naturally introduce. Ordering and grouping into chapters will be decided as we
go.

```
   Foundations
   ┌───────────────────────────────────────────────────────────┐
   │ vocabulary & mental model of a task queue                  │
   │ project shape: library vs binary crates, workspaces        │
   └───────────────────────────────────────────────────────────┘
                              │
                              ▼
   In-process execution
   ┌───────────────────────────────────────────────────────────┐
   │ background execution        → async Rust, Tokio            │
   │ producer/consumer handoff   → channels                     │
   │ heterogeneous task types    → traits / trait objects       │
   └───────────────────────────────────────────────────────────┘
                              │
                              ▼
   Crossing process boundaries
   ┌───────────────────────────────────────────────────────────┐
   │ exchanging data             → serialization                │
   │ talking between processes   → networking / protocol        │
   └───────────────────────────────────────────────────────────┘
                              │
                              ▼
   Durability & scale
   ┌───────────────────────────────────────────────────────────┐
   │ surviving restarts          → persistence / storage        │
   │ many workers                → synchronization, worker pools │
   └───────────────────────────────────────────────────────────┘
                              │
                              ▼
   Production concerns
   ┌───────────────────────────────────────────────────────────┐
   │ seeing what happens         → tracing, structured logging  │
   │ not losing work             → acks, retries, scheduling     │
   │ operability & reliability   → backpressure, failure modes   │
   └───────────────────────────────────────────────────────────┘
```

The Rust ecosystem topics that are new to the developer — **Tokio, Axum, SQLx,
tracing, tower, and macros (declarative and procedural)** — will be introduced only
at the point in this arc where the project genuinely needs them, with a justification
for why building it by hand is no longer the learning objective.

## Chapter Index

Chapters are added here as they are written.

| #  | File                        | Title        | Status      |
|----|-----------------------------|--------------|-------------|
| 00 | `docs/00-roadmap.md`        | Roadmap      | Living doc  |
| 01 | `docs/01-introduction.md`   | Introduction | Written     |
| 02 | *(to be written on request)*| —            | Not written |

> Chapter 01 is written. Request the next chapter when you have worked through its
> milestone and are ready to begin implementing Version Zero.

## How to Use This Guide

1. Read `README.md` for the project overview and learning methodology.
2. Read `design-process/` to understand how this project is mentored.
3. Work through `docs/` chapters in order, one at a time.
4. Record significant, hard-to-reverse decisions as ADRs in `adr/`.

## Project Status

- **Phase:** Foundations. Chapter 01 (Introduction) written; vocabulary and mental
  model established. No queue implementation yet (by design).
- **Repository:** Documentation skeleton in place (`README.md`, `docs/`, `adr/`,
  `design-process/`), plus the first narrative chapter (`docs/01-introduction.md`).
- **Code:** Untouched starter binary only.
- **Next step:** Work through the Chapter 01 milestone (glossary, scope statement,
  first ADR, defensible Version Zero description), then request the next chapter.
