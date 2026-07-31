//! Base32 built-ins (RFC 4648).
//!
//! TOTP secrets and several external APIs are specified in base32, so the port
//! needs a codec that is exact about padding: a decoder that quietly accepts
//! malformed padding will happily read a corrupted shared secret.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `base32_encode(input)` | uppercase padded base32 str |
//! | `base32_encode_nopad(input)` | uppercase unpadded base32 str |
//! | `base32_decode(text)` | `Result` of the decoded str |
//!
//! Decoding accepts either case and either padding style, but rejects a
//! non-alphabet character, padding in the middle, excess padding, an impossible
//! length, and non-zero unused tail bits, so no two spellings decode alike.
//!
//! # Examples
//!
//! ```tether
//! println(base32_encode("foobar"))          // MZXW6YTBOI======
//! println(base32_decode("MZXW6YTB").unwrap()) // fooba
//! ```
//!
//! # Reconstruction note
//!
//! This entry point was rebuilt by the integrator after a parallel agent deleted
//! it; the concern modules below are the original implementation.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "base32_decode.rs"]
pub(super) mod base32_decode;
#[path = "base32_encode.rs"]
pub(super) mod base32_encode;
#[path = "base32_install.rs"]
pub(super) mod base32_install;
#[path = "base32_validate.rs"]
pub(super) mod base32_validate;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    base32_install::install(env);
}
