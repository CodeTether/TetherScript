//! Scheduled task state and execution tests.

use crate::value::{Runtime, Value};

use super::OwnedResource;

#[derive(Default)]
struct MockRuntime {
    calls: usize,
}

impl Runtime for MockRuntime {
    fn invoke(&mut self, _callee: &Value, args: &[Value]) -> Result<Value, String> {
        self.calls += 1;
        Ok(args.first().cloned().unwrap_or(Value::Nil))
    }
}

#[test]
fn scheduled_task_is_lazy_and_one_shot() {
    let mut task = OwnedResource::scheduled_task(Value::Nil, vec![Value::Int(7)]).unwrap();
    let mut runtime = MockRuntime::default();
    assert!(!task.task_ready());
    assert_eq!(runtime.calls, 0);
    assert_eq!(task.await_task(&mut runtime).unwrap(), Value::Int(7));
    assert_eq!(runtime.calls, 1);
    assert!(task
        .await_task(&mut runtime)
        .unwrap_err()
        .contains("consumed"));
}

#[test]
fn completed_spawn_value_is_ready_before_await() {
    let mut task = OwnedResource::completed_task(Value::Int(9)).unwrap();
    let mut runtime = MockRuntime::default();
    assert!(task.task_ready());
    assert_eq!(task.await_task(&mut runtime).unwrap(), Value::Int(9));
    assert_eq!(runtime.calls, 0);
}
