//! Signed session-cookie payload built-ins.
//!
//! This ports the cookie half of the reference application's Actix session middleware. The
//! reference keeps session state in Redis and uses a **signed**, not encrypted,
//! cookie so a Node service can read the same session id. There is no Redis
//! client in this group, so these built-ins only mint and verify tamper-proof
//! payloads that a caller hands to `cookie_serialize`.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `session_sign(payload, secret)` | `Result` of the signed cookie value |
//! | `session_verify(value, secret)` | `Result` of the payload map |
//! | `session_touch(payload, ttl_seconds)` | `Result` of the payload with a fresh `exp` |
//! | `session_expired(payload)` | `Result` of bool |
//!
//! # Security
//!
//! **Signed is not encrypted.** Anyone holding the cookie can read the payload, so
//! never place a password, API key, OAuth token, or any other secret in it. Store
//! identifiers and non-sensitive metadata only; the signature prevents tampering,
//! not disclosure. This mirrors the reference, which keeps the sensitive session
//! data in Redis and puts only the session id in the cookie.
//!
//! # Examples
//!
//! ```tether
//! let payload = map()
//! payload.sid = "session-id"
//! payload.user_id = "user-1"
//!
//! // 604800 seconds is the reference 7-day TTL, extended on every request.
//! let refreshed = session_touch(payload, 604800)?
//! let value = session_sign(refreshed, "cookie-secret")?
//!
//! let opts = map()
//! opts.path = "/"
//! opts.http_only = true
//! opts.same_site = "Lax"
//! println(cookie_serialize("id", value, opts)?)
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::Env;

use super::super::pure_native;

#[path = "session_args.rs"]
pub(super) mod session_args;
#[path = "session_base64url.rs"]
pub(super) mod session_base64url;
#[path = "session_payload.rs"]
pub(super) mod session_payload;
#[path = "session_sign.rs"]
pub(super) mod session_sign;
#[path = "session_ttl.rs"]
pub(super) mod session_ttl;

/// Register the session built-ins.
///
/// # Arguments
///
/// * `env` — Global interpreter environment being populated.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "session_sign",
        pure_native("session_sign", Some(2), |args| {
            Ok(result_value(session_args::sign(args)))
        }),
        false,
    );
    bindings.define(
        "session_verify",
        pure_native("session_verify", Some(2), |args| {
            Ok(result_value(session_args::verify(args)))
        }),
        false,
    );
    bindings.define(
        "session_touch",
        pure_native("session_touch", Some(2), |args| {
            Ok(result_value(session_args::touch(args)))
        }),
        false,
    );
    bindings.define(
        "session_expired",
        pure_native("session_expired", Some(1), |args| {
            Ok(result_value(session_ttl::expired(&args[0])))
        }),
        false,
    );
}
