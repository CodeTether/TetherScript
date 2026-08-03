//! # Borrow errors
//!
//! Every rejection carries the *binding name* the script author wrote, so the
//! backend can render `cannot mutably borrow `items` while it is already
//! borrowed` rather than `borrow error`. AGENTS.md requires every error path to
//! name the thing that went wrong.
//!
//! ## Message parity with the static pass
//!
//! The wording here is copied verbatim from `src/ownership.rs` so a program that
//! is rejected statically and the same program reaching the runtime backstop
//! produce *the same sentence*. Divergent wording for one rule would make users
//! believe there are two different rules.
//!
//! Errors are returned, never panicked, because only the caller knows the
//! source location to attach. Both backends already funnel `Result<_, String>`
//! through their existing error reporting.

use super::kind::BorrowKind;

/// Why a borrow, release, or move was refused.
///
/// # Examples
///
/// ```rust
/// use tetherscript::borrow_runtime::{BorrowError, BorrowKind};
///
/// let err = BorrowError::Conflict {
///     binding: "items".to_string(),
///     requested: BorrowKind::Mutable,
/// };
/// assert_eq!(
///     err.to_string(),
///     "cannot mutably borrow `items` while it is already borrowed"
/// );
///
/// let leak = BorrowError::UnbalancedRelease {
///     binding: "items".to_string(),
///     requested: BorrowKind::Shared,
/// };
/// assert!(leak.to_string().contains("`items`"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorrowError {
    /// The requested borrow conflicts with a live borrow of the same heap value.
    Conflict {
        /// Binding the script named at the borrow site.
        binding: String,
        /// Borrow the script asked for.
        requested: BorrowKind,
    },
    /// The value was moved out, so it has no borrowable owner any more.
    Moved {
        /// Binding the script named at the borrow site.
        binding: String,
    },
    /// A move was attempted while a borrow of the value was still live.
    MoveWhileBorrowed {
        /// Binding the script tried to move.
        binding: String,
    },
    /// A release arrived that no acquire paid for. Reported instead of being
    /// absorbed, because silently clamping to zero hides the real defect.
    UnbalancedRelease {
        /// Binding whose table entry was asked to release.
        binding: String,
        /// Borrow kind the bogus release claimed to hold.
        requested: BorrowKind,
    },
}
