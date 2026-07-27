use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TaskError {
    #[error("Task execution failed")]
    ExecutionFailed,
}

#[derive(Debug, PartialEq)]
pub enum ExecutionState {
    Pending,
    Failure,
    Success
}

pub struct Task {
    pub state: ExecutionState,
    pub task: Box<dyn Runnable>
}

impl Task {
    pub fn new(task: Box<dyn Runnable>) -> Self {
        Self { state: ExecutionState::Pending, task }
    }

    pub fn run(&mut self) -> Result<(), TaskError> {
        match self.task.run() {
            Ok(_) => {
                self.state = ExecutionState::Success;
                Ok(())
            },
            Err(e) => {
                self.state = ExecutionState::Failure;
                Err(e)
            }
        }
    }
}

// Single thing a worker needs to do to any task
pub trait Runnable {
    fn run(&self) -> Result<(), TaskError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRunnable;

    impl Runnable for MockRunnable {
        fn run(&self) -> Result<(), TaskError> {
            Ok(())
        }
    }

    #[test]
    fn create_task(){
        let mock_runnable = MockRunnable;
        let task = Task::new(Box::new(mock_runnable));
        assert_eq!(task.state, ExecutionState::Pending);
    }

    #[test]
    fn run_task_with_error(){
        struct ErrorRunnable;

        impl Runnable for ErrorRunnable {
            fn run(&self) -> Result<(), TaskError> {
                Err(TaskError::ExecutionFailed)
            }
        }

        let error_runnable = ErrorRunnable;
        let mut task = Task::new(Box::new(error_runnable));
        assert_eq!(task.run(), Err(TaskError::ExecutionFailed));
        assert_eq!(task.state, ExecutionState::Failure);
    }
}