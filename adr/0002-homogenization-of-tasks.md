# ADR 0002: Homogenization of tasks

- **Status:** Proposed
- **Date:** 2026-07-05
- **Deciders:** Luis Pérez
- **Related chapter(s):** [`docs/02-modeling-a-task.md`](../docs/02-modeling-a-task.md)

## Context

It is important to decide whether our library is going to allow it's users define their own tasks, or if it will provide a set of fixed tasks that they are intended to implement, you can read more about the discussion [here](../docs/02-modeling-a-task.md#the-central-tension-data-vs-behavior).

## Decision

We will allow the library users to implement their own tasks, provisioning them with a specific Task type and a Runnable Trait, that they will have to manually implement themselves for their own created tasks.

## Alternatives Considered

- **Providing them with a predefined set of tasks** — This would allow the Tasks to be defined on a single Enum by the library, and then provide it to the users to implement. Nevertheless, this would reduce flexibility on the library, and it would not make it suitable for all kind of users

## Consequences
Providing a Task type and a Runnable Trait will require the list to use dynamic dispatch to allow different user defined tasks to be executed and pushed into the queue, this will introduce some runtime performance cost, and also consideration when handling this type in a collection.

### Positive

- Using a trait + trait objects to define behavior increases flexibility.

### Negative / Tradeoffs

- Handle of trait objects can be costly in terms of runtime performance and more complex to manage in a collection.