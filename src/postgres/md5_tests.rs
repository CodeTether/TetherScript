//! MD5 coverage against the RFC 1321 test suite.
//!
//! MD5 is only present for PostgreSQL's legacy `md5` auth method, but a silently
//! wrong digest would surface as an unexplained authentication failure, so the
//! published vectors are pinned here.

use super::md5::digest;
use super::md5_password::postgres_password;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn matches_rfc1321_empty_string() {
    assert_eq!(hex(&digest(b"")), "d41d8cd98f00b204e9800998ecf8427e");
}

#[test]
fn matches_rfc1321_abc() {
    assert_eq!(hex(&digest(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
}

#[test]
fn matches_rfc1321_alphabet() {
    assert_eq!(
        hex(&digest(b"abcdefghijklmnopqrstuvwxyz")),
        "c3fcd3d76192e4007dfb496cca67e13b"
    );
}

/// Spans a block boundary, exercising the padding path.
#[test]
fn matches_rfc1321_long_digits() {
    assert_eq!(
        hex(&digest(
            b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"
        )),
        "57edf4a22be3c955ac49da2e2107b67a"
    );
}

/// PostgreSQL's `md5` secret is `"md5" + md5(md5(password + user) + salt)`.
#[test]
fn postgres_password_is_double_hashed_with_the_salt() {
    let salt = [0x01, 0x02, 0x03, 0x04];
    let secret = postgres_password("postgres", "secret", &salt);
    assert!(secret.starts_with("md5"), "got: {secret}");
    assert_eq!(secret.len(), 35, "md5 + 32 hex chars, got: {secret}");

    // Recompute the documented construction independently.
    let stage1 = hex(&digest(b"secretpostgres"));
    let mut outer = stage1.into_bytes();
    outer.extend_from_slice(&salt);
    assert_eq!(secret, format!("md5{}", hex(&digest(&outer))));
}
