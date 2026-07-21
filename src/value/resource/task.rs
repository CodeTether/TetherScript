//! Owned cooperative task completion handles.

use crate::value::Value;

use super::{result, transfer};

pub(super) struct Handle {
    pub(super) state: State,
}

pub(super) enum State {
    Pending,
    Complete(Value),
    Consumed,
}

impl Handle {
    pub(super) fn pending() -> Self {
        Self {
            state: State::Pending,
        }
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("complete", [value]) => Ok(result::nil(
                transfer::retained(value, "task.complete").and_then(|value| self.complete(value)),
            )),
            ("result", []) => Ok(result::value(self.result())),
            ("is_complete", []) => Ok(Value::Bool(matches!(self.state, State::Complete(_)))),
            _ => Err(format!(
                "task: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }
}
