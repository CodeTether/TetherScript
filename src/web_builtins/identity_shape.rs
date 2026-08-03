//! The identity map, and the single point at which `authenticated` is decided.
//!
//! # Anonymous by default, structurally
//!
//! [`identity`] is the **only** constructor of an identity map anywhere in this
//! group, and it takes no `authenticated` parameter. The flag is *derived*: it is
//! `true` exactly when a non-empty subject was supplied, and there is no argument
//! a caller could pass to say otherwise.
//!
//! That is the structural part, and it is what makes a missing check fail closed:
//!
//! * There is no `authenticated: bool` parameter to get the wrong way round.
//! * There is no path that copies an `authenticated` field out of the claims, so a
//!   caller who mints `{"authenticated": true}` as a claim gains nothing. Only a
//!   subject the verifier put there can produce `true`.
//! * [`anonymous`] is the same function called with `None`, not a second
//!   hand-written map, so the two shapes cannot drift apart and leave the anonymous
//!   map missing a field a handler reads.
//! * Every failure path in [`super::identity_claims`] returns either an error or
//!   [`anonymous`]. None of them can return a partially-populated identity, because
//!   there is no way to build one.
//!
//! The alternative — a builder with a settable flag, or a struct literal in each
//! call site — makes `authenticated: true` reachable by omission. Here it is
//! reachable only by presenting a subject.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Assemble an identity map.
///
/// # Arguments
///
/// * `subject` — The verified subject, or `None` for an unauthenticated caller. A
///   `Some("")` is treated as `None`, because a claim set with an empty `sub` names
///   nobody.
/// * `roles` — Role strings held by the subject. Ignored for an anonymous identity:
///   roles without a subject would be authority attached to nobody.
///
/// # Returns
///
/// A map with `subject` (str or `nil`), `roles` (list of str), and `authenticated`
/// (bool, derived from `subject`).
pub(super) fn identity(subject: Option<&str>, roles: Vec<String>) -> Value {
    let named = subject.filter(|text| !text.is_empty());
    let mut map = HashMap::new();
    let entries: Vec<Value> = match named {
        Some(_) => roles.into_iter().map(|r| Value::Str(Rc::new(r))).collect(),
        None => Vec::new(),
    };
    map.insert(
        "subject".into(),
        match named {
            Some(text) => Value::Str(Rc::new(text.to_string())),
            None => Value::Nil,
        },
    );
    map.insert("roles".into(), Value::List(Rc::new(RefCell::new(entries))));
    // Derived, never supplied. This line is the whole invariant.
    map.insert("authenticated".into(), Value::Bool(named.is_some()));
    Value::Map(Rc::new(RefCell::new(map)))
}

/// The identity of an unauthenticated caller.
///
/// # Returns
///
/// An identity map with `subject: nil`, `roles: []`, `authenticated: false`.
pub(super) fn anonymous() -> Value {
    identity(None, Vec::new())
}
