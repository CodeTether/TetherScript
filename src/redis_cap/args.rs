//! Positional argument access.
//!
//! One concern: fetching argument *n* under its declared name, so a missing argument is
//! reported the same way everywhere and an extra one is never ignored.
//!
//! Arity is checked before types on purpose: told that `redis.setex` takes three
//! arguments, a caller can fix the call in one step, whereas a type complaint about the
//! argument that landed in the wrong slot sends them the wrong way.

use super::args_missing;
use crate::value::Value;

/// Borrow the argument at `index`, or report it missing.
///
/// # Arguments
///
/// * `method` — Fully qualified method, for the error message.
/// * `parameter` — Declared name of this positional parameter.
/// * `arguments` — The full argument list as supplied by the script.
/// * `index` — Zero-based position of `parameter`.
///
/// # Returns
///
/// A borrow of the supplied value.
///
/// # Errors
///
/// Returns [`args_missing::missing`] naming `parameter` when the list is shorter than
/// `index + 1`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::args;
/// use tetherscript::value::Value;
///
/// let supplied = [Value::Int(7)];
/// let found = args::at("redis.incrby", "key", &supplied, 0).expect("present");
/// assert!(matches!(found, Value::Int(7)));
///
/// let error = args::at("redis.incrby", "delta", &supplied, 1).unwrap_err();
/// assert_eq!(error, "redis.incrby: parameter `delta` is required");
/// ```
pub fn at<'a>(
    method: &str,
    parameter: &str,
    arguments: &'a [Value],
    index: usize,
) -> Result<&'a Value, String> {
    arguments
        .get(index)
        .ok_or_else(|| args_missing::missing(method, parameter))
}

/// Refuse an argument list that is not exactly the length the method accepts.
///
/// # Arguments
///
/// * `method` — Fully qualified method, for the error message.
/// * `arguments` — The full argument list.
/// * `expected` — How many the method takes.
///
/// # Returns
///
/// `Ok(())` when the count matches exactly.
///
/// # Errors
///
/// Returns an error naming both counts. Extra arguments are refused rather than dropped:
/// silently ignoring the third argument to `redis.set` would hide a caller who believed
/// they were passing a TTL and instead got a key with no expiry.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::args;
/// use tetherscript::value::Value;
///
/// assert!(args::exactly("redis.ping", &[], 0).is_ok());
/// let error = args::exactly("redis.ping", &[Value::Nil], 0).unwrap_err();
/// assert!(error.contains("takes 0"), "got: {error}");
/// ```
pub fn exactly(method: &str, arguments: &[Value], expected: usize) -> Result<(), String> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(format!(
        "{method}: takes {expected} argument(s), got {}",
        arguments.len()
    ))
}
