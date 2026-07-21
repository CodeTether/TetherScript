//! Owned cooperative task completion handles.

use crate::value::Value;

use super::result;

pub(super) struct Handle {
    result: Option<Value>,
}

impl Handle {
    pub(super) fn pending() -> Self {
        Self { result: None }
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("complete", [value]) => Ok(result::nil(self.complete(value.clone()))),
            ("result", []) => Ok(result::value(self.result())),
            ("is_complete", []) => Ok(Value::Bool(self.result.is_some())),
            _ => Err(format!(
                "task: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }

    fn complete(&mut self, value: Value) -> Result<(), String> {
        if self.result.is_some() {
            return Err("task.complete: task already completed".into());
        }
        self.result = Some(value);
        Ok(())
    }

    fn result(&self) -> Result<Value, String> {
        self.result
            .clone()
            .ok_or_else(|| "task.result: backpressure: task is still pending".into())
    }
}
