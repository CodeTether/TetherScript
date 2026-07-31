//! Script-visible built-in constructors for the HMAC/hex group.
//!
//! Split out of `hmac.rs` purely to respect the 50-line file limit; the
//! registration list stays there so the group has one obvious entry point.

use std::rc::Rc;

use super::super::super::pure_native;
use super::hmac_digest::hmac_sha256;
use super::hmac_hex_codec::{decode_hex, encode_hex};
use crate::system::result_value;
use crate::value::Value;

/// `hmac_sha256_hex(key, message)` -> lowercase hex str.
pub(super) fn hmac_builtin() -> Value {
    pure_native("hmac_sha256_hex", Some(2), |args| {
        let key = str_arg(&args[0], "hmac_sha256_hex: key")?;
        let message = str_arg(&args[1], "hmac_sha256_hex: message")?;
        let mac = hmac_sha256(key.as_bytes(), message.as_bytes());
        Ok(Value::Str(Rc::new(encode_hex(&mac))))
    })
}

/// `hex_encode(input)` -> lowercase hex str.
pub(super) fn hex_encode_builtin() -> Value {
    pure_native("hex_encode", Some(1), |args| {
        let input = str_arg(&args[0], "hex_encode: input")?;
        Ok(Value::Str(Rc::new(encode_hex(input.as_bytes()))))
    })
}

/// `hex_decode(hex)` -> `Result` of the decoded str.
pub(super) fn hex_decode_builtin() -> Value {
    pure_native("hex_decode", Some(1), |args| {
        let input = str_arg(&args[0], "hex_decode: hex")?;
        Ok(result_value(decode_hex(&input).and_then(as_utf8_value)))
    })
}

/// Hex can encode arbitrary bytes, but tetherscript strings are UTF-8.
fn as_utf8_value(bytes: Vec<u8>) -> Result<Value, String> {
    String::from_utf8(bytes)
        .map(|text| Value::Str(Rc::new(text)))
        .map_err(|_| "hex_decode: decoded bytes are not valid UTF-8".to_string())
}

pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
