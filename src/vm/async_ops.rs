//! VM stack operations for cooperative tasks.

use crate::scheduler::runtime;
use crate::value::Runtime;

use super::{resource_transfer, Unwind, VM};

impl VM {
    pub(super) fn spawn_top(&mut self) -> Result<(), Unwind> {
        let value = self.stack.pop().expect("Spawn with empty stack");
        self.stack.push(runtime::spawn(value)?);
        Ok(())
    }

    pub(super) fn await_top(&mut self) -> Result<(), Unwind> {
        let task = self.stack.pop().expect("Await with empty stack");
        let value = runtime::await_value(self as &mut dyn Runtime, task)?;
        self.stack.push(value);
        Ok(())
    }

    pub(super) fn join_top(&mut self, count: usize) -> Result<(), Unwind> {
        let start = self
            .stack
            .len()
            .checked_sub(count)
            .ok_or_else(|| Unwind::Error("Join task count exceeded the VM stack".into()))?;
        let tasks = self.stack.drain(start..).collect();
        let values = runtime::join(self as &mut dyn Runtime, tasks)?;
        self.stack.push(resource_transfer::list(values)?);
        Ok(())
    }
}
