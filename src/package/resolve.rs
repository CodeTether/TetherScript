//! Resolve explicit files and package directories to executable entries.

use std::path::{Path, PathBuf};

/// A concrete source entry and its package root, when package-backed.
#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    entry: PathBuf,
    root: Option<PathBuf>,
}

impl ResolvedTarget {
    pub fn entry(&self) -> &Path {
        &self.entry
    }
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }
}

/// Resolve a source file, package directory, or nearest package.
///
/// # Arguments
///
/// * `explicit` — Optional source file or package directory.
/// * `cwd` — Base for relative targets and package discovery.
///
/// # Errors
///
/// Returns an error for missing targets, invalid manifests, or unsafe entries.
pub fn resolve_target(explicit: Option<&Path>, cwd: &Path) -> Result<ResolvedTarget, String> {
    let target = explicit.map(|path| absolute(path, cwd));
    if let Some(path) = target.as_deref().filter(|path| path.is_file()) {
        let entry = std::fs::canonicalize(path)
            .map_err(|error| format!("can't resolve {}: {error}", path.display()))?;
        return Ok(ResolvedTarget { entry, root: None });
    }
    let manifest_path = match target {
        Some(path) if path.is_dir() => path.join(super::MANIFEST_NAME),
        Some(path) => return Err(format!("target {} does not exist", path.display())),
        None => super::discover::manifest(cwd)?,
    };
    let manifest = super::Manifest::load(&manifest_path)?;
    let root = manifest_path
        .parent()
        .ok_or("manifest has no package root")?;
    let root = std::fs::canonicalize(root)
        .map_err(|error| format!("can't resolve package root: {error}"))?;
    let entry = std::fs::canonicalize(root.join(manifest.entry()))
        .map_err(|error| format!("package entry {}: {error}", manifest.entry().display()))?;
    if !entry.starts_with(&root) {
        return Err("package entry escapes package root".into());
    }
    Ok(ResolvedTarget {
        entry,
        root: Some(root),
    })
}

fn absolute(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        cwd.join(path)
    }
}
