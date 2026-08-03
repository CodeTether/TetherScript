//! Mutable state of one bounded channel.
//!
//! One [`ChannelState`] holds the buffer, the endpoint counts, the close flag,
//! and the parked-waiter queues for a single channel. Keeping all of it in one
//! plain struct (rather than in the sender and receiver handles) is what lets the
//! cooperative scheduler reason about every channel uniformly when it looks for
//! deadlock.

use std::collections::VecDeque;

use crate::value::Value;

/// Buffer, endpoint counts, and waiter queues for one bounded channel.
pub(super) struct ChannelState {
    /// Diagnostic name reported in every error message.
    pub(super) name: String,
    /// Fixed buffer bound. Never grows; this is the backpressure limit.
    pub(super) capacity: usize,
    /// Buffered values in first-in, first-out order.
    pub(super) queue: VecDeque<Value>,
    /// Live sender handles.
    pub(super) senders: usize,
    /// Live receiver handles.
    pub(super) receivers: usize,
    /// Set by an explicit sender close.
    pub(super) closed: bool,
    /// Task ids parked because the buffer was full, oldest first.
    pub(super) send_waiters: VecDeque<u64>,
    /// Task ids parked because the buffer was empty, oldest first.
    pub(super) recv_waiters: VecDeque<u64>,
}

impl ChannelState {
    pub(super) fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            capacity,
            queue: VecDeque::new(),
            senders: 1,
            receivers: 1,
            closed: false,
            send_waiters: VecDeque::new(),
            recv_waiters: VecDeque::new(),
        }
    }

    /// True when no further value fits without growing the buffer.
    pub(super) fn is_full(&self) -> bool {
        self.queue.len() >= self.capacity
    }

    /// True when no new value can ever arrive.
    pub(super) fn sealed(&self) -> bool {
        self.closed || self.senders == 0
    }

    /// True when the channel is sealed *and* fully drained.
    pub(super) fn ended(&self) -> bool {
        self.sealed() && self.queue.is_empty()
    }
}
