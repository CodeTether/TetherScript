//! # The borrow table
//!
//! A side table mapping [`HeapId`] to [`BorrowState`]. It is a *side* table
//! rather than a field on `Value` for two reasons:
//!
//! * `src/value.rs` is load-bearing and must not grow a per-variant field; and
//! * only heap values are ever entered, so unborrowed Copy scalars cost nothing.
//!
//! Entries are created lazily on first borrow and can be dropped by
//! [`BorrowTable::forget`] when the allocation dies, so the table does not grow
//! without bound.

use std::collections::HashMap;

use super::heap::HeapId;
use super::state::BorrowState;

/// Borrow bookkeeping for every heap value currently under observation.
///
/// # Examples
///
/// ```rust
/// use tetherscript::borrow_runtime::{BorrowKind, BorrowTable, HeapId};
///
/// let mut table = BorrowTable::default();
/// let id = HeapId(0x1000);
///
/// table.acquire(id, BorrowKind::Shared, "items").unwrap();
/// assert!(table.acquire(id, BorrowKind::Mutable, "items").is_err());
/// table.release(id, BorrowKind::Shared, "items").unwrap();
/// assert!(table.acquire(id, BorrowKind::Mutable, "items").is_ok());
/// ```
#[derive(Debug, Default)]
pub struct BorrowTable {
    states: HashMap<HeapId, BorrowState>,
    accounting_faults: usize,
}

impl BorrowTable {
    /// Read the state of one heap value without creating an entry.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    ///
    /// # Returns
    ///
    /// `Some(&BorrowState)` if the value has ever been borrowed or moved,
    /// otherwise `None` (which is semantically the default state).
    pub fn state(&self, id: HeapId) -> Option<&BorrowState> {
        self.states.get(&id)
    }

    /// Get or create the mutable state for one heap value.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    ///
    /// # Returns
    ///
    /// A mutable reference to the (possibly freshly defaulted) state.
    pub fn state_mut(&mut self, id: HeapId) -> &mut BorrowState {
        self.states.entry(id).or_default()
    }

    /// Number of tracked heap values.
    ///
    /// # Returns
    ///
    /// The entry count, useful for asserting the table does not leak.
    pub fn tracked(&self) -> usize {
        self.states.len()
    }

    /// Drop the entry for a heap value.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value whose allocation is gone.
    ///
    /// # Returns
    ///
    /// The removed state, if there was one.
    pub fn forget(&mut self, id: HeapId) -> Option<BorrowState> {
        self.states.remove(&id)
    }

    /// Count of unbalanced releases observed during guard drops.
    ///
    /// # Returns
    ///
    /// How many times a guard's `Drop` found no matching borrow. A non-zero
    /// value is an internal defect, not a script error; `Drop` cannot return an
    /// error, so it is tallied here instead of being swallowed.
    pub fn accounting_faults(&self) -> usize {
        self.accounting_faults
    }

    /// Record an unbalanced release seen in a destructor.
    pub fn note_accounting_fault(&mut self) {
        self.accounting_faults += 1;
    }
}
