//! Built-in registration for the session-store group.
//!
//! Split from `store.rs` so the group root carries only documentation and module
//! declarations, matching how `cors_install` and `ratelimit_install` are arranged.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::{store_ops_access as access, store_ops_config as config};
use super::{store_ops_mint as mint, store_ops_revoke as revoke};
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// One built-in's implementation. A plain fn pointer, so the list below needs no
/// per-entry closure and stays one readable line per built-in.
type Op = fn(&[Value]) -> Result<Value, String>;

/// Define one built-in, wrapping its `Result` into a script-visible `Result`.
///
/// # Arguments
///
/// * `bindings` — Environment being populated.
/// * `name` — Script-visible built-in name, also used in its error messages.
/// * `arity` — Exact argument count the interpreter should enforce.
/// * `body` — Implementation.
fn define(bindings: &mut Env, name: &'static str, arity: usize, body: Op) {
    let native = pure_native(name, Some(arity), move |args| Ok(wrap(body(args))));
    bindings.define(name, native, false);
}

/// Define every session-store built-in in `env`.
///
/// # Arguments
///
/// * `env` — The global environment being populated.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut borrowed = env.borrow_mut();
    let bindings = &mut *borrowed;
    define(bindings, "store_create", 2, mint::create);
    define(bindings, "store_rotate", 1, mint::rotate);
    define(bindings, "store_load", 1, access::load);
    define(bindings, "store_save", 2, access::save);
    define(bindings, "store_touch", 1, access::touch);
    define(bindings, "store_destroy", 1, revoke::destroy);
    define(
        bindings,
        "store_destroy_subject",
        1,
        revoke::destroy_subject,
    );
    define(bindings, "store_configure", 2, config::configure);
    define(bindings, "store_sweep", 0, |_args| revoke::sweep());
    define(bindings, "store_count", 0, |_args| config::count());
}
