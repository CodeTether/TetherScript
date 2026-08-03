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
#[path = "web_builtins/base32.rs"]
pub(crate) mod base32;
#[path = "web_builtins/cookie.rs"]
pub(crate) mod cookie;
#[path = "web_builtins/csrf.rs"]
pub(crate) mod csrf;
#[path = "web_builtins/datetime.rs"]
pub(crate) mod datetime;
#[path = "web_builtins/etag.rs"]
pub(crate) mod etag;
#[path = "web_builtins/form.rs"]
pub(crate) mod form;
#[path = "web_builtins/header.rs"]
pub(crate) mod header;
#[path = "web_builtins/hmac.rs"]
pub(crate) mod hmac;
#[path = "web_builtins/jwks.rs"]
pub(crate) mod jwks;
#[path = "web_builtins/jwt.rs"]
pub(crate) mod jwt;
#[path = "web_builtins/log.rs"]
pub(crate) mod log;
#[path = "web_builtins/mime.rs"]
pub(crate) mod mime;
#[path = "web_builtins/multipart.rs"]
pub(crate) mod multipart;
#[path = "web_builtins/password.rs"]
pub(crate) mod password;
#[path = "web_builtins/random.rs"]
pub(crate) mod random;
#[path = "web_builtins/ratelimit.rs"]
pub(crate) mod ratelimit;
#[path = "web_builtins/route.rs"]
pub(crate) mod route;
#[path = "web_builtins/session.rs"]
pub(crate) mod session;
// Middleware and integration groups: the layers Actix `.wrap(..)`s in the reference.
#[path = "web_builtins/abtest.rs"]
pub(crate) mod abtest;
#[path = "web_builtins/cors.rs"]
pub(crate) mod cors;
#[path = "web_builtins/dynpage.rs"]
pub(crate) mod dynpage;
#[path = "web_builtins/identity.rs"]
pub(crate) mod identity;
#[path = "web_builtins/oauth.rs"]
pub(crate) mod oauth;
// Server-side session records and a sliding-window limiter, both Redis-shaped.
#[path = "web_builtins/sessionstore.rs"]
pub(crate) mod sessionstore;
#[path = "web_builtins/sse.rs"]
pub(crate) mod sse;
#[path = "web_builtins/store.rs"]
pub(crate) mod store;
// Streaming SSE: `sse` frames one event, this frames a whole chunked stream.
#[path = "web_builtins/ssestream.rs"]
pub(crate) mod ssestream;
// Pointed at a directory rather than a bare file: the template engine has 23
// submodules, and a `#[path]`-included file resolves its own submodules against
// `web_builtins/`, so the declarations would each need a redundant `#[path]` too.
#[path = "web_builtins/template/mod.rs"]
pub(crate) mod template;
#[path = "web_builtins/uuid.rs"]
pub(crate) mod uuid;
#[path = "web_builtins/validate.rs"]
pub(crate) mod validate;

/// Register every web built-in group.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    hmac::install(env);
    jwt::install(env);
    jwks::install(env);
    cookie::install(env);
    form::install(env);
    uuid::install(env);
    sse::install(env);
    ssestream::install(env);
    abtest::install(env);
    cors::install(env);
    dynpage::install(env);
    identity::install(env);
    oauth::install(env);
    store::install(env);
    sessionstore::install(env);
    base32::install(env);
    csrf::install(env);
    datetime::install(env);
    etag::install(env);
    mime::install(env);
    password::install(env);
    random::install(env);
    route::install(env);
    header::install(env);
    log::install(env);
    multipart::install(env);
    ratelimit::install(env);
    session::install(env);
    // Called directly rather than through a wrapper in the group's mod.rs, which keeps
    // that file's declaration list within the line budget.
    template::template_install::install(env);
    validate::install(env);
}
