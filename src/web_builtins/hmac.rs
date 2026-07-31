//! HMAC and hex built-ins.
//!
//! Owner: sub-agent `hmac_hex`. Implement `hmac_sha256_hex`, `hex_encode`, and
//! `hex_decode` here, plus `constant_time_eq` for signature comparison.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Register this group's built-ins.
pub(crate) fn install(_env: &Rc<RefCell<Env>>) {}
