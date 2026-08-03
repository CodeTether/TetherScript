//! # Mapping a `Value` to a borrow-table key
//!
//! Bridges `src/value.rs` to this module *without modifying it*: the mapping is
//! read-only pattern matching over `Value`.
//!
//! Scalars return `None`. They are Copy in tetherscript (`Value::is_copy`), so a
//! "borrow" of one is a copy and no alias exists to exclude. Giving them a key
//! would be pointless — there is no aliasing to detect — and a real cost: a
//! hash probe on every arithmetic operand, for values that have no allocation to
//! key off. `None` is therefore the correct answer, not a gap.
//!
//! Function, native, capability, and result payloads are heap-allocated and so do
//! get an id; whether the backends choose to borrow-check calls through them is
//! their decision, and `heap_id_of` stays honest either way.

use crate::value::Value;

use super::heap::HeapId;

/// Borrow-table key for a value, if it has one.
///
/// # Arguments
///
/// * `value` — any runtime value.
///
/// # Returns
///
/// `Some(HeapId)` for heap-backed values, `None` for Copy scalars (`Int`,
/// `Float`, `Bool`, `Nil`) which need no borrow tracking.
///
/// # Examples
///
/// ```rust
/// use std::{cell::RefCell, rc::Rc};
/// use tetherscript::borrow_runtime::heap_id_of;
/// use tetherscript::value::Value;
///
/// assert_eq!(heap_id_of(&Value::Int(1)), None);
/// assert_eq!(heap_id_of(&Value::Nil), None);
///
/// let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
/// let alias = list.clone();
/// assert!(heap_id_of(&list).is_some());
/// // Cloning a Value is an Rc clone: the alias shares one borrow state.
/// assert_eq!(heap_id_of(&list), heap_id_of(&alias));
/// ```
pub fn heap_id_of(value: &Value) -> Option<HeapId> {
    match value {
        Value::Nil | Value::Int(_) | Value::Float(_) | Value::Bool(_) => None,
        Value::Str(rc) => Some(HeapId::from_rc(rc)),
        Value::Bytes(rc) => Some(HeapId::from_rc(rc)),
        Value::List(rc) => Some(HeapId::from_rc(rc)),
        Value::Map(rc) => Some(HeapId::from_rc(rc)),
        Value::Fn(rc) => Some(HeapId::from_rc(rc)),
        Value::VmFn(rc) => Some(HeapId::from_rc(rc)),
        Value::Native(rc) => Some(HeapId::from_rc(rc)),
        Value::Result(rc) => Some(HeapId::from_rc(rc)),
        Value::Capability(rc) => Some(HeapId::from_rc(rc)),
        Value::Resource(rc) => Some(HeapId::from_rc(rc)),
    }
}
