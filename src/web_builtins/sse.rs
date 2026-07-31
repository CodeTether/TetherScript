//! Server-sent events built-ins.
//!
//! Owner: sub-agent `sse`. Implement `sse_event` frame formatting here: the
//! `data:`/`event:`/`id:`/`retry:` fields, multi-line data, and the blank-line
//! terminator.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Register this group's built-ins.
pub(crate) fn install(_env: &Rc<RefCell<Env>>) {}
