//! Owned bounded channels with explicit backpressure.

use std::collections::VecDeque;

use crate::value::Value;

use super::{result, transfer};

pub(super) struct Handle {
    pub(super) queue: VecDeque<Value>,
    pub(super) capacity: usize,
}

impl Handle {
    pub(super) fn bounded(capacity: usize) -> Result<Self, String> {
        if capacity == 0 {
            return Err("resource.channel: capacity must be greater than zero".into());
        }
        Ok(Self {
            queue: VecDeque::new(),
            capacity,
        })
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("send", [value]) => Ok(result::nil(
                transfer::retained(value, "channel.send").and_then(|value| self.send(value)),
            )),
            ("recv", []) => Ok(result::value(self.recv())),
            ("len", []) => Ok(Value::Int(self.queue.len() as i64)),
            ("capacity", []) => Ok(Value::Int(self.capacity as i64)),
            ("is_full", []) => Ok(Value::Bool(self.queue.len() == self.capacity)),
            _ => Err(format!(
                "channel: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }
}
