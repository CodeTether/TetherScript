//! JWKS (JSON Web Key Set) parsing and key selection.
//!
//! Completes the RS256 path: a Keycloak-style provider publishes its signing keys as a
//! JWKS document, and verifying a token means selecting the right key from it by `kid` and
//! algorithm, then handing the modulus and exponent to RSA verification.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Document and key shape | `document`, `keys_array`, `key`, `keyset`, `fields` |
//! | RSA components | `modulus`, `exponent`, `bits`, `base64url` |
//! | Algorithm and usage | `alg`, `key_ops` |
//! | Selection | `select`, `select_kid`, `select_suits` |
//! | Bounds and errors | `limits`, `error*` |
//!
//! # Security posture
//!
//! A JWKS document arrives over the network, so selection is strict by design. A key is
//! chosen only when its algorithm matches what the token claims *and* what the verifier
//! accepts — never by trusting the token's own header alone, which is the classic
//! algorithm-confusion attack. Key material is bounded so a hostile document cannot force
//! an unbounded allocation, and `kid` matching is exact rather than prefix-based.

#[path = "jwks/alg.rs"]
pub mod alg;
#[path = "jwks/base64url.rs"]
pub mod base64url;
#[path = "jwks/bits.rs"]
pub mod bits;
#[path = "jwks/document.rs"]
pub mod document;
#[path = "jwks/error.rs"]
pub mod error;
#[path = "jwks/error_display.rs"]
mod error_display;
#[path = "jwks/example.rs"]
pub mod example;
#[path = "jwks/exponent.rs"]
pub mod exponent;
#[path = "jwks/fields.rs"]
mod fields;
#[path = "jwks/key.rs"]
pub mod key;
#[path = "jwks/key_ops.rs"]
mod key_ops;
#[path = "jwks/keys_array.rs"]
mod keys_array;
#[path = "jwks/keyset.rs"]
pub mod keyset;
#[path = "jwks/limits.rs"]
pub mod limits;
#[path = "jwks/modulus.rs"]
pub mod modulus;
#[path = "jwks/parse_key.rs"]
mod parse_key;
#[path = "jwks/select.rs"]
pub mod select;
#[path = "jwks/select_kid.rs"]
mod select_kid;
#[path = "jwks/select_suits.rs"]
mod select_suits;
#[path = "jwks/select_unique.rs"]
mod select_unique;
#[path = "jwks/skipped.rs"]
mod skipped;
#[path = "jwks/usage.rs"]
pub mod usage;
