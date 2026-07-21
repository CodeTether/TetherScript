//! Installation of HTTP and HTTPS listener built-ins.

use std::cell::RefCell;
use std::rc::Rc;

use crate::http;
use crate::value::Env;

use super::runtime_native;

pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let static_globals = env.clone();
    let mut bindings = env.borrow_mut();
    bindings.define(
        "http_serve",
        runtime_native("http_serve", Some(2), |runtime, args| {
            http::serve(runtime, &args[0], &args[1])
        }),
        false,
    );
    bindings.define(
        "https_serve",
        runtime_native("https_serve", Some(4), |runtime, args| {
            http::serve_tls(runtime, &args[0], &args[1], &args[2], &args[3])
        }),
        false,
    );
    bindings.define(
        "http_serve_static",
        runtime_native("http_serve_static", Some(2), move |runtime, args| {
            http::serve_static(runtime, &static_globals, &args[0], &args[1])
        }),
        false,
    );
}
