//! Last-known-good recovery for embedded sidecar generations.

use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn restore(
    path: Option<&str>,
    good: &str,
    failed: &str,
    error: &str,
) -> Result<(), String> {
    let path = path.ok_or_else(|| format!("reload failed without a sidecar: {error}"))?;
    let rejected = artifact_path(path, "failed");
    fs::write(&rejected, failed)
        .map_err(|e| format!("could not preserve {}: {e}", rejected.display()))?;
    fs::write(path, good).map_err(|e| format!("could not restore {path}: {e}"))?;
    eprintln!(
        "tetherscript: restored {path} after reload failure: {error}; rejected source: {}",
        rejected.display()
    );
    Ok(())
}

fn artifact_path(path: &str, suffix: &str) -> PathBuf {
    let mut value = Path::new(path).as_os_str().to_owned();
    value.push(format!(".{suffix}"));
    PathBuf::from(value)
}
