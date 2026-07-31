//! Parameter encoding for the extended query protocol.
//!
//! Parameters are sent in text format with an unspecified type OID, letting the
//! server infer types from the statement. This is what matters for safety: the
//! value never enters the SQL string, so it cannot alter the parsed statement.

use crate::value::Value;

/// A single bound parameter: `None` is SQL NULL, `Some` is text-format bytes.
pub(super) type Parameter = Option<Vec<u8>>;

/// Convert tetherscript values into text-format wire parameters.
///
/// # Errors
///
/// Returns an error naming the offending position and type when a value has no
/// faithful text-format encoding, rather than silently stringifying it.
pub(super) fn encode_all(parameters: &[Value]) -> Result<Vec<Parameter>, String> {
    parameters
        .iter()
        .enumerate()
        .map(|(index, value)| {
            encode(value).map_err(|error| format!("db.query: parameter ${}: {error}", index + 1))
        })
        .collect()
}

fn encode(value: &Value) -> Result<Parameter, String> {
    match value {
        Value::Nil => Ok(None),
        Value::Int(int) => Ok(Some(int.to_string().into_bytes())),
        Value::Float(float) => Ok(Some(float.to_string().into_bytes())),
        // PostgreSQL accepts `t`/`f` for boolean input.
        Value::Bool(true) => Ok(Some(b"t".to_vec())),
        Value::Bool(false) => Ok(Some(b"f".to_vec())),
        Value::Str(text) => Ok(Some(text.as_bytes().to_vec())),
        other => Err(format!(
            "cannot bind a {} value; pass a str, int, float, bool, or nil",
            other.type_name()
        )),
    }
}
