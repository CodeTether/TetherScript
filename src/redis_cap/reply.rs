//! Mapping a Redis reply into a script-facing [`Value`].
//!
//! One concern: representing an answer without losing information.
//!
//! # Nil is not the empty string
//!
//! This is the distinction the whole module exists to preserve.
//! [`Connection::get`](crate::redis::Connection::get) returns `Option<Vec<u8>>`,
//! and the two absences are different facts:
//!
//! | Server sent | Rust | `Value` | Meaning |
//! |---|---|---|---|
//! | `$-1\r\n` | `None` | [`Value::Nil`] | **cache miss** — no such key |
//! | `$0\r\n\r\n` | `Some(vec![])` | `Value::Str("")` | **cache hit** — key holds `""` |
//!
//! Flattening them is a real bug, not a nicety. A render cache that maps both to
//! `""` re-renders forever on a legitimately-empty page. A session store that maps
//! both to `nil` cannot distinguish a logged-out user from one whose session value
//! is empty. Scripts test the difference with `== nil`, which is false for `""`
//! because [`Value::Nil`] and [`Value::Str`] are different variants.
//!
//! [`Value::truthy`] is *not* the way to test this: `nil` and `""` are both falsey.
//!
//! # Binary values
//!
//! A stored value that is not valid UTF-8 becomes [`Value::Bytes`] rather than
//! erroring or substituting replacement characters, so a cached PNG round-trips. A
//! script distinguishes the two with the usual `type_name`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

/// Convert an optional bulk payload, keeping absence distinct from emptiness.
///
/// # Arguments
///
/// * `payload` — `None` for the null bulk string, `Some(bytes)` for a present
///   value including a zero-length one.
///
/// # Returns
///
/// [`Value::Nil`] for `None`; [`Value::Str`] for valid UTF-8; [`Value::Bytes`]
/// otherwise. Never returns `Value::Str("")` for a missing key.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::reply;
/// use tetherscript::value::Value;
///
/// // A miss and a cached empty string are different values.
/// assert!(matches!(reply::optional_bulk(None), Value::Nil));
/// match reply::optional_bulk(Some(Vec::new())) {
///     Value::Str(text) => assert_eq!(text.as_str(), ""),
///     other => panic!("expected an empty str, got {}", other.type_name()),
/// }
///
/// // Binary data is preserved rather than lossily decoded.
/// assert_eq!(reply::optional_bulk(Some(vec![0xff])).type_name(), "bytes");
/// ```
pub fn optional_bulk(payload: Option<Vec<u8>>) -> Value {
    match payload {
        None => Value::Nil,
        Some(bytes) => match String::from_utf8(bytes) {
            Ok(text) => Value::Str(Rc::new(text)),
            Err(error) => Value::Bytes(Rc::new(RefCell::new(error.into_bytes()))),
        },
    }
}
