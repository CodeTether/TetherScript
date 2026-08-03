//! Explicit sender close with drain-before-end semantics.

use super::endpoint::Sender;
use super::{registry, wake};

impl Sender {
    /// Seal the channel against further sends while preserving buffered values.
    ///
    /// Closing is deliberately *not* a discard. The classic bug is to treat
    /// close as "stop the channel now", which loses every value the producer
    /// already handed over. Here close only forbids new sends; a receiver keeps
    /// draining the buffer and observes end-of-stream afterwards.
    ///
    /// # Returns
    ///
    /// Nothing. Repeated closes are harmless, and every receiver parked on the
    /// empty buffer is queued for wakeup so it can observe end-of-stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel::{self, RecvOutcome};
    /// use tetherscript::value::Value;
    ///
    /// let (tx, rx) = channel::bounded(2, "closing")?;
    /// tx.send(&Value::Int(1), 1)?;
    /// tx.send(&Value::Int(2), 1)?;
    /// tx.close();
    /// tx.close();
    /// assert!(matches!(rx.recv(2), RecvOutcome::Value(Value::Int(1))));
    /// assert!(matches!(rx.recv(2), RecvOutcome::Value(Value::Int(2))));
    /// assert!(matches!(rx.recv(2), RecvOutcome::Ended));
    /// # Ok::<(), String>(())
    /// ```
    pub fn close(&self) {
        let waiters = registry::with(self.id, |state| {
            state.closed = true;
            std::mem::take(&mut state.recv_waiters)
        })
        .unwrap_or_default();
        for waiter in waiters {
            wake::wake(waiter);
        }
    }
}
