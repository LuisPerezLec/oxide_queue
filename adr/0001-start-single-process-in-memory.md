# ADR 0001: < Start Single Process in memory>

- **Status:** Proposed
- **Date:** 2026-07-21
- **Deciders:** <@LuisPerezLec>
- **Related chapter(s):** `docs/01-introduction.md`

## Context

This file represents the scope and limitations if the Version Zero.

## Decision

We will start the project with a single thread in memory process, that will make use of a FIFO data structure, and will be limited to creating and running tasks defined on the binary.

## Alternatives Considered

- **Alternative A** — Start distributed; Starting with a distributed system would end in bigger efforts on debugging insfrastructure instead of learning the queue's shape.
- **Alternative B** — Start with a database; Starting with a persistent database would end in bigger efforts on debugging insfrastructure instead of learning the queue's shape.

## Consequences

### Positive

- We can focus on Task struct, traits and methods, and on building our queues core behavior.

### Negative / Tradeoffs

- We have to keep the project scalable in order for later development to be feasible and do not have to turn back to solve design isues.

### Neutral / Follow-ups

- Codificate Version Zero project

## References

- [Chapter 1](../docs/01-introduction.md)
