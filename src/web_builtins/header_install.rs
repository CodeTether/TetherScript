//! Registration of the lookup-oriented header built-ins.
//!
//! Split from `header.rs` so the entry point stays within the line budget.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::header_lookup::{as_map, find};
// `result_value` is a private item in `crate::system`, so it cannot be
// re-exported from the group root; each consumer imports it directly.
use super::{header_client_ip, str_arg};
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define the lookup built-ins, then delegate to the negotiation group.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    {
        let mut bindings = env.borrow_mut();
        bindings.define(
            "header_get",
            pure_native("header_get", Some(2), |args| {
                let headers = as_map(&args[0], "header_get: headers")?;
                let name = str_arg(&args[1], "header_get: name")?;
                // Absent is `nil`, not an error: a missing optional header is normal.
                Ok(wrap(Ok(match find(&headers, &name) {
                    Some(value) => Value::Str(Rc::new(value)),
                    None => Value::Nil,
                })))
            }),
            false,
        );
        bindings.define(
            "client_ip",
            pure_native("client_ip", Some(2), |args| {
                let headers = as_map(&args[0], "client_ip: headers")?;
                let remote = str_arg(&args[1], "client_ip: remote_addr")?;
                Ok(header_client_ip::resolve(&headers, &remote))
            }),
            false,
        );
    }
    super::header_negotiate::install(env);
}
