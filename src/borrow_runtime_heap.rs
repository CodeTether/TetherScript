//! # Heap identity for borrow tracking
//!
//! Borrow state is keyed by *heap identity*, not by binding name. Two bindings
//! that were created by cloning the same `Rc` (a list element, a map value, a
//! closure capture, a function parameter) are the same heap value and must
//! share one borrow state — that is precisely the aliasing the static pass in
//! `src/ownership.rs` cannot see.
//!
//! A [`HeapId`] is the address of the `Rc` allocation. It is stable for as long
//! as the allocation lives, which is exactly as long as any borrow of it can
//! live, because a live borrow implies a live `Rc` clone.
//!
//! ## Why scalars are never given a `HeapId`
//!
//! `Int`, `Float`, `Bool`, and `Nil` are Copy in tetherscript (`Value::is_copy`).
//! A "borrow" of a Copy value is a value copy: no alias exists, so no exclusion
//! rule can be violated. Tracking them would be *pointless* (no aliasing to
//! detect) and *costly* (a hash-map probe on every arithmetic operand, on values
//! that have no allocation to key off in the first place). Callers must simply
//! not call into this module for Copy values.

/// Identity of a heap allocation, used as the borrow-table key.
///
/// # Examples
///
/// ```rust
/// use std::rc::Rc;
/// use tetherscript::borrow_runtime::HeapId;
///
/// let xs = Rc::new(vec![1, 2, 3]);
/// let alias = Rc::clone(&xs);
///
/// // An Rc clone is an alias, so it maps to the same HeapId.
/// assert_eq!(HeapId::from_rc(&xs), HeapId::from_rc(&alias));
///
/// let other = Rc::new(vec![1, 2, 3]);
/// assert_ne!(HeapId::from_rc(&xs), HeapId::from_rc(&other));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HeapId(pub usize);

impl HeapId {
    /// Derive the identity of an `Rc` allocation.
    ///
    /// # Arguments
    ///
    /// * `rc` — the reference-counted payload of a heap `Value`.
    ///
    /// # Returns
    ///
    /// The [`HeapId`] shared by every clone of `rc`.
    pub fn from_rc<T>(rc: &std::rc::Rc<T>) -> Self {
        HeapId(std::rc::Rc::as_ptr(rc) as *const u8 as usize)
    }

    /// Derive the identity from a raw pointer, for payload types the caller
    /// already has a `*const` for.
    ///
    /// # Arguments
    ///
    /// * `ptr` — pointer to the heap payload.
    ///
    /// # Returns
    ///
    /// The [`HeapId`] for that address.
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        HeapId(ptr as *const u8 as usize)
    }
}
