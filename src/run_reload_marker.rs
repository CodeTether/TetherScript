//! Reload marker file handling.

use std::fs;
use std::path::PathBuf;

const DIR: &str = ".tetherscript";
const FILE: &str = "reload";

pub(super) fn clear(path: &str) {
    if marked(path) {
        let _ = fs::remove_file(marker());
    }
}

pub(super) fn take(path: &str) -> bool {
    let found = marked(path);
    if found {
        let _ = fs::remove_file(marker());
    }
    found
}

fn marked(path: &str) -> bool {
    fs::read_to_string(marker())
        .map(|text| text.trim() == path)
        .unwrap_or(false)
}

fn marker() -> PathBuf {
    PathBuf::from(DIR).join(FILE)
}
