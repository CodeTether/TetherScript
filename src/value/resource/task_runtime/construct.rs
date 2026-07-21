//! Ownership-checked scheduled task construction.

use std::sync::atomic::Ordering;

use crate::value::Value;

use super::super::{payload::Payload, task, transfer, OwnedResource};

impl OwnedResource {
    pub(crate) fn scheduled_task(callee: Value, args: Vec<Value>) -> Result<Self, String> {
        for argument in &args {
            transfer::validate(argument, "async task argument")?;
        }
        Ok(Self::new(Payload::Task(task::Handle::scheduled(
            callee, args,
        ))))
    }

    pub(crate) fn completed_task(value: Value) -> Result<Self, String> {
        transfer::validate(&value, "spawn value")?;
        Ok(Self::new(Payload::Task(task::Handle::completed(value))))
    }
}

impl task::Handle {
    fn scheduled(callee: Value, args: Vec<Value>) -> Self {
        Self {
            id: task::NEXT_ID.fetch_add(1, Ordering::Relaxed),
            state: task::State::Scheduled(callee, args),
        }
    }

    fn completed(value: Value) -> Self {
        Self {
            id: task::NEXT_ID.fetch_add(1, Ordering::Relaxed),
            state: task::State::Complete(value),
        }
    }
}
