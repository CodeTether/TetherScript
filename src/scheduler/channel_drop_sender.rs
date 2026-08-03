//! Sender retirement so receivers observe end-of-stream instead of hanging.

use std::collections::VecDeque;

use super::endpoint::Sender;
use super::{drop_endpoint, registry, wake};

impl Drop for Sender {
    /// Retire one sender and wake receivers so they can observe end-of-stream.
    ///
    /// When the *last* sender goes, a receive that would otherwise park forever
    /// must instead complete: buffered values still drain, and the empty channel
    /// then reports end-of-stream.
    fn drop(&mut self) {
        let waiters = registry::with(self.id, |state| {
            state.senders = state.senders.saturating_sub(1);
            if state.senders == 0 {
                return std::mem::take(&mut state.recv_waiters);
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
