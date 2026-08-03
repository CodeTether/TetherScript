//! Built-in bodies for the session half: key, encode, decode, new id, rotate.
//!
//! Each function takes the raw argument slice and returns a plain value, so the
//! transport owner can call the same logic without going through the interpreter.

use std::cell::RefCell;
use std::rc::Rc;

use super::sessionstore_args::{map_arg, str_arg};
use super::sessionstore_decode::decode as decode_text;
use super::sessionstore_encode::encode as encode_map;
use super::sessionstore_id::generate;
use super::sessionstore_key::derive;
use super::sessionstore_rotate::rotate as rotate_id;
use crate::value::Value;

/// `session_store_key(prefix, session_id)` — the namespaced key.
///
/// # Arguments
///
/// * `args` — `[prefix: str, session_id: str]`.
///
/// # Returns
///
/// The key string.
///
/// # Errors
///
/// Returns a named error on a non-str argument or an id that fails validation.
pub(super) fn key(args: &[Value]) -> Result<Value, String> {
    let prefix = str_arg(&args[0], "session_store_key: prefix")?;
    let id = str_arg(&args[1], "session_store_key: session_id")?;
    Ok(Value::Str(Rc::new(derive(&prefix, &id)?)))
}

/// `session_store_encode(payload_map)` — the compact serialized string.
///
/// # Arguments
///
/// * `args` — `[payload: map]`.
///
/// # Returns
///
/// The serialized text, empty for an empty map.
///
/// # Errors
///
/// Returns a named error for a non-map argument, an empty key, or a nested value.
pub(super) fn encode(args: &[Value]) -> Result<Value, String> {
    let label = "session_store_encode";
    let payload = map_arg(&args[0], &format!("{label}: payload_map"))?;
    Ok(Value::Str(Rc::new(encode_map(label, &payload)?)))
}

/// `session_store_decode(text)` — the payload map.
///
/// # Arguments
///
/// * `args` — `[text: str]`.
///
/// # Returns
///
/// The reconstructed map.
///
/// # Errors
///
/// Returns a named error for malformed text; see [`super::sessionstore_decode`].
pub(super) fn decode(args: &[Value]) -> Result<Value, String> {
    let label = "session_store_decode";
    let text = str_arg(&args[0], &format!("{label}: text"))?;
    let map = decode_text(label, &text)?;
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

/// `session_store_new_id()` — a fresh 256-bit id as 64 hex characters.
///
/// # Returns
///
/// The id. Not a `Result`: there is no argument to reject and no failure mode.
pub(super) fn new_id() -> Value {
    Value::Str(Rc::new(generate()))
}

/// `session_rotate_id(old_id)` — a different id, defeating session fixation.
///
/// # Arguments
///
/// * `args` — `[old_id: str]`.
///
/// # Returns
///
/// A fresh id, never equal to `old_id`.
///
/// # Errors
///
/// Returns a named error when `old_id` is not a str or fails validation.
pub(super) fn rotate(args: &[Value]) -> Result<Value, String> {
    let old = str_arg(&args[0], "session_rotate_id: old_id")?;
    Ok(Value::Str(Rc::new(rotate_id(&old)?)))
}
