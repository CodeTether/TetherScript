//! Bounded file reads and writes.

use std::fs::File;
use std::io::{Read, Write};

use crate::value::Value;

use super::super::args;

const MAX_READ_BYTES: usize = 16 * 1024 * 1024;

pub(super) fn read(file: &mut File, limit: &Value) -> Result<Value, String> {
    let limit = args::usize(limit, "file.read limit")?;
    if limit > MAX_READ_BYTES {
        return Err(format!(
            "file.read limit {limit} exceeds maximum {MAX_READ_BYTES}"
        ));
    }
    let mut bytes = vec![0; limit];
    let count = file
        .read(&mut bytes)
        .map_err(|error| format!("file.read: {error}"))?;
    bytes.truncate(count);
    Ok(Value::Bytes(std::rc::Rc::new(std::cell::RefCell::new(
        bytes,
    ))))
}

pub(super) fn write(file: &mut File, body: &Value) -> Result<Value, String> {
    let bytes = args::bytes(body, "file.write body")?;
    let count = file
        .write(&bytes)
        .map_err(|error| format!("file.write: {error}"))?;
    Ok(Value::Int(count as i64))
}

pub(super) fn flush(file: &mut File) -> Result<(), String> {
    file.flush().map_err(|error| format!("file.flush: {error}"))
}
