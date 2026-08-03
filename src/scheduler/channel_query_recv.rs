//! Non-mutating inspection of the receiving half.

use super::endpoint::Receiver;
use super::registry;

impl Receiver {
    /// Return the number of buffered values still available to drain.
    ///
    /// # Returns
    ///
    /// Current buffer occupancy, or `0` once the channel has been released.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel;
    /// use tetherscript::value::Value;
    ///
    /// let (tx, rx) = channel::bounded(2, "len")?;
    /// tx.send(&Value::Int(1), 1)?;
    /// assert_eq!(rx.len(), 1);
    /// # Ok::<(), String>(())
    /// ```
    pub fn len(&self) -> usize {
        registry::with(self.id, |state| state.queue.len()).unwrap_or(0)
    }

    /// Return whether no value is currently buffered.
    ///
    /// # Returns
    ///
    /// `true` when a receive would park or report end-of-stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel;
    ///
    /// let (_tx, rx) = channel::bounded(2, "empty")?;
    /// assert!(rx.is_empty());
    /// # Ok::<(), String>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return whether the channel is sealed *and* fully drained.
    ///
    /// # Returns
    ///
    /// `true` only when no value remains and none can arrive, so a close never
    /// makes still-buffered values look like end-of-stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel;
    /// use tetherscript::value::Value;
    ///
    /// let (tx, rx) = channel::bounded(2, "ended")?;
    /// tx.send(&Value::Int(1), 1)?;
    /// tx.close();
    /// assert!(!rx.is_ended());
    /// # Ok::<(), String>(())
    /// ```
    pub fn is_ended(&self) -> bool {
        registry::with(self.id, |state| state.ended()).unwrap_or(true)
    }
}
