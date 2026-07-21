//! Bounded channel queue operations.

use crate::value::Value;

use super::channel::Handle;

impl Handle {
    pub(super) fn send(&mut self, value: Value) -> Result<(), String> {
        if self.queue.len() == self.capacity {
            return Err(format!(
                "channel.send: backpressure: capacity {} reached",
                self.capacity
            ));
        }
        self.queue.push_back(value);
        Ok(())
    }

    pub(super) fn recv(&mut self) -> Result<Value, String> {
        self.queue
            .pop_front()
            .ok_or_else(|| "channel.recv: backpressure: channel is empty".into())
    }
}
