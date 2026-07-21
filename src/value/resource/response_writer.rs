//! Owned bounded response writers.

use crate::value::Value;

use super::result;

pub(super) struct Handle {
    pub(super) bytes: Vec<u8>,
    pub(super) capacity: usize,
}

impl Handle {
    pub(super) fn new(capacity: usize) -> Result<Self, String> {
        if capacity == 0 {
            return Err("resource.response_writer: capacity must be greater than zero".into());
        }
        Ok(Self {
            bytes: Vec::new(),
            capacity,
        })
    }

    pub(super) fn call(&mut self, name: &str, values: &[Value]) -> Result<Value, String> {
        match (name, values) {
            ("write", [body]) => Ok(result::value(self.write(body))),
            ("body", []) => Ok(self.body()),
            ("len", []) => Ok(Value::Int(self.bytes.len() as i64)),
            ("capacity", []) => Ok(Value::Int(self.capacity as i64)),
            _ => Err(format!(
                "response_writer: no method `{name}` accepting {} arguments",
                values.len()
            )),
        }
    }
}
