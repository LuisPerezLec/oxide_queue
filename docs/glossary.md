# Glossary

* **Task**  
  A representation of a set of instructions to execute in order to complete a job.
* **Producer**  
  An actor responsible for creating and requesting task to be executed, handling them to a queue.
* **Queue**  
  A data structure which holds/stores elements in a FIFO order.
* **Broker**  
  An actor responsible for managing the task collection and providing them to workers.
* **Worker**  
  An actor responsible for executing the tasks that it recieves.
* **Result backend**  
  A representation (persistent or not) of the results obtained from the result of workers executing tasks