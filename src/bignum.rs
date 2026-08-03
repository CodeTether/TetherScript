//! Arbitrary-precision unsigned integer arithmetic.
//!
//! Exists for one reason: RSA signature verification needs modular exponentiation over
//! 2048-bit integers, and nothing in-tree could do that. [`Uint`] stores little-endian
//! `u64` limbs and is a pure numeric type — it does not touch [`crate::value::Value`],
//! so it stays independently testable.
//!
//! No dependency on `num-bigint` or any other crate, matching how JSON, HTTP, and the
//! PostgreSQL client are all written in-tree.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | The type and its limbs | `uint`, `bits`, `cmp`, `shift` |
//! | Conversion | `bytes`, `parse`, `format`, `display` |
//! | Arithmetic | `add`, `sub`, `mul` |
//! | Division (Knuth algorithm D) | `div`, `div_estimate`, `div_knuth`, `div_mulsub`, `div_step` |
//! | Modular arithmetic | `modular`, `modpow` |
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bignum::Uint;
//!
//! // Round-trips through the minimal big-endian encoding.
//! let a = Uint::from_be_bytes(&[0x01, 0x00]);
//! assert_eq!(a.to_be_bytes(), vec![0x01, 0x00]);
//! ```

#[path = "bignum/add.rs"]
mod add;
#[path = "bignum/bits.rs"]
mod bits;
#[path = "bignum/bytes.rs"]
mod bytes;
#[path = "bignum/cmp.rs"]
mod cmp;
#[path = "bignum/display.rs"]
mod display;
#[path = "bignum/div.rs"]
mod div;
#[path = "bignum/div_estimate.rs"]
mod div_estimate;
#[path = "bignum/div_knuth.rs"]
mod div_knuth;
#[path = "bignum/div_mulsub.rs"]
mod div_mulsub;
#[path = "bignum/div_step.rs"]
mod div_step;
#[path = "bignum/error.rs"]
mod error;
#[path = "bignum/format.rs"]
mod format;
#[path = "bignum/modpow.rs"]
mod modpow;
#[path = "bignum/modular.rs"]
mod modular;
#[path = "bignum/mul.rs"]
mod mul;
#[path = "bignum/parse.rs"]
mod parse;
#[path = "bignum/shift.rs"]
mod shift;
#[path = "bignum/sub.rs"]
mod sub;
#[path = "bignum/uint.rs"]
pub mod uint;

pub use error::ParseUintError;
pub use uint::Uint;
