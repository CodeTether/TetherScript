//! # Runtime `&mut` exclusivity enforcement
//!
//! The runtime backstop for tetherscript's aliasing-xor-mutability guarantee.
//! `src/ownership.rs` is the **static fast path**: it rejects the lexically
//! obvious violations during `check`, before anything executes. This module is
//! the layer underneath it, for the aliases the static pass cannot see —
//! aliases created through a list element, a map value, a closure capture, or a
//! function boundary.
//!
//! ## The rule
//!
//! * A **mutable** borrow (`&mut x`) requires **zero** other borrows of any kind.
//! * A **shared** borrow (`&x`) requires **zero** mutable borrows.
//! * A **moved** value cannot be borrowed; a **borrowed** value cannot be moved.
//!
//! ## Design in one paragraph
//!
//! Borrow state is keyed by *heap identity* ([`HeapId`], the `Rc` address), not
//! by binding name, which is exactly why it sees dynamic aliases: every `Value`
//! clone is an `Rc` clone and therefore maps to the same key. State lives in a
//! side [`BorrowTable`] so `src/value.rs` needs no new field. Backends hold a
//! cheap [`BorrowTracker`] handle. Borrowing yields a [`BorrowGuard`] whose
//! `Drop` releases, so an early `return` or a propagated `?` cannot leak a count.
//! Violations are returned as a [`BorrowError`] naming the binding, never
//! panicked, because only the caller knows the source location.
//!
//! ## Scalars are not tracked
//!
//! `Int`, `Float`, `Bool`, and `Nil` are Copy (`Value::is_copy`). Borrowing one
//! copies it, so no alias exists and no exclusion rule can be violated. Tracking
//! them would be pointless *and* a per-operand hash-probe cost.
//! [`heap_id_of`] returns `None` for them.
//!
//! ## Reentrancy
//!
//! See [`reentrancy`] for the case-by-case account of what is caught (iteration
//! while mutating, closure capture versus enclosing mutation, `&mut` into a
//! callee while the caller holds `&`) and what is not.
//!
//! ## Quick start
//!
//! ```rust
//! use std::{cell::RefCell, rc::Rc};
//! use tetherscript::borrow_runtime::{BorrowKind, BorrowTracker};
//! use tetherscript::value::Value;
//!
//! let tracker = BorrowTracker::new();
//! let items = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
//!
//! // `alias` is a dynamically created alias: same allocation, different name.
//! let alias = items.clone();
//!
//! let held = tracker
//!     .borrow_named(&items, BorrowKind::Shared, "items")
//!     .unwrap();
//! let err = tracker
//!     .borrow_named(&alias, BorrowKind::Mutable, "alias")
//!     .unwrap_err();
//! assert_eq!(
//!     err.to_string(),
//!     "cannot mutably borrow `alias` while it is already borrowed"
//! );
//!
//! drop(held);
//! assert!(tracker
//!     .borrow_named(&alias, BorrowKind::Mutable, "alias")
//!     .is_ok());
//! ```

#[path = "borrow_runtime_acquire.rs"]
mod acquire;
#[path = "borrow_runtime_borrow.rs"]
mod borrow;
#[path = "borrow_runtime_display.rs"]
mod display;
#[path = "borrow_runtime_error.rs"]
pub mod error;
#[path = "borrow_runtime_guard.rs"]
pub mod guard;
#[path = "borrow_runtime_heap.rs"]
pub mod heap;
#[path = "borrow_runtime_kind.rs"]
pub mod kind;
#[path = "borrow_runtime_move.rs"]
mod move_rules;
#[path = "borrow_runtime_named.rs"]
mod named;
#[path = "borrow_runtime_reentrancy.rs"]
pub mod reentrancy;
#[path = "borrow_runtime_state.rs"]
pub mod state;
#[path = "borrow_runtime_table.rs"]
pub mod table;
#[path = "borrow_runtime_table_ops.rs"]
mod table_ops;
#[path = "borrow_runtime_tracker.rs"]
pub mod tracker;
#[path = "borrow_runtime_value.rs"]
pub mod value;

pub use error::BorrowError;
pub use guard::BorrowGuard;
pub use heap::HeapId;
pub use kind::BorrowKind;
pub use state::BorrowState;
pub use table::BorrowTable;
pub use tracker::BorrowTracker;
pub use value::heap_id_of;
