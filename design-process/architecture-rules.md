# Architecture Rules (ADRs & Evolution)

> This document defines how architectural decisions are recorded and how the
> architecture is allowed to evolve over time.

## Source of Truth

The files in `docs/`, `adr/`, and `design-process/` represent the **current state**
of the project. Treat them as the source of truth. Do not contradict previous
chapters.

## The Roadmap May Evolve

The roadmap is intentionally allowed to evolve. New knowledge gained while building
the project may reveal that an earlier architectural decision is no longer
appropriate.

## When an Earlier Decision No Longer Fits

If new knowledge reveals that an earlier architectural decision is no longer
appropriate, **do not silently modify previous chapters**. Instead:

1. Explain **why** the previous decision no longer fits.
2. Create a **new ADR that supersedes** the previous one.
3. Explain the **migration path**.
4. Update the **project status** accordingly.

Preserve the historical evolution of the project. Superseded ADRs remain in the
repository, marked as superseded, with a link to the ADR that replaces them.

## Architecture Decision Records (ADRs)

- ADRs live in the `adr/` folder.
- Each ADR is a numbered Markdown file (e.g., `adr/0001-title.md`).
- Use the template in `adr/0000-template.md` as the basis for every new ADR.
- ADRs are immutable once accepted, except for status changes (e.g., moving from
  `Accepted` to `Superseded`) and adding links to superseding ADRs.

## ADR Lifecycle / Status Values

- **Proposed** — under discussion, not yet decided.
- **Accepted** — the decision is in effect.
- **Superseded** — replaced by a newer ADR (link to it).
- **Deprecated** — no longer relevant, but not directly replaced.
- **Rejected** — considered but not adopted (kept for the historical record).

## Relationship to Chapters

- Chapters (`docs/`) narrate the learning journey and reasoning.
- ADRs (`adr/`) record the concrete, dated decisions that result from that reasoning.
- When a chapter drives a significant, hard-to-reverse decision, it should suggest
  capturing that decision as an ADR.
