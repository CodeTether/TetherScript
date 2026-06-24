//! Write one framed JSON message to stdout.

use std::io::{self, Write};

use crate::{json, value::Value};

pub(super) fn value(value: &Value) -> Result<Value, String> {
    let text = json::encode_to_string(value).map_err(|error| format!("stdio_write: {error}"))?;
    let mut out = io::stdout().lock();
    write!(out, "Content-Length: {}\r\n\r\n", text.len())
        .map_err(|error| format!("stdio_write: {error}"))?;
    out.write_all(text.as_bytes())
        .map_err(|error| format!("stdio_write: {error}"))?;
    out.flush()
        .map_err(|error| format!("stdio_write: {error}"))?;
    Ok(Value::Nil)
}
