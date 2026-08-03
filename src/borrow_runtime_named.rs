//! # Value-level convenience API
//!
//! What the tree-walker and the VM call at an `Expr::Borrow` / `Expr::BorrowMut`
//! site: hand it a `Value` and the binding name, get back an
//! `Option<BorrowGuard>`. `None` means "Copy scalar, nothing to track" — not
//! "unchecked".

use crate::value::Value;

use super::error::BorrowError;
use super::guard::BorrowGuard;
use super::kind::BorrowKind;
use super::tracker::BorrowTracker;
use super::value::heap_id_of;

impl BorrowTracker {
    /// Borrow a `Value` by binding name.
    ///
    /// # Arguments
    ///
    /// * `value` — the value being borrowed.
    /// * `kind` — shared for `&x`, mutable for `&mut x`.
    /// * `binding` — the identifier the script wrote.
    ///
    /// # Returns
    ///
    /// `Ok(Some(guard))` for heap values, `Ok(None)` for Copy scalars.
    ///
    /// # Errors
    ///
    /// [`BorrowError::Conflict`] or [`BorrowError::Moved`], naming `binding`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{cell::RefCell, rc::Rc};
    /// use tetherscript::borrow_runtime::{BorrowKind, BorrowTracker};
    /// use tetherscript::value::Value;
    ///
    /// let tracker = BorrowTracker::new();
    /// let items = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
    ///
    /// // Scalars are Copy, so no guard is produced.
    /// assert!(tracker
    ///     .borrow_named(&Value::Int(7), BorrowKind::Mutable, "n")
    ///     .unwrap()
    ///     .is_none());
    ///
    /// // A dynamically created alias shares the borrow state.
    /// let alias = items.clone();
    /// let held = tracker
    ///     .borrow_named(&items, BorrowKind::Shared, "items")
    ///     .unwrap();
    /// assert!(held.is_some());
    /// let err = tracker
    ///     .borrow_named(&alias, BorrowKind::Mutable, "alias")
    ///     .unwrap_err();
    /// assert_eq!(
    ///     err.to_string(),
    ///     "cannot mutably borrow `alias` while it is already borrowed"
    /// );
    /// ```
    pub fn borrow_named(
        &self,
        value: &Value,
        kind: BorrowKind,
        binding: &str,
    ) -> Result<Option<BorrowGuard>, BorrowError> {
        match heap_id_of(value) {
            None => Ok(None),
            Some(id) => self.borrow_value(id, kind, binding).map(Some),
        }
    }

    /// Mark a `Value` moved out of `binding`.
    ///
    /// # Arguments
    ///
    /// * `value` — the value leaving its slot.
    /// * `binding` — the identifier being moved from.
    ///
    /// # Returns
    ///
    /// `Ok(())`; a Copy scalar is a no-op because moving one clones it.
    ///
    /// # Errors
    ///
    /// [`BorrowError::MoveWhileBorrowed`] if a borrow of the heap value is live.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::{cell::RefCell, rc::Rc};
    /// use tetherscript::borrow_runtime::{BorrowKind, BorrowTracker};
    /// use tetherscript::value::Value;
    ///
    /// let tracker = BorrowTracker::new();
    /// let xs = Value::List(Rc::new(RefCell::new(Vec::new())));
    ///
    /// let guard = tracker.borrow_named(&xs, BorrowKind::Shared, "xs").unwrap();
    /// assert_eq!(
    ///     tracker.move_named(&xs, "xs").unwrap_err().to_string(),
    ///     "cannot move `xs` while it is borrowed"
    /// );
    ///
    /// drop(guard);
    /// tracker.move_named(&xs, "xs").unwrap();
    /// assert_eq!(
    ///     tracker
    ///         .borrow_named(&xs, BorrowKind::Shared, "xs")
    ///         .unwrap_err()
    ///         .to_string(),
    ///     "cannot borrow moved value `xs`"
    /// );
    /// ```
    pub fn move_named(&self, value: &Value, binding: &str) -> Result<(), BorrowError> {
        match heap_id_of(value) {
            None => Ok(()),
            Some(id) => self.mark_moved(id, binding),
        }
    }
}
