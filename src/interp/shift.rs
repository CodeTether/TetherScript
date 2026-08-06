//! Shift-count validation for `<<` and `>>`.
//!
//! Rust panics in debug builds when a shift count is at least the operand's bit
//! width, and silently masks it in release. Neither is acceptable for a scripting
//! language, so shifts are checked and report a named error instead.

use crate::value::Value;

/// Bit width of the integer type backing [`Value::Int`].
const INT_BITS: i64 = i64::BITS as i64;

/// Apply `a << b` or `a >> b` after validating the shift count.
///
/// `>>` is an arithmetic shift: it preserves the sign bit, matching Rust's
/// behavior on `i64` rather than JavaScript's `>>>`.
///
/// # Arguments
///
/// * `a` — the value being shifted.
/// * `b` — the shift count; must be in `0..64`.
/// * `operator` — `"<<"` or `">>"`, used in the error message.
///
/// # Errors
///
/// Returns an error naming the operator and the offending count when `b` is
/// negative or at least 64.
///
/// # Examples
///
/// ```rust
/// use tetherscript::interp::shift::apply;
/// use tetherscript::value::Value;
///
/// assert_eq!(apply(1, 4, "<<").unwrap(), Value::Int(16));
/// assert_eq!(apply(-8, 1, ">>").unwrap(), Value::Int(-4));
/// assert!(apply(1, 64, "<<").is_err());
/// assert!(apply(1, -1, ">>").is_err());
/// ```
pub fn apply(a: i64, b: i64, operator: &str) -> Result<Value, String> {
    if b < 0 {
        return Err(format!("shift count for `{operator}` is negative: {b}"));
    }
    if b >= INT_BITS {
        return Err(format!(
            "shift count for `{operator}` must be less than {INT_BITS}, got {b}"
        ));
    }
    let shifted = if operator == "<<" { a << b } else { a >> b };
    Ok(Value::Int(shifted))
}
