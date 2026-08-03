//! Inference for columns whose type OID is unrecognised.
//!
//! This is what the decoder did for *every* column before type OIDs were carried through. It stays
//! as the last resort so an unrecognised type is usable rather than opaque, but it is no longer
//! applied to a column the server declared textual — which is what stopped a `varchar` holding
//! `"0123"` from becoming the integer 123.

use std::rc::Rc;

use crate::value::Value;

/// Guess a scalar from text alone.
///
/// # Arguments
///
/// * `text` — The field's text-format rendering.
///
/// # Returns
///
/// A `Bool` for `t`/`f`, then the narrowest number that parses, then `Str`.
pub(super) fn infer(text: &str) -> Value {
    match text {
        "t" => return Value::Bool(true),
        "f" => return Value::Bool(false),
        _ => {}
    }
    if let Ok(int) = text.parse::<i64>() {
        return Value::Int(int);
    }
    if let Ok(float) = text.parse::<f64>() {
        return Value::Float(float);
    }
    Value::Str(Rc::new(text.to_string()))
}
