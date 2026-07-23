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