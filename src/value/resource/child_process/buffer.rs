//! Shared bounded byte-buffer state for process pumps.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

pub(super) struct State {
    pub(super) bytes: VecDeque<u8>,
    pub(super) capacity: usize,
    pub(super) closed: bool,
    pub(super) failure: Option<String>,
}

#[derive(Clone)]
pub(super) struct Buffer(pub(super) Arc<(Mutex<State>, Condvar)>);

impl Buffer {
    pub(super) fn new(capacity: usize, label: &str) -> Result<Self, String> {
        if capacity == 0 {
            return Err(format!("{label}: capacity must be greater than zero"));
        }
        Ok(Self(Arc::new((
            Mutex::new(State {
                bytes: VecDeque::new(),
                capacity,
                closed: false,
                failure: None,
            }),
            Condvar::new(),
        ))))
    }

    pub(super) fn state(&self) -> MutexGuard<'_, State> {
        self.0
             .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn notify(&self) {
        self.0 .1.notify_all();
    }

    pub(super) fn close(&self) {
        self.state().closed = true;
        self.notify();
    }

    pub(super) fn fail(&self, error: String) {
        let mut state = self.state();
        state.failure = Some(error);
        state.closed = true;
        drop(state);
        self.notify();
    }
}
