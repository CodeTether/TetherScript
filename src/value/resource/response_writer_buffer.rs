//! Bounded response-buffer operations.

use crate::value::Value;

use super::{args, response_writer::Handle};

impl Handle {
    pub(super) fn write(&mut self, body: &Value) -> Result<Value, String> {
        let bytes = args::bytes(body, "response_writer.write body")?;
        if self.bytes.len().saturating_add(bytes.len()) > self.capacity {
            return Err(format!(
                "response_writer.write: backpressure: capacity {} exceeded",
                self.capacity
            ));
        }
        self.bytes.extend_from_slice(&bytes);
        Ok(Value::Int(bytes.len() as i64))
    }

    pub(super) fn body(&self) -> Value {
        Value::Bytes(std::rc::Rc::new(std::cell::RefCell::new(
            self.bytes.clone(),
        )))
    }
}
