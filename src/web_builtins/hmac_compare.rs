//! Constant-time comparison for signature checking.
//!
//! Signature verification must not leak *where* two values first differ. A plain
//! `==` on byte slices short-circuits, so an attacker who can time many attempts
//! can recover a signature byte by byte. Everything here inspects every byte.

use super::super::super::pure_native;
use super::hmac_builtins::str_arg;
use crate::value::Value;

/// `constant_time_eq(a, b)` -> bool.
pub(super) fn constant_time_eq_builtin() -> Value {
    pure_native("constant_time_eq", Some(2), |args| {
        let left = str_arg(&args[0], "constant_time_eq: a")?;
        let right = str_arg(&args[1], "constant_time_eq: b")?;
        Ok(Value::Bool(constant_time_eq(
            left.as_bytes(),
            right.as_bytes(),
        )))
    })
}

/// Compare two byte strings without leaking the first difference via timing.
///
/// Every byte pair is folded into an accumulator, so the running time depends on
/// the input length rather than on where a mismatch occurs. Unequal lengths
/// return early: the length of a value is not the secret, its contents are.
///
/// # Arguments
///
/// * `left` — First byte string.
/// * `right` — Second byte string.
///
/// # Returns
///
/// True when both sides are byte-identical.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(constant_time_eq(b"abc", b"abc"));
/// assert!(!constant_time_eq(b"abc", b"abd"));
/// assert!(!constant_time_eq(b"abc", b"ab"));
/// ```
pub(crate) fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}
