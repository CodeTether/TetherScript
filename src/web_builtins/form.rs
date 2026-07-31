//! URL-encoded form and query-string built-ins.
//!
//! Owner: sub-agent `form`. Implement `form_parse`, `form_encode`,
//! `url_encode`, and `url_decode` here, including `+` as space and percent
//! escapes.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Register this group's built-ins.
pub(crate) fn install(_env: &Rc<RefCell<Env>>) {}
