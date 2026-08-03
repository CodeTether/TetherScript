//! Resource bounds for JWKS documents fetched from a remote server.
//!
//! One responsibility: hold the numeric limits every other JWKS concern applies.
//! They live in one file so a deployment can audit the whole attack surface of
//! "how much work can a remote JWKS endpoint make us do" by reading 8 lines.
//!
//! # Why bounds at all
//!
//! A JWKS is fetched over the network from an identity provider. That provider
//! is trusted to publish keys, but the *transport* is not trusted to deliver a
//! small document: a compromised or buggy endpoint, or anything able to answer
//! in its place, can return a 4 GiB `keys` array. Every limit below is a refusal
//! to allocate unbounded memory on a stranger's say-so.
//!
//! # The numbers, and why these numbers
//!
//! | Limit | Value | Rationale |
//! |---|---|---|
//! | [`MAX_DOCUMENT_BYTES`] | 262144 (256 KiB) | A Keycloak realm with 64 8192-bit keys still fits; nothing legitimate is larger. |
//! | [`MAX_KEYS`] | 64 | Realms rotate, they do not accumulate. 64 is ~5 years of monthly rotation. |
//! | [`MAX_FIELD_CHARS`] | 4096 | An 8192-bit modulus is 1366 base64url chars; 4096 leaves headroom without permitting a megabyte `kid`. |
//! | [`MAX_KEY_OPS`] | 16 | RFC 7517 §4.3 registers 10 values; 16 allows unregistered ones without permitting an unbounded array. |
//! | [`MIN_MODULUS_BITS`] | 2048 | 1024-bit RSA is not safe for new signatures, and a JWKS is where a downgrade would be smuggled in. |
//! | [`MIN_MODULUS_BYTES`] | 256 | The byte-length form of the same rule, checked first for a clearer error. |
//! | [`MAX_MODULUS_BYTES`] | 1024 (8192 bits) | Verification cost is quadratic in modulus size, so an absurd modulus is a CPU-exhaustion vector. |
//! | [`MAX_EXPONENT_BYTES`] | 8 | Real exponents are 3 bytes (65537). 8 is generous and fits a `u64`. |

/// Largest accepted JWKS document, in bytes of UTF-8 source.
pub const MAX_DOCUMENT_BYTES: usize = 256 * 1024;

/// Largest accepted number of entries in the `keys` array.
pub const MAX_KEYS: usize = 64;

/// Largest accepted length, in bytes, of any single JSON string member of a JWK.
pub const MAX_FIELD_CHARS: usize = 4096;

/// Largest accepted number of entries in a JWK `key_ops` array.
pub const MAX_KEY_OPS: usize = 16;

/// Smallest accepted RSA modulus, in significant bits.
pub const MIN_MODULUS_BITS: usize = 2048;

/// Smallest accepted RSA modulus, in bytes.
pub const MIN_MODULUS_BYTES: usize = 256;

/// Largest accepted RSA modulus, in bytes.
pub const MAX_MODULUS_BYTES: usize = 1024;

/// Largest accepted RSA public exponent, in bytes.
pub const MAX_EXPONENT_BYTES: usize = 8;
