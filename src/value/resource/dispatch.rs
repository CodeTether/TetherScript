//! Shared lifecycle method dispatch.

use crate::value::Value;

use super::{result, OwnedResource};

impl OwnedResource {
    pub(crate) fn call(&mut self, name: &str, values: &[Value]) -> Result<Value, String> {
        if let Some(value) = self.control_call(name, values) {
            return value;
        }
        if let Some(error) = self.unavailable(name) {
            return Ok(result::value(Err(error)));
        }
        let deadline = self.deadline.instant();
        self.payload
            .as_mut()
            .expect("open resource must retain its payload")
            .call(name, values, deadline)
    }
}
