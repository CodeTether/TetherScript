//! Wording for an argument that was not supplied at all.
//!
//! Split from [`super::args_error`] because "wrong type" and "absent" are
//! different failures with different fixes, and because a missing argument has no
//! [`Value`](crate::value::Value) to describe.

/// Report a parameter the caller omitted.
///
/// # Arguments
///
/// * `method` — Fully qualified method, e.g. `redis.get`.
/// * `parameter` — The declared parameter name that has no argument.
///
/// # Returns
///
/// A message naming the method and the parameter. Deliberately distinct from
/// [`super::args_error::mismatch`]: `got nil` would be wrong, because passing an
/// explicit `nil` and passing nothing are different mistakes.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::args_missing;
///
/// assert_eq!(
///     args_missing::missing("redis.get", "key"),
///     "redis.get: parameter `key` is required"
/// );
/// ```
pub fn missing(method: &str, parameter: &str) -> String {
    format!("{method}: parameter `{parameter}` is required")
}
