# Roadmap Rules (How to Write Chapters)

> This document defines the required structure and constraints for every chapter of
> the OxideQueue learning guide.

## Book-Like Experience

The roadmap should feel like reading an entire technical book. It is split into many
Markdown files stored in `docs/`, written **one chapter at a time**.

## Chapter Files

Chapters are stored in `docs/` using a zero-padded numeric prefix:

```
docs/00-roadmap.md
docs/01-introduction.md
docs/02-architecture.md
...
```

- `docs/00-roadmap.md` is the high-level table of contents and evolving plan.
- Each subsequent chapter is a self-contained unit of learning.

## Required Chapter Sections

Every chapter should include, where applicable:

- **Objectives**
- **Concepts to learn**
- **Why those concepts matter**
- **Architectural discussions**
- **Diagrams using ASCII**
- **Exercises**
- **Milestones**
- **Reflection questions**
- **Recommended reading**
- **Common mistakes**
- **Design decisions I must make myself**

## Required Chapter Ending

At the **end of every chapter**, always include:

- **Project status**
- **Current architecture**
- **Open questions**
- **Pending design decisions** (with alternatives when convenient)
- **Knowledge checklist**
- **Suggested commits**
- **Suggested Git tags**
- **Suggested GitHub milestone**

## Chapter Constraints

- Every chapter must end in a **stable stopping point** where the developer can spend
  several days implementing before requesting the next chapter.
- Do **not** reference future chapters in detail.
- Only introduce concepts that become **necessary at the current stage**.
- Do **not** contradict previous chapters.
- One chapter is written per request. **Do not pre-generate future chapters.**

## Tone and Method

- Teach through questions where possible (see `methodology.md`).
- Push the developer to think rather than giving away implementations (see
  `constraints.md`).
- Connect concepts to real crates and prior art in the Rust ecosystem when relevant.
