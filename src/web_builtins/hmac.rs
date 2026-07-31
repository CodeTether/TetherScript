//! HMAC and hex built-ins.
//!
//! Owner: sub-agent `hmac_hex`. Provides the primitives a web application needs
//! to sign and verify opaque tokens: HMAC-SHA-256, a hex codec, and a
//! constant-time comparison for checking signatures.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `hmac_sha256_hex(key, message)` | lowercase hex str |
//! | `hex_encode(input)` | lowercase hex str |
//! | `hex_decode(hex)` | `Result` of the decoded str |
//! | `constant_time_eq(a, b)` | bool |
//!
//! # Reuse
//!
//! The digest is [`crate::system::sha256`] and hex encoding is
//! [`crate::system::hex_encode`], so no primitive is reimplemented here. See
//! [`hmac_digest`] for why the HMAC construction is recomputed locally instead of
//! calling the equivalent in [`crate::postgres`].
//!
//! # Examples
//!
//! ```tether
//! let mac = hmac_sha256_hex("secret", "payload")
//! if constant_time_eq(mac, provided) { println("signature ok") }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "hmac_builtins.rs"]
pub(super) mod hmac_builtins;
#[path = "hmac_compare.rs"]
pub(super) mod hmac_compare;
#[path = "hmac_digest.rs"]
pub(super) mod hmac_digest;
#[path = "hmac_hex_codec.rs"]
pub(super) mod hmac_hex_codec;

pub(crate) use hmac_compare::constant_time_eq;
pub(crate) use hmac_digest::hmac_sha256;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define("hmac_sha256_hex", hmac_builtins::hmac_builtin(), false);
    bindings.define("hex_encode", hmac_builtins::hex_encode_builtin(), false);
    bindings.define("hex_decode", hmac_builtins::hex_decode_builtin(), false);
    bindings.define(
        "constant_time_eq",
        hmac_compare::constant_time_eq_builtin(),
        false,
    );
}
