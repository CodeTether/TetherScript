//! Converting a decoded RESP reply into a script [`Value`].
//!
//! Written against the RESP2 codec in [`super::value`], whose null forms are distinct
//! variants (`NullBulk`, `NullArray`) rather than `Option` payloads. That distinction is
//! load-bearing for a session store: a missing key and a stored empty string are
//! different answers, and collapsing them cannot tell a logged-out user from one whose
//! session value happens to be empty.

use std::rc::Rc;

use super::value::RespValue;
use crate::value::Value;

/// Convert a decoded reply to a script value.
///
/// # Arguments
///
/// * `reply` — The decoded RESP reply.
///
/// # Returns
///
/// The corresponding [`Value`]. A missing key yields [`Value::Nil`], which a script can
/// distinguish from a stored empty string (`Value::Str("")`).
///
/// # Errors
///
/// Returns the server's message for an error reply, or a named decode error when a bulk
/// payload is not valid UTF-8.
pub(super) fn from_resp(reply: RespValue) -> Result<Value, String> {
    match reply {
        RespValue::Simple(text) => Ok(Value::Str(Rc::new(text))),
        RespValue::Error { kind, message } => Err(format!("redis: {kind} {message}")),
        RespValue::Integer(number) => Ok(Value::Int(number)),
        // Both null forms collapse to nil at the script boundary, where tetherscript has
        // only one absent value. The codec keeps them apart so this decision is explicit.
        RespValue::NullBulk | RespValue::NullArray => Ok(Value::Nil),
        RespValue::Bulk(bytes) => bulk(bytes),
        RespValue::Array(items) => super::handler_value_resp3::list(items),
    }
}

/// Convert a bulk payload, refusing to lose bytes.
///
/// # Errors
///
/// Returns an error naming the invalid byte offset when the payload is not valid UTF-8,
/// instead of substituting replacement characters — a script that stored bytes should be
/// told they cannot be represented rather than handed corrupted text.
fn bulk(bytes: Vec<u8>) -> Result<Value, String> {
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Value::Str(Rc::new(text))),
        Err(error) => Err(format!(
            "redis: bulk payload is not valid UTF-8 at byte {}",
            error.utf8_error().valid_up_to()
        )),
    }
}
