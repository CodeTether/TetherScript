//! One-shot task completion and owned result transfer.

use crate::value::Value;

use super::task::{Handle, State};

impl Handle {
    pub(super) fn complete(&mut self, value: Value) -> Result<(), String> {
        match self.state {
            State::Pending => {
                self.state = State::Complete(value);
                Ok(())
            }
            State::Complete(_) => Err("task.complete: task already completed".into()),
            State::Scheduled(_, _) | State::Running => {
                Err("task.complete: scheduled task controls its own completion".into())
            }
            State::Failed(_) => Err("task.complete: scheduled task failed".into()),
            State::Consumed => Err("task.complete: task result was already consumed".into()),
        }
    }

    pub(super) fn result(&mut self) -> Result<Value, String> {
        match &self.state {
            State::Pending => Err("task.result: backpressure: task is still pending".into()),
            State::Scheduled(_, _) | State::Running => {
                Err("task.result: backpressure: scheduled task has not been awaited".into())
            }
            State::Failed(error) => Err(error.clone()),
            State::Consumed => Err("task.result: task result was already consumed".into()),
            State::Complete(_) => match std::mem::replace(&mut self.state, State::Consumed) {
                State::Complete(value) => Ok(value),
                _ => unreachable!("task state changed during result transfer"),
            },
        }
    }
}
