//! Environment registration for the base32 built-ins.
//!
//! Separated from `base32.rs` so the owning module only declares its submodules,
//! keeping every file inside the 50-line limit.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::{base32_decode, base32_encode};
use crate::system::result_value;
use crate::value::{Env, Value};

/// Define `base32_encode`, `base32_encode_nopad`, and `base32_decode`.
///
/// # Arguments
///
/// * `env` — Environment to populate.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define("base32_encode", encode(), false);
    bindings.define("base32_encode_nopad", encode_nopad(), false);
    bindings.define("base32_decode", decode(), false);
}

fn encode() -> Value {
    pure_native("base32_encode", Some(1), |args| {
        let input = str_arg(&args[0], "base32_encode: input")?;
        Ok(Value::Str(Rc::new(base32_encode::encode(input.as_bytes()))))
    })
}

fn encode_nopad() -> Value {
    pure_native("base32_encode_nopad", Some(1), |args| {
        let input = str_arg(&args[0], "base32_encode_nopad: input")?;
        Ok(Value::Str(Rc::new(base32_encode::encode_nopad(
            input.as_bytes(),
        ))))
    })
}

/// Base32 can carry arbitrary bytes, but tetherscript strings are UTF-8, so a
/// payload that is not valid UTF-8 is reported rather than lossily replaced.
fn decode() -> Value {
    pure_native("base32_decode", Some(1), |args| {
        let input = str_arg(&args[0], "base32_decode: text")?;
        Ok(result_value(base32_decode::decode(&input).and_then(
            |bytes| {
                String::from_utf8(bytes)
                    .map(|text| Value::Str(Rc::new(text)))
                    .map_err(|_| "base32_decode: decoded bytes are not valid UTF-8".to_string())
            },
        )))
    })
}

/// Require a str argument, naming the built-in and parameter on mismatch.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
