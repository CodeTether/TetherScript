//! # Reentrancy analysis
//!
//! This module is documentation only. It records, case by case, what the runtime
//! borrow layer catches and what it does not, so nobody has to infer the
//! guarantee from the code.
//!
//! Throughout: the state is keyed by **heap identity**, so any two names that
//! reached the same `Rc` allocation share one [`super::state::BorrowState`]. That
//! is the whole reason the runtime layer sees things
//! [`crate::ownership`] cannot: the static pass keys on *binding names* in
//! lexical scopes.
//!
//! ## Case 1 — a method that borrows a list while iterating it
//!
//! ```text
//! let mut xs = [1, 2, 3]
//! for x in &xs {        // shared borrow of xs held for the loop
//!     xs.push(x)        // wants a mutable borrow of the same heap value
//! }
//! ```
//!
//! **Caught.** The loop holds a [`super::guard::BorrowGuard`] for
//! [`super::kind::BorrowKind::Shared`] across the body, so `push`'s mutable
//! acquire sees `shared == 1` and fails with `cannot mutably borrow `xs` while it
//! is already borrowed`. This requires the integrator to hold the iteration guard
//! for the loop's duration and to acquire a mutable borrow for mutating methods;
//! the diff in the delivery report does exactly that.
//!
//! ## Case 2 — a closure capturing a value its enclosing scope also mutates
//!
//! ```text
//! let mut cell = [0]
//! let bump = fn() { cell.push(1) }   // capture aliases the same Rc
//! let view = &cell                   // shared borrow in the outer scope
//! bump()                             // mutable borrow inside the closure
//! ```
//!
//! **Caught.** The capture is an `Rc` clone, so `heap_id_of(&captured) ==
//! heap_id_of(&cell)`. The outer shared guard is live when the closure body
//! acquires mutably, so the mutable acquire is refused. The static pass cannot
//! see this: the closure body refers to `cell` in a different scope and the alias
//! is created at capture time, not at a `&`/`&mut` site.
//!
//! ## Case 3 — a function receiving `&mut x` while the caller holds `&x`
//!
//! ```text
//! let mut items = [1]
//! let view = &items          // caller's shared borrow, still live
//! mutate(&mut items)         // callee wants exclusivity
//! ```
//!
//! **Caught** — and this is the one case the static pass also catches when both
//! borrows are written as simple identifiers in the same scope. The runtime layer
//! agrees with it word for word (see `borrow_runtime_display.rs`), and extends it
//! to the variants the static pass misses: when the `&x` was stored in a list
//! element, a map value, or was passed through a second function, the static pass
//! sees no `Expr::Borrow(Expr::Ident)` to record and accepts the program. The
//! runtime layer still refuses it.
//!
//! ## Case 4 — reentrancy through a native callback
//!
//! A `NativeFunc::Runtime` built-in (see `crate::value::NativeFunc`) can call
//! back into user code while the native itself holds a borrow. **Caught** if the
//! native takes its borrow through a guard, because the reentered script's
//! mutable acquire then conflicts. **Not caught** if a native mutates a payload
//! through `RefCell::borrow_mut` directly without going through the tracker —
//! that is an unchecked hole, and it is a hole in the *native*, not in these
//! rules. It is listed here rather than hidden.
//!
//! ## Case 5 — recursion re-borrowing the same value shared twice
//!
//! ```text
//! fn walk(node) { walk(node) }   // shared borrow per frame
//! ```
//!
//! **Correctly allowed.** Shared borrows compose, so nesting is fine. Rejecting
//! it would break existing programs; that is the direction of divergence that
//! matters, and the rules do not take it.
//!
//! ## Where the runtime layer and the static pass could disagree
//!
//! 1. **Rebinding.** `crate::ownership`'s `assign_binding` clears `moved` when a
//!    binding is reassigned. The runtime table keys on the *old* heap identity,
//!    which the new value does not share, so the fresh value starts clean.
//!    Agreement, provided the integrator does **not** reuse a `HeapId` across a
//!    rebind — call [`super::tracker::BorrowTracker::forget`] or
//!    [`super::state::BorrowState::reset`] on the old id.
//! 2. **Interned / identical heap values.** Two independently written equal
//!    strings normally have distinct allocations, so distinct ids. If a future
//!    constant-pool change made `Value::Str` payloads shared between unrelated
//!    literals, they would share a borrow state and the runtime layer could
//!    reject a program the static pass accepts. This is a real divergence risk
//!    and must be revisited if string interning lands.
//! 3. **Scope-end release.** The static pass releases borrows at lexical scope
//!    exit (`pop_scope`). The runtime releases at guard drop. Those coincide when
//!    the integrator scopes guards to the same block. If a guard is deliberately
//!    held longer — e.g. stored in a data structure — the runtime is *stricter*
//!    than the static pass. The delivery diff therefore keeps every guard in a
//!    block-scoped local.
//! 4. **Copy scalars.** The static pass tracks a `copy` flag per binding and
//!    exempts scalars from move rules. The runtime layer exempts them by
//!    returning `None` from [`super::value::heap_id_of`]. Same outcome, reached
//!    two ways; both must keep `Value::is_copy` as the single source of truth.
//!
//! No rule here rejects anything the static pass accepts for a legitimate
//! reason, with the two caveats above (interning, over-long guards) called out
//! explicitly rather than assumed away.
//!
//! ## The static pass stays the fast path
//!
//! Nothing in this module runs during `check`. `crate::ownership` still rejects
//! the lexically obvious violations before a single instruction executes; the
//! table is only consulted at `&`/`&mut`/`move` sites on heap values that
//! actually reach the runtime.
