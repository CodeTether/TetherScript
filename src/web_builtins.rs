//! Installation of built-ins needed for web-application workloads.
//!
//! These are the primitives the the reference application port needs that the core did not
//! yet provide: HMAC/hex, JWT, cookies, URL-encoded forms, UUIDs, and
//! server-sent events. Each concern owns one submodule with its own `install`,
//! so a group can be added or changed without editing a shared registration
//! list — and without two authors colliding in `interp.rs`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

// Explicit paths: this module is itself declared with `#[path]` from `interp.rs`,
// so relative submodule resolution would otherwise look in `src/` directly.
#[path = "web_builtins/cookie.rs"]
pub(crate) mod cookie;
#[path = "web_builtins/form.rs"]
pub(crate) mod form;
#[path = "web_builtins/hmac.rs"]
pub(crate) mod hmac;
#[path = "web_builtins/jwt.rs"]
pub(crate) mod jwt;
#[path = "web_builtins/sse.rs"]
pub(crate) mod sse;
#[path = "web_builtins/uuid.rs"]
pub(crate) mod uuid;

/// Register every web built-in group.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    hmac::install(env);
    jwt::install(env);
    cookie::install(env);
    form::install(env);
    uuid::install(env);
    sse::install(env);
}
