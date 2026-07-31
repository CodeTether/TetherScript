//! JWT built-ins for HS256.
//!
//! Provides the token half of the the reference application auth port: the middleware in
//! `src/middleware/jwt_auth.rs` extracts a `Bearer` token and reads `sub`,
//! `username`, `email`, `exp`, `iat`, `iss`, and `roles` from its claims. Only
//! the legacy HS256 shared-secret path is implemented here; the Keycloak JWKS
//! RS256 path needs asymmetric keys and remains out of scope.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `jwt_sign(claims, secret)` | `Result` of the compact JWS str |
//! | `jwt_verify(token, secret)` | `Result` of the claims map |
//! | `jwt_decode_unverified(token)` | `Result` of the claims map, **unchecked** |
//!
//! # Security
//!
//! Three properties are load-bearing, and each is asserted in `tests/web_jwt.rs`:
//!
//! 1. **The verifier picks the algorithm.** `jwt_verify` requires `alg` to be
//!    exactly `HS256`; `none` and every asymmetric name are rejected. Dispatching
//!    on the header's own `alg` is the classic JWT forgery vector.
//! 2. **Signature comparison is constant-time**, via
//!    [`super::hmac::constant_time_eq`], so a mismatch cannot be located byte by
//!    byte through timing.
//! 3. **`exp` and `nbf` are enforced when present**, and the signature is checked
//!    before either, so unauthenticated claims are never interpreted.
//!
//! `jwt_decode_unverified` deliberately carries the word `unverified`: it skips
//! every check above and its output must not be used for authorization.
//!
//! # Examples
//!
//! ```tether
//! let claims = map()
//! claims.sub = "user-1"
//! claims.exp = time_now_ms() / 1000 + 3600
//!
//! let token = jwt_sign(claims, "shared-secret")?
//! let verified = jwt_verify(token, "shared-secret")?
//! println(verified.sub)
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use super::super::pure_native;
use crate::system::result_value;
use crate::value::Env;

#[path = "jwt_args.rs"]
pub(crate) mod jwt_args;
#[path = "jwt_base64url.rs"]
pub(crate) mod jwt_base64url;
#[path = "jwt_base64url_decode.rs"]
pub(crate) mod jwt_base64url_decode;
#[path = "jwt_claims.rs"]
pub(crate) mod jwt_claims;
#[path = "jwt_header.rs"]
pub(crate) mod jwt_header;
#[path = "jwt_sign.rs"]
pub(crate) mod jwt_sign;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "jwt_sign",
        pure_native("jwt_sign", Some(2), |args| {
            Ok(result_value(jwt_args::sign(args)))
        }),
        false,
    );
    bindings.define(
        "jwt_verify",
        pure_native("jwt_verify", Some(2), |args| {
            Ok(result_value(jwt_args::verify(args)))
        }),
        false,
    );
    bindings.define(
        "jwt_decode_unverified",
        pure_native("jwt_decode_unverified", Some(1), |args| {
            Ok(result_value(jwt_args::decode_unverified(args)))
        }),
        false,
    );
}
