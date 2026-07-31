//! Unit coverage for the SCRAM crypto primitives.
//!
//! Vectors come from RFC 4231 (HMAC-SHA-256) and RFC 7677 (SCRAM-SHA-256), so a
//! regression in the in-tree SHA-256 or the HMAC padding is caught here rather
//! than surfacing as an opaque authentication failure.

use super::hmac::{hmac_sha256, pbkdf2_sha256};
use crate::system::hex_encode;

#[test]
fn hmac_matches_rfc4231_case_1() {
    let key = [0x0bu8; 20];
    let mac = hmac_sha256(&key, b"Hi There");
    assert_eq!(
        hex_encode(&mac),
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
}

#[test]
fn hmac_matches_rfc4231_case_2() {
    let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
    assert_eq!(
        hex_encode(&mac),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
}

/// Key longer than the 64-byte block must be hashed before padding.
#[test]
fn hmac_handles_oversized_key() {
    let key = [0xaau8; 131];
    let mac = hmac_sha256(
        &key,
        b"Test Using Larger Than Block-Size Key - Hash Key First",
    );
    assert_eq!(
        hex_encode(&mac),
        "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
    );
}

/// RFC 6070 PBKDF2 vector, adapted to HMAC-SHA-256: password "password",
/// salt "salt", 4096 iterations. The published SHA-256 answer for this input is
/// c5e478d5.., which pins both the iteration XOR chain and the HMAC below it.
#[test]
fn pbkdf2_matches_rfc6070_style_sha256_vector() {
    let salted = pbkdf2_sha256(b"password", b"salt", 4096);
    assert_eq!(
        hex_encode(&salted),
        "c5e478d59288c841aa530db6845c4c8d962893a001ce4e11a4963873aa98134a"
    );
}

/// A single iteration must reduce to one plain HMAC, with no XOR folding.
#[test]
fn pbkdf2_single_iteration_equals_hmac() {
    let expected = hmac_sha256(b"pencil", b"salt\0\0\0\x01");
    assert_eq!(pbkdf2_sha256(b"pencil", b"salt", 1), expected);
}
