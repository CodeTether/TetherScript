//! Safe local package scaffolding.

use std::path::{Path, PathBuf};

use super::Manifest;

/// Create a package manifest and starter entry file.
///
/// # Arguments
///
/// * `root` — Destination package directory.
///
/// # Errors
///
/// Returns an error rather than overwriting an existing manifest or entry.
pub fn init(root: &Path) -> Result<Manifest, String> {
    let name = package_name(root)?;
    let entry = PathBuf::from("src/main.tether");
    super::validate::metadata(&name, "0.1.0", &entry)?;
    let manifest_path = root.join(super::MANIFEST_NAME);
    let entry_path = root.join(&entry);
    for path in [&manifest_path, &entry_path] {
        if path.exists() {
            return Err(format!("refusing to overwrite {}", path.display()));
        }
    }
    std::fs::create_dir_all(entry_path.parent().expect("entry has parent"))
        .map_err(|error| format!("can't create {}: {error}", root.display()))?;
    std::fs::write(&manifest_path, super::template::manifest(&name))
        .map_err(|error| format!("can't write {}: {error}", manifest_path.display()))?;
    std::fs::write(&entry_path, super::template::MAIN)
        .map_err(|error| format!("can't write {}: {error}", entry_path.display()))?;
    Ok(Manifest {
        name,
        version: "0.1.0".into(),
        entry,
    })
}

fn package_name(root: &Path) -> Result<String, String> {
    let resolved = if root.file_name().is_some() {
        root.to_owned()
    } else {
        std::env::current_dir().map_err(|error| format!("can't read current directory: {error}"))?
    };
    resolved
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or_else(|| format!("can't derive package name from {}", root.display()))
}
