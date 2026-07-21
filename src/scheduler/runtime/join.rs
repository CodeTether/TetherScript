//! Ordered completion of a task collection.

use crate::value::{Runtime, Value};

use super::await_value;

pub(crate) fn values(runtime: &mut dyn Runtime, tasks: Vec<Value>) -> Result<Vec<Value>, String> {
    tasks
        .into_iter()
        .enumerate()
        .map(|(index, task)| {
            await_value(runtime, task).map_err(|error| format!("join task {index}: {error}"))
        })
        .collect()
}
