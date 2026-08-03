//! # Tracker handle
//!
//! [`BorrowTracker`] is an `Rc<RefCell<BorrowTable>>` handle. Both backends hold
//! one; guards hold a clone of it so their `Drop` can reach the table without
//! borrowing the interpreter. Cloning is a refcount bump, so handing a tracker to
//! a closure, a spawned task, or a native call is free.
//!
//! `RefCell` is used rather than a lock because tetherscript's runtime is
//! single-threaded and cooperative (see `src/scheduler/`); the core build has
//! zero dependencies and adds none here.

use std::cell::RefCell;
use std::rc::Rc;

use super::error::BorrowError;
use super::heap::HeapId;
use super::table::BorrowTable;

/// Shared, cheaply cloneable handle to a [`BorrowTable`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::borrow_runtime::{BorrowKind, BorrowTracker, HeapId};
///
/// let tracker = BorrowTracker::new();
/// let id = HeapId(0x2000);
///
/// // A closure capture is just another handle onto the same table.
/// let captured = tracker.clone();
/// let guard = tracker.borrow_value(id, BorrowKind::Mutable, "cell").unwrap();
/// assert!(captured.borrow_value(id, BorrowKind::Shared, "cell").is_err());
/// drop(guard);
/// assert!(captured.borrow_value(id, BorrowKind::Shared, "cell").is_ok());
/// ```
#[derive(Debug, Clone, Default)]
pub struct BorrowTracker(Rc<RefCell<BorrowTable>>);

impl BorrowTracker {
    /// Create an empty tracker.
    ///
    /// # Returns
    ///
    /// A tracker with no tracked values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::BorrowTracker;
    ///
    /// let tracker = BorrowTracker::new();
    /// assert_eq!(tracker.with_table(|table| table.tracked()), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Run a closure against the underlying table.
    ///
    /// # Arguments
    ///
    /// * `f` — receives the table mutably.
    ///
    /// # Returns
    ///
    /// Whatever `f` returns.
    ///
    /// # Panics
    ///
    /// Panics if the table is already mutably borrowed, which would mean a
    /// reentrant call from inside another `with_table` closure. Keep those
    /// closures short and non-reentrant.
    pub fn with_table<R>(&self, f: impl FnOnce(&mut BorrowTable) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }

    /// Mark a heap value as moved out of `binding`.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    /// * `binding` — identifier being moved from.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the move is permitted and recorded.
    ///
    /// # Errors
    ///
    /// [`BorrowError::MoveWhileBorrowed`] if a borrow is still live.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::{BorrowTracker, HeapId};
    ///
    /// let tracker = BorrowTracker::new();
    /// tracker.mark_moved(HeapId(1), "xs").unwrap();
    /// assert!(tracker.is_moved(HeapId(1)));
    /// ```
    pub fn mark_moved(&self, id: HeapId, binding: &str) -> Result<(), BorrowError> {
        self.with_table(|table| table.mark_moved(id, binding))
    }

    /// Whether a heap value has been moved out.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    ///
    /// # Returns
    ///
    /// `true` if the value is a moved tombstone.
    pub fn is_moved(&self, id: HeapId) -> bool {
        self.with_table(|table| table.state(id).is_some_and(|state| state.is_moved()))
    }

    /// Forget a heap value's bookkeeping once its allocation is gone or its
    /// binding was rebound to a fresh value.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the dead allocation.
    pub fn forget(&self, id: HeapId) {
        self.with_table(|table| {
            table.forget(id);
        });
    }
}
