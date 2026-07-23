# ADR 0001: < Start Single Process in memory>

- **Status:** Proposed
- **Date:** 2026-07-21
- **Deciders:** <@LuisPerezLec>
- **Related chapter(s):** `docs/01-introduction.md`

## Context

This file represents the scope and limitations if the Version Zero.

## Decision

For version zero, we are not aiming to provide a library instance (no database connections).
We will model a Task struct, with some fields (including one for the task status, e.g.: Done, Pending, Failed), and the trait that will allow for homogenization of the tasks.

```
  ┌-----------------┐    Push      ┌----------------┐    Pop    ┌---------------┐
  | Producer living |  ──────────> |  List storing  | --------> | Execution     | 
  | inside the bin  |              |    dyn type    |           | (worker)      |
  | crate using lib |              |living in binary|           | inside binary |
  | types and traits|              └----------------┘           └---------------┘
  └-----------------┘                                                   |
                                                                        v
                                                                ┌---------------┐
                                  Posible network cut --------> | Future Result |
                                                                |    backend    |
                                                                └---------------┘
```

## Alternatives Considered

- **Alternative A** — Start distributed; Starting with a distributed system would end in bigger efforts on debugging infrastructure instead of learning the queue's shape.
- **Alternative B** — Start with a database; Starting with a persistent database would end in bigger efforts on debugging infrastructure instead of learning the queue's shape.

## Consequences

### Positive

- We can focus on Task struct, traits and methods, and on building our queues core behavior.

### Negative / Tradeoffs

- We have to keep the project scalable in order for later development to be feasible and do not have to turn back to solve design isues.

### Neutral / Follow-ups

- Codify Version Zero project

## References

- [Chapter 1](../docs/01-introduction.md)
