//! Blocking key input helper for future TUI built-ins.

use std::io::{self, Read};

use crate::value::Value;

use super::{key_parse, key_value, val};

/// Read stdin bytes and return a parsed key event Result value.
pub(super) fn read_key(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("tui_read_key expects no arguments".into());
    }
    Ok(val::result(read_bytes().and_then(|bytes| {
        key_parse::parse(&bytes).map(key_value::to_value)
    })))
}

fn read_bytes() -> Result<Vec<u8>, String> {
    let mut stdin = io::stdin();
    let mut bytes = [0u8; 8];
    let count = stdin
        .read(&mut bytes)
        .map_err(|error| format!("tui_read_key: read failed: {error}"))?;
    if count == 0 {
        return Err("tui_read_key: end of input".into());
    }
    Ok(bytes[..count].to_vec())
}
