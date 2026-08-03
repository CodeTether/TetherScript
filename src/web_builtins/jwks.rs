//! JWK / JWKS key-material built-ins and unverified JWS header access.
//!
//! The reference application authenticates against Keycloak, which publishes an
//! RS256 JWKS document at `/protocol/openid-connect/certs`. The in-tree JWT
//! group (`src/web_builtins/jwt.rs`) only implements the symmetric HS256 path,
//! so a script had no way to (a) read a token's `kid` in order to choose a key,
//! (b) turn a JWKS document into usable key material, or (c) obtain the exact
//! bytes an RSA verifier must operate on. This group supplies precisely those
//! three things and deliberately stops there: **no signature verification and
//! no RSA arithmetic happen anywhere in this module.**
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `jwks_parse(json)` | `Result` of a list of validated key maps |
//! | `jwks_find(keys, kid)` | `Result` of the one key with that `kid` |
//! | `jwt_header(token)` | `Result` of the **unverified** header map |
//! | `jwt_rs256_parts(token)` | `Result` of signing input, signature, and `alg` |
//!
//! Each key map exposes `kid`, `kty`, `alg`, `use`, `modulus`, `exponent`,
//! `modulus_bits`, and the JWK-spelled aliases `n` and `e` (identical bytes to
//! `modulus` and `exponent`). See `jwks_key::normalize`.
//!
//! # Security
//!
//! Five properties are load-bearing, and each is asserted in `tests/web_jwks.rs`:
//!
//! 1. **`jwt_header` is unverified, and says so in its name.** Its output is
//!    attacker-controlled: anyone can author a header. It exists *only* so a
//!    caller can read `kid` and select a candidate key. It must never inform an
//!    authorization decision.
//! 2. **The verifier picks the algorithm, never the token.** `jwks_alg` rejects
//!    `none` and anything outside RS256/RS384/RS512, and this module never maps
//!    a token's `alg` onto a verification routine. Dispatching on the header's
//!    own `alg` is the classic JWT forgery: an attacker downgrades an RS256
//!    deployment to `none`, or re-signs with HMAC using the public key as the
//!    shared secret, and a trusting verifier obeys.
//! 3. **Weak or wrong-typed keys are refused at parse time**, not at use time:
//!    non-RSA `kty`, a modulus under 2048 bits, or an empty exponent
//!    (`jwks_rsa`).
//! 4. **base64url is strict.** `+`, `/`, `=`, and every other non-alphabet byte
//!    are rejected rather than tolerated (`jwks_base64url`).
//! 5. **A two-segment token is refused.** That is the unsecured JWS form
//!    (`jwks_token::split`).
//!
//! # Examples
//!
//! ```tether
//! let keys = jwks_parse(certs_json)?
//! let header = jwt_header(token)?          // UNVERIFIED: key selection only
//! let key = jwks_find(keys, header.kid)?
//! let parts = jwt_rs256_parts(token)?
//! // `parts.signing_input`, `parts.signature`, `key.modulus`, `key.exponent`
//! // are then handed to an RSA verifier, which is not part of this module.
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "jwks_alg.rs"]
pub(crate) mod jwks_alg;
#[path = "jwks_args.rs"]
pub(crate) mod jwks_args;
#[path = "jwks_base64url.rs"]
pub(crate) mod jwks_base64url;
#[path = "jwks_base64url_pack.rs"]
pub(crate) mod jwks_base64url_pack;
#[path = "jwks_document.rs"]
pub(crate) mod jwks_document;
#[path = "jwks_field.rs"]
pub(crate) mod jwks_field;
#[path = "jwks_find.rs"]
pub(crate) mod jwks_find;
#[path = "jwks_install.rs"]
pub(crate) mod jwks_install;
#[path = "jwks_key.rs"]
pub(crate) mod jwks_key;
#[path = "jwks_parts.rs"]
pub(crate) mod jwks_parts;
#[path = "jwks_rsa.rs"]
pub(crate) mod jwks_rsa;
#[path = "jwks_token.rs"]
pub(crate) mod jwks_token;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Environment the four built-ins are defined into.
///
/// # Returns
///
/// Nothing; `env` gains `jwks_parse`, `jwks_find`, `jwt_header`, and
/// `jwt_rs256_parts`.
///
/// # Errors
///
/// Cannot fail. Every fallible step happens inside a built-in and surfaces to
/// the script as a `Result`.
///
/// # Examples
///
/// ```tether
/// // After installation the whole group is callable:
/// println(str(jwt_header("only.two").is_err()))   // true
/// ```
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    jwks_install::install(env);
}
