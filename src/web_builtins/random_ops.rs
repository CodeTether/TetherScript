//! The four `random_*` operations, separated from registration.

use std::rc::Rc;

use crate::system::hex_encode;
use crate::value::Value;

use super::codec::{base64_url, byte_count};
use super::random_range::below;
use super::random_source::bytes;

/// `random_bytes_hex(n)` — `2n` lowercase hex characters.
///
/// # Arguments
///
/// * `count` — Number of random bytes; the string is twice this long.
///
/// # Returns
///
/// Lowercase hex, suitable for a password salt or an opaque identifier.
///
/// # Errors
///
/// Returns a named error when `count` is not a positive int within the cap.
pub(super) fn bytes_hex(count: &Value) -> Result<Value, String> {
    let len = byte_count(count, "random_bytes_hex")?;
    Ok(Value::Str(Rc::new(hex_encode(&bytes(len)))))
}

/// `random_token(n)` — an unpadded URL-safe base64 token from `n` bytes.
///
/// # Arguments
///
/// * `count` — Number of random bytes behind the token. 32 gives 256 bits.
///
/// # Returns
///
/// A token safe to place in a URL, cookie, or header without further escaping.
///
/// # Errors
///
/// Returns a named error when `count` is not a positive int within the cap.
pub(super) fn token(count: &Value) -> Result<Value, String> {
    let len = byte_count(count, "random_token")?;
    Ok(Value::Str(Rc::new(base64_url(&bytes(len)))))
}

/// `random_int(min, max)` — a uniform int in the half-open range `[min, max)`.
///
/// # Arguments
///
/// * `min` — Inclusive lower bound.
/// * `max` — Exclusive upper bound.
///
/// # Returns
///
/// A uniformly distributed int. The range is half-open so `random_int(0, len)`
/// indexes a list of length `len` without an off-by-one.
///
/// # Errors
///
/// Returns a named error when either bound is not an int, or when
/// `min >= max`, which describes an empty range and has no valid answer.
pub(super) fn int(min: &Value, max: &Value) -> Result<Value, String> {
    let (Value::Int(low), Value::Int(high)) = (min, max) else {
        return Err(format!(
            "random_int: min and max must be int, got {} and {}",
            min.type_name(),
            max.type_name()
        ));
    };
    if low >= high {
        return Err(format!(
            "random_int: min must be less than max, got min={low} and max={high}"
        ));
    }
    // Width as u64: the subtraction is done in i128 so `random_int(i64::MIN,
    // i64::MAX)` cannot overflow on the way to the span.
    let span = (*high as i128 - *low as i128) as u64;
    let offset = below(span);
    Ok(Value::Int(low.wrapping_add(offset as i64)))
}

/// `random_choice(list)` — one uniformly chosen element.
///
/// # Arguments
///
/// * `list` — The list to choose from. It is not modified.
///
/// # Returns
///
/// A clone of one element, chosen with the same unbiased draw as `random_int`.
///
/// # Errors
///
/// Returns a named error when the argument is not a list, or when it is empty and
/// therefore has nothing to return.
pub(super) fn choice(list: &Value) -> Result<Value, String> {
    let Value::List(items) = list else {
        return Err(format!(
            "random_choice: argument must be list, got {}",
            list.type_name()
        ));
    };
    let items = items.borrow();
    if items.is_empty() {
        return Err("random_choice: list is empty".into());
    }
    Ok(items[below(items.len() as u64) as usize].clone())
}
