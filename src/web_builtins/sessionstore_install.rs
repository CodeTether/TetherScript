//! Built-in registration for the Redis-backed session-store logic group.
//!
//! Split from `sessionstore.rs` so the group root carries only documentation and
//! module declarations, matching how `ratelimit_install` and `store_install` are
//! arranged.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::sessionstore_ops_limit as limit;
use super::sessionstore_ops_session as session;
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// One built-in's implementation. A plain fn pointer, so the list below needs no
/// per-entry closure and stays one readable line per built-in.
type Op = fn(&[Value]) -> Result<Value, String>;

/// Define one `Result`-returning built-in.
///
/// # Arguments
///
/// * `bindings` — Environment being populated.
/// * `name` — Script-visible name, also used in its error messages.
/// * `arity` — Exact argument count the interpreter enforces.
/// * `body` — Implementation.
fn define(bindings: &mut Env, name: &'static str, arity: usize, body: Op) {
    let native = pure_native(name, Some(arity), move |args| Ok(wrap(body(args))));
    bindings.define(name, native, false);
}

/// Define every session-store built-in in `env`.
///
/// # Arguments
///
/// * `env` — The global interpreter environment being populated.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut borrowed = env.borrow_mut();
    let bindings = &mut *borrowed;
    define(bindings, "session_store_key", 2, session::key);
    define(bindings, "session_store_encode", 1, session::encode);
    define(bindings, "session_store_decode", 1, session::decode);
    define(bindings, "session_rotate_id", 1, session::rotate);
    define(bindings, "ratelimit_window_key", 4, limit::window_key);
    define(bindings, "ratelimit_window_verdict", 4, limit::verdict);
    // Returns the id directly rather than a `Result`: it takes no argument and has
    // no failure mode, so wrapping it would force every caller to write `?`.
    let fresh = pure_native("session_store_new_id", Some(0), |_args| {
        Ok(session::new_id())
    });
    bindings.define("session_store_new_id", fresh, false);
}
