//! Receiving from a bounded channel, including end-of-stream detection.

use crate::value::Value;

use super::endpoint::Receiver;
use super::park::{self, ParkKind};
use super::{registry, wake};

/// Result of a non-blocking [`Receiver::recv`].
#[derive(Clone, Debug)]
pub enum RecvOutcome {
    /// A buffered value was transferred out of the channel.
    Value(Value),
    /// The channel is sealed *and* drained: no value will ever arrive.
    Ended,
    /// The buffer is empty but still live: the task is parked.
    Parked,
    /// The channel was already released by both halves.
    Gone,
}

impl Receiver {
    /// Take the oldest buffered value, or park `task` until one arrives.
    ///
    /// # Arguments
    ///
    /// * `task` — Id of the receiving task, recorded if the receive parks.
    ///
    /// # Returns
    ///
    /// [`RecvOutcome::Value`] in first-in first-out order; [`RecvOutcome::Ended`]
    /// once the channel is closed or sender-less *and* drained, so a close never
    /// discards already-buffered values; [`RecvOutcome::Parked`] while more
    /// values may still arrive; [`RecvOutcome::Gone`] if the channel was
    /// released entirely.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel::{self, RecvOutcome};
    /// use tetherscript::value::Value;
    ///
    /// let (tx, rx) = channel::bounded(2, "drain")?;
    /// tx.send(&Value::Int(1), 1)?;
    /// tx.close();
    /// // Buffered values survive the close, then end-of-stream is observed.
    /// assert!(matches!(rx.recv(2), RecvOutcome::Value(Value::Int(1))));
    /// assert!(matches!(rx.recv(2), RecvOutcome::Ended));
    /// # Ok::<(), String>(())
    /// ```
    pub fn recv(&self, task: u64) -> RecvOutcome {
        let Some(outcome) = registry::with(self.id, |state| {
            if let Some(value) = state.queue.pop_front() {
                return (Some(value), state.send_waiters.pop_front(), false);
            }
            (None, None, state.sealed())
        }) else {
            return RecvOutcome::Gone;
        };
        match outcome {
            (Some(value), waiter, _) => {
                if let Some(waiter) = waiter {
                    wake::wake(waiter);
                }
                RecvOutcome::Value(value)
            }
            (None, _, true) => RecvOutcome::Ended,
            (None, _, false) => {
                if park::park(task, self.id, ParkKind::Recv) {
                    registry::edit(self.id, |state| state.recv_waiters.push_back(task));
                }
                RecvOutcome::Parked
            }
        }
    }
}
