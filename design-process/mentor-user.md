# User Profile (Who I Am)

> This document describes the developer being mentored. The mentor should calibrate
> explanations, pacing, and assumptions to this profile.

## Background

- Software Engineer.
- ~1 year of experience as a fullstack developer (mostly web).
- ~1 year of experience currently performing as an SDET / QA Automation Engineer.

## Goal

Transition into a **Rust backend or systems programming role** — becoming a
proficient Rust engineer able to make a living as a Rust developer.

## Rust Learning So Far

Completed a core Rust Udemy course and read *The Rust Programming Language* book.

Already comfortable with:

- ownership
- borrowing
- lifetimes
- slices
- structs
- traits
- enums
- generics
- `Result` and `Option`
- modules
- crates
- Cargo workspaces
- testing
- integration testing
- error handling with `anyhow` and `thiserror` (briefly)
- basic concurrency (from the book)
- basic async concepts (from the book)
- channels
- `Arc`
- `Mutex`
- smart pointers (`Box`, `Rc`, `RefCell`)

## Never Built / Never Used

- A large Rust application.
- Tokio
- Axum
- SQLx
- defining procedural macros
- defining declarative macros
- tracing
- tower

## Celery Experience

- Has **never used** Celery.
- Understands what Celery is **conceptually**, but has never built an application
  with it.

## Mentoring Implications

- The developer does **not** need Rust syntax lessons; they need to learn how to
  build and reason about a large system in idiomatic Rust.
- Distributed systems concepts must be introduced progressively and cannot be
  assumed.
- Async Rust, Tokio, and the broader ecosystem (Axum, SQLx, tracing, tower) are new
  and should be introduced only when the project naturally requires them.
- Macros (declarative and procedural) are new and should be motivated by a concrete
  need before being taught.
