//! Arbitrary-precision unsigned integers for RSA.
//!
//! Exists so RSA signature verification can do modular exponentiation over 2048-bit
//! integers. Built alongside [`crate::bignum`] by a separate concern: this one is shaped
//! around what [`crate::rsa`] needs and is the type that module depends on, while `bignum`
//! is a general-purpose sibling. Consolidating the two is open work.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Limb storage and normalization | `limbs`, `bits`, `compare` |
//! | Conversion | `bytes`, `hex` |
//! | Arithmetic | `addsub`, `carry`, `mul`, `divmod`, `shift` |
//! | Modular arithmetic | `modmul`, `modpow` |
//! | Errors | `error` |
//!
//! Limb products use `u128` intermediates, since multiplying two `u64` limbs overflows 64
//! bits. `mod_pow` is square-and-multiply and is **not** constant-time — acceptable here
//! because signature verification handles public data only, but stated rather than assumed.

#[path = "bigint/addsub.rs"]
mod addsub;
#[path = "bigint/bits.rs"]
mod bits;
#[path = "bigint/bytes.rs"]
mod bytes;
#[path = "bigint/carry.rs"]
mod carry;
#[cfg(test)]
#[path = "bigint/carry_tests.rs"]
mod carry_tests;
#[path = "bigint/compare.rs"]
mod compare;
#[path = "bigint/divmod.rs"]
mod divmod;
#[path = "bigint/error.rs"]
pub mod error;
#[path = "bigint/hex.rs"]
mod hex;
#[path = "bigint/limbs.rs"]
pub mod limbs;
#[cfg(test)]
#[path = "bigint/limbs_tests.rs"]
mod limbs_tests;
#[path = "bigint/modmul.rs"]
mod modmul;
#[path = "bigint/modpow.rs"]
mod modpow;
#[path = "bigint/mul.rs"]
mod mul;
#[path = "bigint/shift.rs"]
mod shift;

pub use error::BigUintError;
pub use limbs::BigUint;
