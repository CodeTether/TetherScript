//! Non-mutating inspection of the sending half.

use super::endpoint::Sender;
use super::registry;

impl Sender {
    /// Return the number of buffered values.
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
    /// let (tx, _rx) = channel::bounded(2, "len")?;
    /// tx.send(&Value::Int(1), 1)?;
    /// assert_eq!(tx.len(), 1);
    /// # Ok::<(), String>(())
    /// ```
    pub fn len(&self) -> usize {
        registry::with(self.id, |state| state.queue.len()).unwrap_or(0)
    }

    /// Return whether the buffer holds no values.
    ///
    /// # Returns
    ///
    /// `true` when nothing is buffered.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel;
    ///
    /// let (tx, _rx) = channel::bounded(2, "empty")?;
    /// assert!(tx.is_empty());
    /// # Ok::<(), String>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the fixed capacity bound that produces backpressure.
    ///
    /// # Returns
    ///
    /// The buffer bound chosen at construction. It never grows, which is what
    /// keeps a fast producer from turning into unbounded memory growth.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::scheduler::channel;
    ///
    /// let (tx, _rx) = channel::bounded(3, "cap")?;
    /// assert_eq!(tx.capacity(), 3);
    /// # Ok::<(), String>(())
    /// ```
    pub fn capacity(&self) -> usize {
        registry::with(self.id, |state| state.capacity).unwrap_or(0)
    }
}
