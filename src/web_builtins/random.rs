//! Cryptographically unpredictable values for security-sensitive use.
//!
//! Session IDs, CSRF tokens, API keys, and password salts all need bytes an
//! attacker cannot guess or reproduce. These built-ins provide them; nothing else
//! script-visible in tetherscript does.
//!
//! # Security
//!
//! Three properties are load-bearing:
//!
//! 1. **Fresh OS entropy per call.** There is no cached PRNG state, so recovering
//!    one token tells an attacker nothing about the next. See
//!    [`super::random_source`].
//! 2. **No modulo bias.** `random_int` uses rejection sampling, not `%`, so every
//!    value in the range is equally likely. See [`super::random_range`].
//! 3. **Bounded sizes.** A single call is capped, so a bad argument cannot
//!    allocate unbounded memory inside a request handler.
//!
//! Do **not** use these for reproducible simulation: there is no seeding, by
//! design.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `random_bytes_hex(n)` | `Result` of `2n` lowercase hex characters |
//! | `random_token(n)` | `Result` of an unpadded URL-safe base64 token |
//! | `random_int(min, max)` | `Result` of an int in `[min, max)` |
//! | `random_choice(list)` | `Result` of one element |
//!
//! # Examples
//!
//! ```tether
//! let session_id = random_token(32)?
//! let salt = random_bytes_hex(16)?
//! let dice = random_int(1, 7)?
//! let pick = random_choice(["a", "b", "c"])?
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::Env;

use super::super::pure_native;

#[path = "random_codec.rs"]
mod codec;
#[path = "random_ops.rs"]
mod ops;
#[path = "random_range.rs"]
mod random_range;
#[path = "random_source.rs"]
mod random_source;

/// Register this group's built-ins.
///
/// Defines `random_bytes_hex`, `random_token`, `random_int`, and
/// `random_choice` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "random_bytes_hex",
        pure_native("random_bytes_hex", Some(1), |args| {
            Ok(result_value(ops::bytes_hex(&args[0])))
        }),
        false,
    );
    bindings.define(
        "random_token",
        pure_native("random_token", Some(1), |args| {
            Ok(result_value(ops::token(&args[0])))
        }),
        false,
    );
    bindings.define(
        "random_int",
        pure_native("random_int", Some(2), |args| {
            Ok(result_value(ops::int(&args[0], &args[1])))
        }),
        false,
    );
    bindings.define(
        "random_choice",
        pure_native("random_choice", Some(1), |args| {
            Ok(result_value(ops::choice(&args[0])))
        }),
        false,
    );
}
