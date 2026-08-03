//! Argument error wording for the `redis` capability.
//!
//! One concern: turning a coercion or arity failure into a sentence. The wording
//! is centralized so every `redis.*` method reports a mismatch identically, and
//! so the two facts a caller needs — **which parameter** and **what it actually
//! got** — can never be omitted by an individual method.
//!
//! The in-tree style is `<capability>.<method>: <what went wrong>`, as in
//! `fs.read: capability lacks read mode` and
//! `db.query: expected a SQL string and parameter list`.
//!
//! # Never echo a value
//!
//! These helpers name the parameter and its *type*, never its contents. A Redis
//! `SET` payload can be a session token or a password hash, and an error string
//! travels into logs and script output; printing the value would leak it. The
//! type name is enough to fix the call.

use crate::value::Value;

/// Report a parameter whose type is wrong.
///
/// # Arguments
///
/// * `method` — Fully qualified method, e.g. `redis.setex`.
/// * `parameter` — The declared parameter name, e.g. `seconds`.
/// * `expected` — Prose for the accepted types, e.g. `an int`.
/// * `actual` — The supplied value, read only for [`Value::type_name`].
///
/// # Returns
///
/// A message naming the method, the parameter, the expectation, and the actual
/// type — never the value itself.
///
/// # Examples
///
/// ```rust
/// use std::rc::Rc;
/// use tetherscript::redis_cap::args_error;
/// use tetherscript::value::Value;
///
/// let message = args_error::mismatch(
///     "redis.setex",
///     "seconds",
///     "an int",
///     &Value::Str(Rc::new("60".to_string())),
/// );
/// assert_eq!(message, "redis.setex: parameter `seconds` must be an int, got str");
/// ```
pub fn mismatch(method: &str, parameter: &str, expected: &str, actual: &Value) -> String {
    format!(
        "{method}: parameter `{parameter}` must be {expected}, got {}",
        actual.type_name()
    )
}
