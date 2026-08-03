//! # Per-value borrow state (representation and queries)
//!
//! One [`BorrowState`] per heap value. The invariant the whole feature rests on
//! is small enough to state in one line:
//!
//! ```text
//! !(mutable && shared > 0)   &&   !(mutable && moved)   &&   !(moved && shared > 0)
//! ```
//!
//! i.e. a mutable borrow excludes every other borrow, and a moved value has no
//! borrows at all. [`BorrowState::invariant_holds`] checks it, and every mutating
//! method (see `borrow_runtime_acquire.rs` and `borrow_runtime_move.rs`) is
//! written so that it is true on entry and on exit — which is what makes the
//! state checkable *at any moment* rather than only at scope ends.
//!
//! ## Why the mutable borrow is a `bool`, not a counter
//!
//! `&mut` is exclusive by definition, so the count can only ever be 0 or 1. A
//! counter would admit representing `2`, a state that is illegal by design, and
//! then every reader would have to wonder what it means. The `bool` makes the
//! illegal state unrepresentable.
//!
//! ## Why an unbalanced release is an error, not a clamp
//!
//! A release that no acquire paid for means the caller's bookkeeping is broken.
//! Clamping to zero would let the *next* mutable acquire succeed when a real
//! shared borrow was still live (unsound), while a *leaked* count is worse
//! still: it wedges the value as permanently borrowed and makes later, entirely
//! correct code fail with a conflict that has no live borrow behind it. That
//! failure is non-local — the reported binding is innocent and the real culprit
//! is a borrow site that already returned. So releases are reported, and
//! [`super::guard::BorrowGuard`] exists so correct code never needs to hand-pair
//! one.

/// Live borrow bookkeeping for a single heap value.
///
/// # Examples
///
/// ```rust
/// use tetherscript::borrow_runtime::{BorrowKind, BorrowState};
///
/// let mut state = BorrowState::default();
/// state.acquire(BorrowKind::Shared, "items").unwrap();
/// state.acquire(BorrowKind::Shared, "items").unwrap();
/// assert!(state.acquire(BorrowKind::Mutable, "items").is_err());
///
/// state.release(BorrowKind::Shared, "items").unwrap();
/// state.release(BorrowKind::Shared, "items").unwrap();
/// assert!(state.acquire(BorrowKind::Mutable, "items").is_ok());
/// assert!(state.invariant_holds());
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BorrowState {
    pub(super) shared: usize,
    pub(super) mutable: bool,
    pub(super) moved: bool,
}

impl BorrowState {
    /// Number of live `&` borrows.
    ///
    /// # Returns
    ///
    /// The shared borrow count, `0` when unborrowed.
    pub fn shared_count(&self) -> usize {
        self.shared
    }

    /// Whether a live `&mut` borrow exists.
    ///
    /// # Returns
    ///
    /// `true` while exactly one mutable borrow is outstanding.
    pub fn is_mutably_borrowed(&self) -> bool {
        self.mutable
    }

    /// Whether the value has been moved out of its binding.
    ///
    /// # Returns
    ///
    /// `true` once [`BorrowState::mark_moved`] has succeeded.
    pub fn is_moved(&self) -> bool {
        self.moved
    }

    /// Whether any borrow at all is live.
    ///
    /// # Returns
    ///
    /// `true` if a shared or mutable borrow is outstanding.
    pub fn is_borrowed(&self) -> bool {
        self.mutable || self.shared > 0
    }

    /// Re-check the XOR-mutability invariant.
    ///
    /// # Returns
    ///
    /// `true` when the state is internally consistent. It should never be
    /// `false`; the tests assert it after every operation so a future edit that
    /// breaks the accounting is caught immediately rather than silently.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::BorrowState;
    ///
    /// assert!(BorrowState::default().invariant_holds());
    /// ```
    pub fn invariant_holds(&self) -> bool {
        !(self.mutable && self.shared > 0)
            && !(self.moved && self.mutable)
            && !(self.moved && self.shared > 0)
    }

    /// Reset to the unborrowed, unmoved state, for when a binding is rebound to
    /// a fresh value and its old accounting no longer applies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::BorrowState;
    ///
    /// let mut state = BorrowState::default();
    /// state.mark_moved("xs").unwrap();
    /// state.reset();
    /// assert!(!state.is_moved());
    /// ```
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
