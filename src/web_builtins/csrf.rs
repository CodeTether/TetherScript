//! Signed, expiring CSRF and OAuth-state token built-ins.
//!
//! A QuickBooks OAuth flow needs a one-time, short-TTL state value:
//! `controllers/quickbooks/connect.rs` mints one with a 10-minute TTL, and
//! `callback.rs` rejects the callback when it is invalid or expired. That
//! implementation stores every state row in Postgres. These tokens are instead
//! **stateless** — the TTL and a random nonce are carried in the token and
//! authenticated by an HMAC — so a port needs no table and no cleanup job.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `csrf_token(secret, ttl_seconds)` | `Result` of the token str |
//! | `csrf_verify(token, secret)` | `Result` of bool: false when expired |
//! | `csrf_claims(token)` | `Result` map, **unverified** |
//!
//! # Design notes
//!
//! * A fresh 16-byte random nonce per token means two tokens minted in the same
//!   second still differ, so one cannot be guessed from another.
//! * Signature comparison is constant-time, via the HMAC group's
//!   [`constant_time_eq`](super::hmac::constant_time_eq).
//! * `csrf_verify` returns `Ok(false)` for a validly signed but expired token and
//!   an `Err` only for tampering or malformed input, so a caller can tell "start
//!   over" apart from "someone is attacking this".
//! * Being stateless, these tokens are **not single-use**: a valid token replays
//!   until it expires. The reference flow deletes the row on consumption, which is
//!   strictly stronger. Callers needing one-time semantics must still record spent
//!   nonces; that is why `csrf_claims` exposes the nonce.
//!
//! # Examples
//!
//! ```tether
//! let state = csrf_token(secret, 600)?
//! // ... redirect the user, then on callback:
//! if csrf_verify(state, secret)? {
//!     println("state accepted")
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::Env;

use super::super::pure_native;

#[path = "csrf_args.rs"]
mod csrf_args;
#[path = "csrf_base64url.rs"]
mod csrf_base64url;
#[path = "csrf_base64url_decode.rs"]
mod csrf_base64url_decode;
#[path = "csrf_claims.rs"]
mod csrf_claims;
#[path = "csrf_parse.rs"]
mod csrf_parse;
#[path = "csrf_payload.rs"]
mod csrf_payload;
#[path = "csrf_sign.rs"]
mod csrf_sign;

/// Register this group's built-ins.
///
/// Defines `csrf_token`, `csrf_verify`, and `csrf_claims` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "csrf_token",
        pure_native("csrf_token", Some(2), |args| {
            Ok(result_value(csrf_args::token(args)))
        }),
        false,
    );
    bindings.define(
        "csrf_verify",
        pure_native("csrf_verify", Some(2), |args| {
            Ok(result_value(csrf_args::verify(args)))
        }),
        false,
    );
    bindings.define(
        "csrf_claims",
        pure_native("csrf_claims", Some(1), |args| {
            Ok(result_value(csrf_args::claims(args)))
        }),
        false,
    );
}
