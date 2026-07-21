//! Owned file handles.

mod io;
mod open;

use std::fs::File;

use crate::value::Value;

use super::result;

pub(super) struct Handle {
    file: File,
}

impl Handle {
    pub(super) fn open(path: &std::path::Path, mode: &str) -> Result<Self, String> {
        open::file(path, mode).map(|file| Self { file })
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("read", [limit]) => Ok(result::value(io::read(&mut self.file, limit))),
            ("write", [body]) => Ok(result::value(io::write(&mut self.file, body))),
            ("flush", []) => Ok(result::nil(io::flush(&mut self.file))),
            _ => Err(format!(
                "file: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }
}
