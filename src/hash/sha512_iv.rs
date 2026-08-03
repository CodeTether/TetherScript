//! SHA-512 and SHA-384 initial hash values, FIPS 180-4 §5.3.5 and §5.3.4.
//!
//! SHA-512's IV is the first 64 bits of the fractional parts of the square
//! roots of the first eight primes (2..19). SHA-384's IV is the same
//! construction over the *ninth through sixteenth* primes (23..53). That is why
//! SHA-384 is not merely a truncation of SHA-512: the two chains start from
//! different states, so their digests disagree in essentially every byte. The
//! integration test `sha384_is_not_truncated_sha512` asserts that inequality
//! directly, so the shortcut cannot be reintroduced silently.

/// SHA-512 initial hash value.
#[rustfmt::skip]
pub(crate) const IV512: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

/// SHA-384 initial hash value.
#[rustfmt::skip]
pub(crate) const IV384: [u64; 8] = [
    0xcbbb9d5dc1059ed8, 0x629a292a367cd507,
    0x9159015a3070dd17, 0x152fecd8f70e5939,
    0x67332667ffc00b31, 0x8eb44a8768581511,
    0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4,
];
