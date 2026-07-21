//! Runtime-driven scheduled task construction and inspection.

mod await_value;
mod construct;
mod flatten;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_lifecycle;

use crate::value::{Runtime, Value};

use super::{payload::Payload, task, OwnedResource};

impl OwnedResource {
    pub(crate) fn await_task(&mut self, runtime: &mut dyn Runtime) -> Result<Value, String> {
        if let Some(error) = self.unavailable("await") {
            return Err(error);
        }
        match self.payload.as_mut() {
            Some(Payload::Task(task)) => task.await_value(runtime),
            _ => Err(format!(
                "await: expected task resource, got {}",
                self.kind.type_name()
            )),
        }
    }

    pub(crate) fn task_ready(&self) -> bool {
        matches!(self.payload.as_ref(), Some(Payload::Task(task)) if task.ready())
    }
}

impl task::Handle {
    fn ready(&self) -> bool {
        matches!(
            self.state,
            task::State::Complete(_) | task::State::Failed(_)
        )
    }
}
