//! `application/x-www-form-urlencoded` pair handling.
//!
//! Kept separate from the codec and the registration layer: this file owns only
//! the `&`/`=` splitting rules and the map shape handed back to scripts.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::form_codec::{decode, encode};
use crate::value::Value;

/// Parse `&`-separated pairs into a map, percent-decoding both sides.
///
/// A bare name with no `=` yields an empty-string value, matching how browsers
/// submit valueless fields. Empty segments, which arise from `a=1&&b=2` or a
/// trailing `&`, are skipped rather than producing an empty key. When a name
/// repeats, the last occurrence wins, because the script-facing shape is a map.
///
/// # Arguments
///
/// * `input` — A query string or form body, without any leading `?`.
///
/// # Errors
///
/// Propagates the decode error, naming the offending escape, if either side of a
/// pair contains a malformed percent sequence.
pub(crate) fn parse(input: &str) -> Result<Value, String> {
    let mut map = HashMap::new();
    for segment in input.split('&') {
        if segment.is_empty() {
            continue;
        }
        let (raw_name, raw_value) = match segment.split_once('=') {
            Some((name, value)) => (name, value),
            None => (segment, ""),
        };
        let name = decode(raw_name, "form_parse: name")?;
        let value = decode(raw_value, "form_parse: value")?;
        map.insert(name, Value::Str(Rc::new(value)));
    }
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

/// Encode a map back into a `&`-separated form body.
///
/// Keys are emitted in sorted order so the output is deterministic; map iteration
/// order is not stable and an unstable body would break both tests and caching.
///
/// # Arguments
///
/// * `map` — Name/value pairs. Values must be str, int, float, bool, or nil.
///
/// # Errors
///
/// Returns an error naming the key and type when a value cannot be rendered.
pub(crate) fn encode_map(map: &HashMap<String, Value>) -> Result<String, String> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let mut parts = Vec::with_capacity(keys.len());
    for key in keys {
        let rendered = render(&map[key], key)?;
        parts.push(format!("{}={}", encode(key), encode(&rendered)));
    }
    Ok(parts.join("&"))
}

/// Render one value as the text that will be percent-encoded.
fn render(value: &Value, key: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        Value::Int(int) => Ok(int.to_string()),
        Value::Float(float) => Ok(float.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        Value::Nil => Ok(String::new()),
        other => Err(format!(
            "form_encode: value for `{key}` must be str, int, float, bool, or nil, got {}",
            other.type_name()
        )),
    }
}
