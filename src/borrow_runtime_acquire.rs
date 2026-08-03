//! # Acquire and release rules
//!
//! The two operations that implement XOR-mutability on a single
//! [`BorrowState`]. Kept in their own file so the rule itself is readable
//! without scrolling past the accessors.
//!
//! Both return a [`BorrowError`] rather than panicking: only the caller (the
//! tree-walker or the VM) knows the source location to attach, and both
//! backends already thread `Result<_, String>` through their error reporting.

use super::error::BorrowError;
use super::kind::BorrowKind;
use super::state::BorrowState;

impl BorrowState {
    /// Take a borrow.
    ///
    /// # Arguments
    ///
    /// * `kind` — [`BorrowKind::Shared`] for `&x`, [`BorrowKind::Mutable`] for
    ///   `&mut x`.
    /// * `binding` — the identifier the script wrote, used in the error message.
    ///
    /// # Returns
    ///
    /// `Ok(())` with the borrow recorded in the state.
    ///
    /// # Errors
    ///
    /// * [`BorrowError::Moved`] if the value was moved out.
    /// * [`BorrowError::Conflict`] if a mutable borrow is requested while any
    ///   borrow is live, or a shared borrow is requested while a mutable borrow
    ///   is live.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::{BorrowKind, BorrowState};
    ///
    /// let mut state = BorrowState::default();
    /// state.acquire(BorrowKind::Mutable, "items").unwrap();
    /// assert!(state.acquire(BorrowKind::Shared, "items").is_err());
    /// ```
    pub fn acquire(&mut self, kind: BorrowKind, binding: &str) -> Result<(), BorrowError> {
        if self.moved {
            return Err(BorrowError::Moved {
                binding: binding.to_string(),
            });
        }
        let blocked = match kind {
            BorrowKind::Mutable => self.is_borrowed(),
            BorrowKind::Shared => self.mutable,
        };
        if blocked {
            return Err(BorrowError::Conflict {
                binding: binding.to_string(),
                requested: kind,
            });
        }
        match kind {
            BorrowKind::Mutable => self.mutable = true,
            BorrowKind::Shared => self.shared += 1,
        }
        Ok(())
    }

    /// Give a borrow back.
    ///
    /// # Arguments
    ///
    /// * `kind` — must match the kind originally acquired.
    /// * `binding` — the identifier, for the error message.
    ///
    /// # Returns
    ///
    /// `Ok(())` with the borrow removed from the state.
    ///
    /// # Errors
    ///
    /// [`BorrowError::UnbalancedRelease`] when no borrow of `kind` is live. The
    /// counters are left untouched rather than clamped, so the accounting bug is
    /// visible instead of turning into an unsound later acquire.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::{BorrowKind, BorrowState};
    ///
    /// let mut state = BorrowState::default();
    /// assert!(state.release(BorrowKind::Shared, "items").is_err());
    /// assert_eq!(state.shared_count(), 0);
    /// ```
    pub fn release(&mut self, kind: BorrowKind, binding: &str) -> Result<(), BorrowError> {
        match kind {
            BorrowKind::Mutable if self.mutable => self.mutable = false,
            BorrowKind::Shared if self.shared > 0 => self.shared -= 1,
            _ => {
                return Err(BorrowError::UnbalancedRelease {
                    binding: binding.to_string(),
                    requested: kind,
                });
            }
        }
        Ok(())
    }
}
