//! File opening modes.

use std::fs::{File, OpenOptions};
use std::path::Path;

pub(super) fn file(path: &Path, mode: &str) -> Result<File, String> {
    let mut options = OpenOptions::new();
    match mode {
        "read" => {
            options.read(true);
        }
        "write" => {
            options.write(true).create(true).truncate(true);
        }
        "append" => {
            options.append(true).create(true);
        }
        "read_write" => {
            options.read(true).write(true).create(true);
        }
        other => {
            return Err(format!(
                "resource.file: unknown mode `{other}`; expected read, write, append, or read_write"
            ));
        }
    }
    options
        .open(path)
        .map_err(|error| format!("resource.file `{}`: {error}", path.display()))
}
