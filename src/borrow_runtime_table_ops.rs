//! # Table-level acquire / release / move
//!
//! Thin forwarding from a [`BorrowTable`] entry to the per-value rules in
//! `borrow_runtime_acquire.rs` and `borrow_runtime_move.rs`. Separated so the
//! table's storage concern and its rule-application concern are distinct files.

use super::error::BorrowError;
use super::heap::HeapId;
use super::kind::BorrowKind;
use super::table::BorrowTable;

impl BorrowTable {
    /// Take a borrow of one heap value.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    /// * `kind` — shared or mutable.
    /// * `binding` — identifier named at the borrow site.
    ///
    /// # Returns
    ///
    /// `Ok(())` with the borrow recorded.
    ///
    /// # Errors
    ///
    /// Whatever [`super::state::BorrowState::acquire`] returns: a conflict or a
    /// moved-value error, both naming `binding`.
    pub fn acquire(
        &mut self,
        id: HeapId,
        kind: BorrowKind,
        binding: &str,
    ) -> Result<(), BorrowError> {
        self.state_mut(id).acquire(kind, binding)
    }

    /// Release a borrow of one heap value.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    /// * `kind` — must match the acquired kind.
    /// * `binding` — identifier, for the error message.
    ///
    /// # Returns
    ///
    /// `Ok(())` with the borrow removed.
    ///
    /// # Errors
    ///
    /// [`BorrowError::UnbalancedRelease`] if no such borrow was live.
    pub fn release(
        &mut self,
        id: HeapId,
        kind: BorrowKind,
        binding: &str,
    ) -> Result<(), BorrowError> {
        self.state_mut(id).release(kind, binding)
    }

    /// Mark one heap value as moved out of `binding`.
    ///
    /// # Arguments
    ///
    /// * `id` — identity of the heap value.
    /// * `binding` — identifier being moved from.
    ///
    /// # Returns
    ///
    /// `Ok(())` with the value marked moved.
    ///
    /// # Errors
    ///
    /// [`BorrowError::MoveWhileBorrowed`] if a borrow is still live.
    pub fn mark_moved(&mut self, id: HeapId, binding: &str) -> Result<(), BorrowError> {
        self.state_mut(id).mark_moved(binding)
    }
}
