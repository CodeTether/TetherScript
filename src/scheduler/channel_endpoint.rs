//! Sender and receiver halves of a bounded channel.
//!
//! Both halves are thin, cheap handles: each stores only the channel id, and the
//! buffer plus waiter queues live in the channel registry. Cloning a half
//! registers one more live endpoint and dropping it retires one — that count is
//! what makes "all senders gone" and "all receivers gone" observable instead of
//! turning into a permanent park.

use super::registry;

/// The sending half of a bounded channel.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel;
/// use tetherscript::value::Value;
///
/// let (tx, _rx) = channel::bounded(2, "orders")?;
/// assert_eq!(tx.capacity(), 2);
/// tx.send(&Value::Int(7), 1)?;
/// assert_eq!(tx.len(), 1);
/// # Ok::<(), String>(())
/// ```
#[derive(Debug)]
pub struct Sender {
    pub(super) id: u64,
}

/// The receiving half of a bounded channel.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{self, RecvOutcome};
/// use tetherscript::value::Value;
///
/// let (tx, rx) = channel::bounded(2, "orders")?;
/// tx.send(&Value::Int(7), 1)?;
/// assert!(matches!(rx.recv(2), RecvOutcome::Value(Value::Int(7))));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug)]
pub struct Receiver {
    pub(super) id: u64,
}

impl Clone for Sender {
    /// Register one more live sender on the same channel.
    fn clone(&self) -> Self {
        registry::edit(self.id, |state| state.senders += 1);
        Self { id: self.id }
    }
}

impl Clone for Receiver {
    /// Register one more live receiver on the same channel.
    fn clone(&self) -> Self {
        registry::edit(self.id, |state| state.receivers += 1);
        Self { id: self.id }
    }
}
