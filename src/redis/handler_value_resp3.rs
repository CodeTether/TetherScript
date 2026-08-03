//! RESP3 reply variants and array conversion.
//!
//! Split from [`super::handler_value`] to keep each file focused: that module owns
//! the RESP2 core every server speaks, this one owns the RESP3 additions and the
//! recursive array walk.
//!
//! RESP3 arrives only when the connection negotiated protocol 3
//! ([`Connection::protocol`](super::connection::Connection::protocol)), so a
//! RESP2 deployment never reaches most of this code.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::resp::Resp;
use crate::value::Value;

/// Convert an array (or push) body into a [`Value::List`], recursively.
///
/// # Arguments
///
/// * `items` — Elements of the array reply, in server order.
///
/// # Returns
///
/// A [`Value::List`] preserving element order, since Redis array order is
/// meaningful (`MGET` answers positionally).
///
/// # Errors
///
/// Propagates the first element that cannot be converted, including a nested
/// `Resp::Error`, so a partly-bad reply is never returned as if it were whole.
pub(super) fn list(items: Vec<Resp>) -> Result<Value, String> {
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        values.push(super::handler_value::from_resp(item)?);
    }
    Ok(Value::List(Rc::new(RefCell::new(values))))
}

/// Convert the RESP3-only reply variants.
///
/// # Arguments
///
/// * `reply` — A `Bool`, `Double`, `Map`, or `Push` frame.
///
/// # Returns
///
/// `Bool` becomes [`Value::Bool`], `Double` becomes [`Value::Float`], `Map` becomes
/// [`Value::Map`], and `Push` becomes a [`Value::List`].
///
/// # Errors
///
/// Returns a named error when a map key is not a string-like reply, because a
/// stringified structural key would collide unpredictably. Also propagates any
/// nested conversion failure.
pub(super) fn from_resp(reply: Resp) -> Result<Value, String> {
    match reply {
        Resp::Bool(flag) => Ok(Value::Bool(flag)),
        Resp::Double(number) => Ok(Value::Float(number)),
        Resp::Push(items) => list(items),
        Resp::Map(pairs) => map(pairs),
        // Unreachable: `handler_value` routes only the four RESP3 variants here and
        // is exhaustive over the rest. Erroring instead of delegating back means a
        // future variant cannot turn a missed arm into infinite mutual recursion.
        _ => Err("redis: unsupported reply kind".into()),
    }
}

/// Build a [`Value::Map`] from RESP3 key/value pairs.
///
/// # Errors
///
/// Returns a named error when a key does not convert to a string.
fn map(pairs: Vec<(Resp, Resp)>) -> Result<Value, String> {
    let mut fields = HashMap::with_capacity(pairs.len());
    for (key, value) in pairs {
        let name = match super::handler_value::from_resp(key)? {
            Value::Str(text) => text.to_string(),
            other => {
                return Err(format!(
                    "redis: map reply key is a {}, which has no string form",
                    other.type_name()
                ));
            }
        };
        fields.insert(name, super::handler_value::from_resp(value)?);
    }
    Ok(Value::Map(Rc::new(RefCell::new(fields))))
}
