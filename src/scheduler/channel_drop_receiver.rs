//! Receiver retirement so sends fail by name instead of parking forever.

use std::collections::VecDeque;

use super::endpoint::Receiver;
use super::{drop_endpoint, registry, wake};

impl Drop for Receiver {
    /// Retire one receiver and wake senders so their sends fail rather than hang.
    ///
    /// When the *last* receiver goes, nothing can ever drain the buffer, so any
    /// parked sender must be released to discover that and report a named error.
    fn drop(&mut self) {
        let waiters = registry::with(self.id, |state| {
            state.receivers = state.receivers.saturating_sub(1);
            if state.receivers == 0 {
                return std::mem::take(&mut state.send_waiters);
            }
            VecDeque::new()
        })
        .unwrap_or_default();
        for waiter in waiters {
            wake::wake(waiter);
        }
        drop_endpoint::release(self.id);
    }
}
