//! Sending into a bounded channel, with backpressure and endpoint checks.

use crate::value::resource::transfer;
use crate::value::Value;

use super::endpoint::Sender;
use super::park::{self, ParkKind};
use super::{registry, send_guard, wake};

/// Result of a non-blocking [`Sender::send`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendOutcome {
    /// The value was buffered and any waiting receiver was queued for wakeup.
    Sent,
    /// The buffer was full: the task is parked and must retry once woken.
    Parked,
}

impl Sender {
    /// Buffer `value`, or park `task` when the bounded buffer is full.
    ///
    /// # Arguments
    ///
    /// * `value` — Value to transfer into the channel; ownership is validated.
    /// * `task` — Id of the sending task, recorded if the send parks.
    ///
    /// # Returns
    ///
    /// [`SendOutcome::Sent`] once buffered, or [`SendOutcome::Parked`] when the
    /// capacity bound is reached. Parking rather than growing is deliberate: an
    /// unbounded buffer would turn a fast producer into unbounded memory use, so
    /// the backpressure is the point of the type.
    ///
    /// # Errors
    ///
    /// Returns `Err` naming the channel when it was closed, when every receiver
    /// has been dropped (a park would deadlock, since nothing can drain it), or
    /// when the value cannot be transferred by the ownership rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel::{self, SendOutcome};
    /// use tetherscript::value::Value;
    ///
    /// let (tx, rx) = channel::bounded(1, "jobs")?;
    /// assert_eq!(tx.send(&Value::Int(1), 1)?, SendOutcome::Sent);
    /// assert_eq!(tx.send(&Value::Int(2), 1)?, SendOutcome::Parked);
    /// drop(rx);
    /// assert!(tx.send(&Value::Int(3), 1).is_err());
    /// # Ok::<(), String>(())
    /// ```
    pub fn send(&self, value: &Value, task: u64) -> Result<SendOutcome, String> {
        let value = transfer::retained(value, "channel.send")?;
        if send_guard::check(self.id)? {
            if park::park(task, self.id, ParkKind::Send) {
                registry::edit(self.id, |state| state.send_waiters.push_back(task));
            }
            return Ok(SendOutcome::Parked);
        }
        let waiter = registry::with(self.id, |state| {
            state.queue.push_back(value);
            state.recv_waiters.pop_front()
        })
        .flatten();
        if let Some(waiter) = waiter {
            wake::wake(waiter);
        }
        Ok(SendOutcome::Sent)
    }
}
