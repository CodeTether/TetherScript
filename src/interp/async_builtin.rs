//! Runtime-aware cooperative scheduling built-ins.

use crate::scheduler::runtime;
use crate::value::{Runtime, Value};

pub(super) fn select(runtime_host: &mut dyn Runtime, arguments: &[Value]) -> Result<Value, String> {
    let Value::List(tasks) = &arguments[0] else {
        return Err(format!(
            "select: expected a list of tasks, got {}",
            arguments[0].type_name()
        ));
    };
    let tasks = tasks.borrow().clone();
    runtime::select(runtime_host, &tasks)
}
