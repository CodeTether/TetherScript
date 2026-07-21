//! Scheduled task cancellation, deadline, and identity tests.

use std::time::Duration;

use crate::value::{Runtime, Value};

use super::OwnedResource;

struct NoopRuntime;

impl Runtime for NoopRuntime {
    fn invoke(&mut self, _callee: &Value, _args: &[Value]) -> Result<Value, String> {
        Ok(Value::Nil)
    }
}

#[test]
fn cancellation_and_deadlines_block_task_execution() {
    let mut cancelled = OwnedResource::completed_task(Value::Int(1)).unwrap();
    cancelled.cancel().unwrap();
    let error = cancelled.await_task(&mut NoopRuntime).unwrap_err();
    assert!(
        error.contains("task.await: resource is cancelled"),
        "{error}"
    );

    let mut expired = OwnedResource::completed_task(Value::Int(2)).unwrap();
    expired.set_deadline_after(Duration::ZERO);
    let error = expired.await_task(&mut NoopRuntime).unwrap_err();
    assert!(error.contains("task.await: deadline exceeded"), "{error}");
}

#[test]
fn task_ids_are_stable_and_unique() {
    let mut first = OwnedResource::completed_task(Value::Nil).unwrap();
    let mut second = OwnedResource::completed_task(Value::Nil).unwrap();
    let first_id = first.call("id", &[]).unwrap();
    assert_eq!(first.call("id", &[]).unwrap(), first_id);
    assert_ne!(second.call("id", &[]).unwrap(), first_id);
}
