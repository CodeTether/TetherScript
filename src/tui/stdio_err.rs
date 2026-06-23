//! Raw stderr writes for TUI diagnostics beside stdio JSON.

use std::io::{self, Write};

use crate::value::Value;

pub(super) fn write(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("stdio_write_err expects text".into());
    }
    let Value::Str(text) = &args[0] else {
        return Err(format!(
            "stdio_write_err expects str, got {}",
            args[0].type_name()
        ));
    };
    io::stderr()
        .lock()
        .write_all(text.as_bytes())
        .map_err(|error| format!("stdio_write_err: {error}"))?;
    Ok(Value::Nil)
}
