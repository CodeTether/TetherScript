//! # Borrow error rendering
//!
//! Split from the error enum itself so the data definition and its user-facing
//! prose are each one concern in one file. The strings must stay
//! character-identical to the corresponding diagnostics in `src/ownership.rs`;
//! `tests/borrow_runtime.rs` asserts the exact sentences so a future edit cannot
//! silently split one rule into two different messages.

use std::fmt;

use super::error::BorrowError;
use super::kind::BorrowKind;

impl BorrowError {
    /// The binding name this error is about.
    ///
    /// # Returns
    ///
    /// The script-visible identifier, for callers that want to attach a source
    /// span to it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::borrow_runtime::BorrowError;
    ///
    /// let err = BorrowError::Moved { binding: "xs".into() };
    /// assert_eq!(err.binding(), "xs");
    /// ```
    pub fn binding(&self) -> &str {
        match self {
            BorrowError::Conflict { binding, .. }
            | BorrowError::Moved { binding }
            | BorrowError::MoveWhileBorrowed { binding }
            | BorrowError::UnbalancedRelease { binding, .. } => binding,
        }
    }
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::Conflict {
                binding,
                requested: BorrowKind::Mutable,
            } => write!(
                f,
                "cannot mutably borrow `{binding}` while it is already borrowed"
            ),
            BorrowError::Conflict {
                binding,
                requested: BorrowKind::Shared,
            } => write!(f, "cannot borrow `{binding}` while it is mutably borrowed"),
            BorrowError::Moved { binding } => {
                write!(f, "cannot borrow moved value `{binding}`")
            }
            BorrowError::MoveWhileBorrowed { binding } => {
                write!(f, "cannot move `{binding}` while it is borrowed")
            }
            BorrowError::UnbalancedRelease { binding, requested } => {
                let kind = requested.label();
                write!(
                    f,
                    "internal borrow accounting error: released a {kind} borrow \
                     of `{binding}` that was never acquired"
                )
            }
        }
    }
}

impl std::error::Error for BorrowError {}
