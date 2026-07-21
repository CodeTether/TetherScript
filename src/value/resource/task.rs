//! Owned cooperative task completion handles.

use crate::value::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{result, transfer};

pub(super) struct Handle {
    pub(super) id: u64,
    pub(super) state: State,
}

pub(super) enum State {
    Pending,
    Scheduled(Value, Vec<Value>),
    Running,
    Complete(Value),
    Failed(String),
    Consumed,
}

pub(super) static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl Handle {
    pub(super) fn pending() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            state: State::Pending,
        }
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("complete", [value]) => Ok(result::nil(
                transfer::retained(value, "task.complete").and_then(|value| self.complete(value)),
            )),
            ("result", []) => Ok(result::value(self.result())),
            ("id", []) => Ok(Value::Int(self.id as i64)),
            ("state", []) => Ok(Value::Str(std::rc::Rc::new(self.state.label().into()))),
            ("is_complete", []) => Ok(Value::Bool(self.state.is_complete())),
            _ => Err(format!(
                "task: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }
}
