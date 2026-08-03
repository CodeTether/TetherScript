//! # Move interaction
//!
//! The two halves of the move/borrow contract, which AGENTS.md lists as
//! load-bearing and which `src/ownership.rs` already enforces statically:
//!
//! 1. **A borrowed value must not be movable.** `mark_moved` refuses while any
//!    borrow is live, with the same sentence the static pass emits.
//! 2. **A moved value must not be borrowable afterwards.** Once `moved` is set,
//!    `acquire` fails with `cannot borrow moved value` — see
//!    `borrow_runtime_acquire.rs`.
//!
//! Setting `moved` only when no borrow is live is also what keeps the invariant
//! `!(moved && borrowed)` true by construction.

use super::error::BorrowError;
use super::state::BorrowState;

impl BorrowState {
    /// Record that the value was moved out of its binding.
    ///
    /// # Arguments
    ///
    /// * `binding` — the identifier being moved from, for the error message.
    ///
    /// # Returns
    ///
    /// `Ok(())` with the value marked moved and therefore unborrowable.
    ///
    /// # Errors
    ///
    /// [`BorrowError::MoveWhileBorrowed`] when a borrow is still live. This is
    /// the runtime twin of the static `cannot move ... while it is borrowed`
    /// diagnostic, and the wording is identical so users see one rule.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::{BorrowKind, BorrowState};
    ///
    /// let mut state = BorrowState::default();
    /// state.acquire(BorrowKind::Shared, "xs").unwrap();
    /// assert!(state.mark_moved("xs").is_err());
    ///
    /// state.release(BorrowKind::Shared, "xs").unwrap();
    /// state.mark_moved("xs").unwrap();
    /// assert!(state.acquire(BorrowKind::Shared, "xs").is_err());
    /// ```
    pub fn mark_moved(&mut self, binding: &str) -> Result<(), BorrowError> {
        if self.is_borrowed() {
            return Err(BorrowError::MoveWhileBorrowed {
                binding: binding.to_string(),
            });
        }
        self.moved = true;
        Ok(())
    }
}
