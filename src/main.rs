use std::collections::VecDeque;
use oxide_queue::{Runnable, Task, TaskError};

fn main() {
    let calculator = Calculator::new(1, 2, add_numbers);
    let messager = Messager::new("Hello, world!");

    let task = Task::new(Box::new(messager));
    let task2 = Task::new(Box::new(calculator));

    let mut queue: VecDeque<Task> = VecDeque::new();
    queue.push_back(task);
    queue.push_back(task2);

    while let Some(mut task) = queue.pop_front() {
        match task.run() {
            Ok(_) => println!("Task completed successfully, task status: {:?}", task.state),
            Err(e) => println!("Error executing task: {}", e)
        }
    }
}

struct Messager {
    message: String
}

impl Messager {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }
}

impl Runnable for Messager {
    fn run(&self) -> Result<(), TaskError> {
        println!("{}", self.message);
        Ok(())
    }
}

struct Calculator {
    first_number: i32,
    second_number: i32,
    operation: fn(a: i32, b: i32) -> Result<i32, TaskError>
}

fn add_numbers(a: i32, b: i32) -> Result<i32, TaskError> {
    println!("{} + {} is {}", a, b, a+b);
    Ok(a+b)
}

impl Calculator {
    fn new(first_number: i32, second_number: i32, operation: fn(a: i32, b: i32)-> Result<i32, TaskError>) -> Self {
        Self {
            first_number,
            second_number,
            operation
        }
    }
}

impl Runnable for Calculator {
    fn run(&self) -> Result<(), TaskError> {
        (self.operation)(self.first_number, self.second_number)?;
        Ok(())
    }
}