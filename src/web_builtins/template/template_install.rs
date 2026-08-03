//! Registration of the template built-ins.
//!
//! Split from `template.rs` so the entry point keeps its documentation and the binding
//! list stays readable.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, Value};

/// Build a pure native from a name, arity, and body.
pub(super) fn native<F>(name: &str, arity: usize, func: F) -> Value
where
    F: Fn(&[Value]) -> Result<Value, String> + 'static,
{
    super::super::super::pure_native(name, Some(arity), func)
}

/// Define every template built-in.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    super::template_install_escape::install(env);
    super::template_install_render::install(env);
}
