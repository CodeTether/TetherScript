//! # Guard-producing borrow entry point
//!
//! The API the backends should actually call: it acquires and hands back a
//! [`BorrowGuard`], so releasing is the compiler's job rather than the caller's
//! discipline. Kept apart from `borrow_runtime_tracker.rs` to avoid a module
//! cycle between the tracker and the guard while keeping each file one concern.

use super::error::BorrowError;
use super::guard::BorrowGuard;
use super::heap::HeapId;
use super::kind::BorrowKind;
use super::tracker::BorrowTracker;

impl BorrowTracker {
    /// Borrow a heap value, returning a guard that releases on drop.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value, from [`HeapId::from_rc`].
    /// * `kind` — [`BorrowKind::Shared`] for `&x`, [`BorrowKind::Mutable`] for
    ///   `&mut x`.
    /// * `binding` — the identifier the script wrote at the borrow site. It goes
    ///   verbatim into the error message, so pass the user's name, not a
    ///   synthetic temporary.
    ///
    /// # Returns
    ///
    /// A [`BorrowGuard`] holding the borrow for as long as it lives.
    ///
    /// # Errors
    ///
    /// * [`BorrowError::Conflict`] — XOR-mutability would be violated.
    /// * [`BorrowError::Moved`] — the value was already moved out.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::{BorrowKind, BorrowTracker, HeapId};
    ///
    /// let tracker = BorrowTracker::new();
    /// let id = HeapId(0x40);
    ///
    /// let a = tracker.borrow_value(id, BorrowKind::Shared, "items").unwrap();
    /// let b = tracker.borrow_value(id, BorrowKind::Shared, "items").unwrap();
    /// let err = tracker
    ///     .borrow_value(id, BorrowKind::Mutable, "items")
    ///     .unwrap_err();
    /// assert_eq!(
    ///     err.to_string(),
    ///     "cannot mutably borrow `items` while it is already borrowed"
    /// );
    ///
    /// drop((a, b));
    /// assert!(tracker.borrow_value(id, BorrowKind::Mutable, "items").is_ok());
    /// ```
    pub fn borrow_value(
        &self,
        id: HeapId,
        kind: BorrowKind,
        binding: &str,
    ) -> Result<BorrowGuard, BorrowError> {
        self.with_table(|table| table.acquire(id, kind, binding))?;
        Ok(BorrowGuard::new(
            self.clone(),
            id,
            kind,
            binding.to_string(),
        ))
    }
}
