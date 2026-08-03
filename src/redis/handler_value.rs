//! Mapping from RESP replies to tetherscript [`Value`]s.
//!
//! The mapping is deliberately lossless or loud: a reply that cannot be
//! represented faithfully produces a named error rather than a mangled value,
//! because a silently replaced byte in a cache read is worse than a failed read.
//!
//! | RESP | `Value` |
//! |---|---|
//! | `Bulk(None)`, `Nil` | `Value::Nil` (distinct from `Value::Str("")`) |
//! | `Bulk(Some(bytes))` | `Value::Str` when valid UTF-8, else an error |
//! | `Int` | `Value::Int` |
//! | `Simple` | `Value::Str` |
//! | `Array` | `Value::List`, or `Value::Nil` for a null array |
//! | `Error` | `Err(..)` |
//! | `Bool`, `Double`, `Map`, `Push` | see [`super::handler_value_resp3`] |

use std::rc::Rc;

use super::handler_value_resp3;
use super::resp::Resp;
use crate::value::Value;

/// Convert one reply into a tetherscript value.
///
/// # Arguments
///
/// * `reply` — A fully decoded RESP frame.
///
/// # Returns
///
/// The corresponding [`Value`]. A missing key yields [`Value::Nil`], which a script
/// can distinguish from a stored empty string (`Value::Str("")`).
///
/// # Errors
///
/// Returns the server's message for `Resp::Error`, or a named decode error when a
/// bulk payload is not valid UTF-8.
pub(super) fn from_resp(reply: Resp) -> Result<Value, String> {
    match reply {
        Resp::Simple(text) => Ok(Value::Str(Rc::new(text))),
        Resp::Error(message) => Err(format!("redis: {message}")),
        Resp::Int(number) => Ok(Value::Int(number)),
        Resp::Bulk(None) | Resp::Nil => Ok(Value::Nil),
        Resp::Bulk(Some(bytes)) => bulk(bytes),
        Resp::Array(None) => Ok(Value::Nil),
        Resp::Array(Some(items)) => handler_value_resp3::list(items),
        // RESP3-only variants; matched explicitly so this dispatch is exhaustive.
        resp3 @ (Resp::Bool(_) | Resp::Double(_) | Resp::Map(_) | Resp::Push(_)) => {
            handler_value_resp3::from_resp(resp3)
        }
    }
}

/// Convert a bulk payload, refusing to lose bytes.
///
/// # Errors
///
/// Returns a named error naming the invalid byte offset when the payload is not
/// valid UTF-8, instead of substituting replacement characters.
fn bulk(bytes: Vec<u8>) -> Result<Value, String> {
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Value::Str(Rc::new(text))),
        Err(error) => Err(format!(
            "redis: reply is not valid UTF-8 at byte {}; read it with a binary-aware command",
            error.utf8_error().valid_up_to()
        )),
    }
}
