//! Consumer operations for process stream buffers.

use super::buffer::Buffer;

impl Buffer {
    pub(super) fn read(&self, limit: usize, label: &str) -> Result<Vec<u8>, String> {
        if limit == 0 {
            return Err(format!("{label}: limit must be greater than zero"));
        }
        let mut state = self.state();
        if state.bytes.is_empty() {
            if let Some(error) = &state.failure {
                return Err(error.clone());
            }
            return if state.closed {
                Ok(Vec::new())
            } else {
                Err(format!("{label}: backpressure: no bytes are ready"))
            };
        }
        let count = limit.min(state.bytes.len());
        let bytes = state.bytes.drain(..count).collect();
        drop(state);
        self.notify();
        Ok(bytes)
    }

    pub(super) fn pop_blocking(&self, limit: usize) -> Option<Vec<u8>> {
        let mut state = self.state();
        while state.bytes.is_empty() && !state.closed {
            state = self
                .0
                 .1
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if state.bytes.is_empty() {
            return None;
        }
        let count = limit.min(state.bytes.len());
        let bytes = state.bytes.drain(..count).collect();
        drop(state);
        self.notify();
        Some(bytes)
    }

    pub(super) fn is_eof(&self) -> bool {
        self.state().closed
    }
}
