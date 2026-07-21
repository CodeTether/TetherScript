//! Owned bounded request-body readers.

use std::collections::VecDeque;

use crate::value::Value;

use super::{args, result};

pub(super) struct Handle {
    bytes: VecDeque<u8>,
    capacity: usize,
}

impl Handle {
    pub(super) fn new(bytes: Vec<u8>, capacity: usize) -> Result<Self, String> {
        if bytes.len() > capacity {
            return Err(format!(
                "resource.request_body: body length {} exceeds capacity {capacity}",
                bytes.len()
            ));
        }
        Ok(Self {
            bytes: bytes.into(),
            capacity,
        })
    }

    pub(super) fn call(&mut self, name: &str, values: &[Value]) -> Result<Value, String> {
        match (name, values) {
            ("read", [limit]) => Ok(result::value(self.read(limit))),
            ("remaining", []) => Ok(Value::Int(self.bytes.len() as i64)),
            ("capacity", []) => Ok(Value::Int(self.capacity as i64)),
            _ => Err(format!(
                "request_body: no method `{name}` accepting {} arguments",
                values.len()
            )),
        }
    }

    fn read(&mut self, limit: &Value) -> Result<Value, String> {
        let limit = args::usize(limit, "request_body.read limit")?;
        let count = limit.min(self.bytes.len());
        let bytes: Vec<u8> = self.bytes.drain(..count).collect();
        Ok(Value::Bytes(std::rc::Rc::new(std::cell::RefCell::new(
            bytes,
        ))))
    }
}
