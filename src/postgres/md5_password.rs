//! The `md5`-method password secret.
//!
//! PostgreSQL expects `"md5" + md5(md5(password + user) + salt)`, where each
//! inner digest is lower-case hex before being fed forward.

use super::md5::digest;

pub(super) fn postgres_password(user: &str, password: &str, salt: &[u8]) -> String {
    let mut inner = Vec::new();
    inner.extend_from_slice(password.as_bytes());
    inner.extend_from_slice(user.as_bytes());
    let stage1 = hex(&digest(&inner));

    let mut outer = Vec::new();
    outer.extend_from_slice(stage1.as_bytes());
    outer.extend_from_slice(salt);
    format!("md5{}", hex(&digest(&outer)))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
