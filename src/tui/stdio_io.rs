//! JSON-RPC stdio transport primitives for scripts.

use crate::value::Value;

use super::val;

mod decode;
mod header;
mod read;
mod write;

pub(super) fn read(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("stdio_read expects no args".into());
    }
    Ok(val::result(read::value()))
}

pub(super) fn write(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("stdio_write expects value".into());
    }
    Ok(val::result(write::value(&args[0])))
}
