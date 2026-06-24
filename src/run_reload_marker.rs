//! Reload marker file handling.

use std::fs;
use std::path::{Path, PathBuf};

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
        .map(|text| same_path(text.trim(), path))
        .unwrap_or(false)
}

fn same_path(left: &str, right: &str) -> bool {
    left == right || normalize(left) == normalize(right)
}

fn normalize(path: &str) -> PathBuf {
    let path = Path::new(path);
    path.canonicalize()
        .unwrap_or_else(|_| path.components().collect())
}

fn marker() -> PathBuf {
    PathBuf::from(DIR).join(FILE)
}

#[cfg(test)]
mod tests {
    use super::same_path;

    #[test]
    fn same_path_accepts_dot_prefixed_relative_form() {
        let plain = format!("examples{}agent_tui.tether", std::path::MAIN_SEPARATOR);
        let dotted = format!(".{}{}", std::path::MAIN_SEPARATOR, plain);
        assert!(same_path(&dotted, &plain));
    }
}
