//! Cookie built-ins.
//!
//! Owner: sub-agent `cookie`. Implement `cookie_parse` and `cookie_serialize`
//! here, including the HttpOnly, Secure, SameSite, Path, and Max-Age attributes.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Register this group's built-ins.
pub(crate) fn install(_env: &Rc<RefCell<Env>>) {}
