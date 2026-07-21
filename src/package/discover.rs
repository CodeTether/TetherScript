//! Nearest-ancestor package discovery.

use std::path::{Path, PathBuf};

pub(super) fn manifest(start: &Path) -> Result<PathBuf, String> {
    let start = std::fs::canonicalize(start)
        .map_err(|error| format!("can't resolve {}: {error}", start.display()))?;
    for directory in start.ancestors() {
        let candidate = directory.join(super::MANIFEST_NAME);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "no {} found from {} or its ancestors",
        super::MANIFEST_NAME,
        start.display()
    ))
}
