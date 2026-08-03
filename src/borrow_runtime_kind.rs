//! # Borrow kinds
//!
//! tetherscript has exactly two borrow forms in the surface syntax: `&x`
//! (shared) and `&mut x` (mutable). This module names them so the acquire and
//! release rules can be written once and read without guessing what a `bool`
//! meant.

/// Which kind of borrow is being taken.
///
/// # Examples
///
/// ```rust
/// use tetherscript::borrow_runtime::BorrowKind;
///
/// assert!(BorrowKind::Mutable.is_mutable());
/// assert!(!BorrowKind::Shared.is_mutable());
/// assert_eq!(BorrowKind::Shared.label(), "shared");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorrowKind {
    /// `&x` — any number may coexist, but not with a mutable borrow.
    Shared,
    /// `&mut x` — requires that no other borrow of any kind exists.
    Mutable,
}

impl BorrowKind {
    /// Whether this is the exclusive (`&mut`) form.
    ///
    /// # Returns
    ///
    /// `true` for [`BorrowKind::Mutable`], `false` for [`BorrowKind::Shared`].
    pub fn is_mutable(self) -> bool {
        matches!(self, BorrowKind::Mutable)
    }

    /// Human-readable name used in error messages.
    ///
    /// # Returns
    ///
    /// `"shared"` or `"mutable"`.
    pub fn label(self) -> &'static str {
        match self {
            BorrowKind::Shared => "shared",
            BorrowKind::Mutable => "mutable",
        }
    }
}
