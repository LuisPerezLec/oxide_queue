# Repository Rules (README, docs, structure)

> This document defines the repository layout and the conventions for documentation
> and information architecture (IA).

## Repository Layout

```
oxide_queue/
├── README.md                 # Project overview, structure, learning methodology
├── Cargo.toml                # (maintained by the developer, not the mentor)
├── src/                      # Source code (written by the developer)
├── docs/                     # The learning guide, one chapter per file
│   ├── 00-roadmap.md
│   ├── 01-introduction.md    # (created on request, one chapter at a time)
│   └── ...
├── adr/                      # Architecture Decision Records
│   ├── 0000-template.md      # ADR template (reference)
│   ├── 0001-....md
│   └── ...
└── design-process/           # Meta-documentation: how this project is mentored
    ├── mentor-system.md      # Mentor role
    ├── mentor-user.md        # User profile
    ├── methodology.md        # How to teach
    ├── constraints.md        # What to avoid
    ├── architecture-rules.md # ADRs, evolution
    ├── roadmap-rules.md      # How to write chapters
    └── repository-rules.md   # README, docs, structure (this file)
```

## Documentation Information Architecture (IA)

- **`design-process/`** — the *meta* layer. Explains how the project is mentored and
  taught. Read this to understand the operating contract, not the queue itself.
- **`docs/`** — the *learning* layer. The narrative guide that walks through building
  OxideQueue chapter by chapter.
- **`adr/`** — the *decision* layer. Dated, immutable records of concrete
  architectural choices.

This separation lets a reader navigate by intent:

- "How is this taught?" → `design-process/`
- "How do I build it, step by step?" → `docs/`
- "Why was this decided?" → `adr/`

## README Requirements

The `README.md` must:

- Explain the project.
- Explain the repository structure.
- Explain how to leverage the information architecture.
- Include the **Learning Methodology** disclaimer describing the AI mentor's
  constraints.

## Cargo / Build Files

- The mentor must **not** generate or modify `Cargo.toml`. The developer owns all
  build configuration and implementation code.
