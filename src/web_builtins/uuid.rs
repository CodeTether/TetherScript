//! UUID built-ins.
//!
//! Owner: sub-agent `uuid`. Implement `uuid_v4` and `uuid_parse` here, drawing
//! entropy the way `crate::postgres::nonce` does rather than adding a dependency.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Register this group's built-ins.
pub(crate) fn install(_env: &Rc<RefCell<Env>>) {}
