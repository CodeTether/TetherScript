//! # RAII borrow guard
//!
//! [`BorrowGuard`] owns one live borrow. Its `Drop` releases it, so an early
//! `return`, a propagated `?`, or an unwind cannot leak a count.
//!
//! ## Why a leaked count is worse than a missed check
//!
//! A missed check is a false *negative*: one unsound program runs. A leaked
//! count is a false *positive* that persists — the value stays wedged as
//! "borrowed" for the rest of the process, so every later borrow of it fails
//! even though the later code is correct. The reported binding is innocent, the
//! real culprit is a borrow site that already returned, and the error is
//! therefore un-debuggable from the message alone. That is why release is never
//! left to hand-written pairing at a call site that can exit early.
//!
//! `Drop` cannot return an error, so an unbalanced release discovered during a
//! drop is tallied via [`super::table::BorrowTable::note_accounting_fault`] and
//! surfaced by `accounting_faults()`, rather than panicking during an unwind
//! (which would abort).

use super::heap::HeapId;
use super::kind::BorrowKind;
use super::tracker::BorrowTracker;

/// A live borrow that releases itself when dropped.
///
/// # Examples
///
/// ```rust
/// use tetherscript::borrow_runtime::{BorrowKind, BorrowTracker, HeapId};
///
/// let tracker = BorrowTracker::new();
/// let id = HeapId(0x30);
///
/// // An early return drops the guard, so the borrow does not leak.
/// fn bail(tracker: &BorrowTracker, id: HeapId) -> Result<(), String> {
///     let _guard = tracker.borrow_value(id, BorrowKind::Mutable, "items").unwrap();
///     Err("early exit".to_string())
/// }
///
/// assert!(bail(&tracker, id).is_err());
/// assert!(tracker.borrow_value(id, BorrowKind::Mutable, "items").is_ok());
/// ```
#[derive(Debug)]
pub struct BorrowGuard {
    tracker: BorrowTracker,
    id: HeapId,
    kind: BorrowKind,
    binding: String,
}

impl BorrowGuard {
    /// Construct a guard for an already-acquired borrow.
    ///
    /// # Arguments
    ///
    /// * `tracker` — handle used to release on drop.
    /// * `id` — identity of the borrowed heap value.
    /// * `kind` — the borrow kind that was acquired.
    /// * `binding` — identifier named at the borrow site.
    ///
    /// # Returns
    ///
    /// A guard that will release `kind` on `id` when dropped.
    pub(super) fn new(
        tracker: BorrowTracker,
        id: HeapId,
        kind: BorrowKind,
        binding: String,
    ) -> Self {
        Self {
            tracker,
            id,
            kind,
            binding,
        }
    }

    /// Which kind of borrow this guard holds.
    ///
    /// # Returns
    ///
    /// The [`BorrowKind`] acquired.
    pub fn kind(&self) -> BorrowKind {
        self.kind
    }

    /// Identity of the borrowed heap value.
    ///
    /// # Returns
    ///
    /// The [`HeapId`] this guard holds a borrow of.
    pub fn heap_id(&self) -> HeapId {
        self.id
    }

    /// Binding named at the borrow site.
    ///
    /// # Returns
    ///
    /// The script-visible identifier.
    pub fn binding(&self) -> &str {
        &self.binding
    }
}

impl Drop for BorrowGuard {
    fn drop(&mut self) {
        let released = self
            .tracker
            .with_table(|table| table.release(self.id, self.kind, &self.binding));
        if released.is_err() {
            self.tracker
                .with_table(|table| table.note_accounting_fault());
        }
    }
}
