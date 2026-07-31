//! JWT built-ins.
//!
//! Owner: sub-agent `jwt`. Implement `jwt_sign` and `jwt_verify` for HS256 here.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Register this group's built-ins.
pub(crate) fn install(_env: &Rc<RefCell<Env>>) {}
