//! Reload marker handling for embedded launchers.

use std::fs;
use std::path::{Path, PathBuf};

const MARKER: &str = ".tetherscript/reload";

pub(crate) fn clear(path: Option<&String>) {
    if marked(path) {
        let _ = fs::remove_file(MARKER);
    }
}

pub(crate) fn take(path: Option<&String>) -> bool {
    let found = marked(path);
    if found {
        let _ = fs::remove_file(MARKER);
    }
    found
}

fn marked(path: Option<&String>) -> bool {
    fs::read_to_string(MARKER)
        .map(|text| matches_path(text.trim(), path))
        .unwrap_or(false)
}

fn matches_path(marker: &str, path: Option<&String>) -> bool {
    path.map(|p| same(marker, p))
        .unwrap_or(marker == "embedded")
}

fn same(left: &str, right: &str) -> bool {
    left == right || norm(left) == norm(right)
}

fn norm(path: &str) -> PathBuf {
    let path = Path::new(path);
    path.canonicalize()
        .unwrap_or_else(|_| path.components().collect())
}
