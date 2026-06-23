//! Stdio JSON transport primitives for protocol scripts.

use std::io::{self, BufRead, Write};

use crate::json;
use crate::value::Value;

use super::val;

pub(super) fn read(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("stdio_read expects no args".into());
    }
    Ok(val::result(read_inner()))
}

pub(super) fn write(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("stdio_write expects value".into());
    }
    Ok(val::result(write_inner(&args[0])))
}

fn read_inner() -> Result<Value, String> {
    let mut line = String::new();
    let bytes = io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|error| format!("stdio_read: {error}"))?;
    if bytes == 0 {
        return Ok(Value::Nil);
    }
    let text = line
        .trim_end_matches(['\r', '\n'])
        .trim_start_matches('\u{feff}');
    json::parse_str(text).map_err(|error| format!("stdio_read: {error}"))
}

fn write_inner(value: &Value) -> Result<Value, String> {
    let text = json::encode_to_string(value).map_err(|error| format!("stdio_write: {error}"))?;
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())
        .map_err(|error| format!("stdio_write: {error}"))?;
    out.write_all(b"\n")
        .map_err(|error| format!("stdio_write: {error}"))?;
    out.flush()
        .map_err(|error| format!("stdio_write: {error}"))?;
    Ok(Value::Nil)
}
