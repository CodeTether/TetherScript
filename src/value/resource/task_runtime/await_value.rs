//! Scheduled task execution and one-shot result transfer.

use crate::value::{Runtime, Value};

use super::super::task::{Handle, State};
use super::flatten;

impl Handle {
    pub(super) fn await_value(&mut self, runtime: &mut dyn Runtime) -> Result<Value, String> {
        let state = std::mem::replace(&mut self.state, State::Running);
        let (callee, args) = match state {
            State::Scheduled(callee, args) => (callee, args),
            State::Complete(value) => {
                self.state = State::Consumed;
                return Ok(value);
            }
            State::Failed(error) => {
                self.state = State::Failed(error.clone());
                return Err(error);
            }
            State::Pending => {
                self.state = State::Pending;
                return Err("task.await: backpressure: manual task is still pending".into());
            }
            State::Running => return Err("task.await: cyclic task dependency".into()),
            State::Consumed => {
                self.state = State::Consumed;
                return Err("task.await: task result was already consumed".into());
            }
        };
        let result = runtime
            .invoke_scheduled(&callee, &args)
            .and_then(|value| flatten::value(value, runtime));
        match result {
            Ok(value) => {
                self.state = State::Consumed;
                Ok(value)
            }
            Err(error) => {
                self.state = State::Failed(error.clone());
                Err(error)
            }
        }
    }
}
