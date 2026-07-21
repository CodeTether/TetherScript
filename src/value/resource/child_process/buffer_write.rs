//! Producer operations for process stream buffers.

use super::buffer::Buffer;

impl Buffer {
    pub(super) fn try_write(&self, bytes: &[u8], label: &str) -> Result<usize, String> {
        let mut state = self.state();
        if let Some(error) = &state.failure {
            return Err(error.clone());
        }
        if state.closed {
            return Err(format!("{label}: stream is closed"));
        }
        if bytes.len() > state.capacity.saturating_sub(state.bytes.len()) {
            return Err(format!(
                "{label}: backpressure: capacity {} exceeded",
                state.capacity
            ));
        }
        state.bytes.extend(bytes);
        drop(state);
        self.notify();
        Ok(bytes.len())
    }

    pub(super) fn push_blocking(&self, bytes: &[u8]) {
        let mut offset = 0;
        while offset < bytes.len() {
            let mut state = self.state();
            while state.bytes.len() == state.capacity && !state.closed {
                state = self
                    .0
                     .1
                    .wait(state)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
            if state.closed {
                return;
            }
            let count = (state.capacity - state.bytes.len()).min(bytes.len() - offset);
            state.bytes.extend(&bytes[offset..offset + count]);
            offset += count;
            drop(state);
            self.notify();
        }
    }

    pub(super) fn capacity(&self) -> usize {
        self.state().capacity
    }
}
