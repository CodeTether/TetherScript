//! Registration of the auth and content-negotiation header built-ins.
//!
//! Kept apart from the lookup bindings so neither file exceeds the line budget,
//! and so the credential-bearing built-in sits beside the negotiation logic that
//! decides what a handler is allowed to return.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::header_lookup::as_map;
use super::{header_accept, header_auth, header_security, str_arg};
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define `bearer_token`, `accepts`, and `security_headers` in `env`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "bearer_token",
        pure_native("bearer_token", Some(1), |args| {
            let headers = as_map(&args[0], "bearer_token: headers")?;
            Ok(wrap(header_auth::bearer(&headers)))
        }),
        false,
    );
    bindings.define(
        "accepts",
        pure_native("accepts", Some(2), |args| {
            let headers = as_map(&args[0], "accepts: headers")?;
            let wanted = str_arg(&args[1], "accepts: content_type")?;
            Ok(Value::Bool(header_accept::accepts(&headers, &wanted)))
        }),
        false,
    );
    bindings.define(
        "security_headers",
        pure_native("security_headers", Some(0), |_args| {
            Ok(header_security::recommended())
        }),
        false,
    );
}
