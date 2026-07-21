//! Secure resolution of file-relative module paths.

use std::path::{Component, Path, PathBuf};

pub(super) fn entry(path: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(path)
        .map_err(|error| format!("can't resolve entry {}: {error}", path.display()))
}

pub(super) fn package_root(entry: &Path) -> Result<PathBuf, String> {
    let parent = entry.parent().ok_or("entry file has no parent directory")?;
    for directory in parent.ancestors() {
        if directory.join(crate::package::MANIFEST_NAME).is_file() {
            return Ok(directory.to_owned());
        }
    }
    Ok(parent.to_owned())
}

pub(super) fn import(importer: &Path, root: &Path, requested: &str) -> Result<PathBuf, String> {
    let relative = Path::new(requested);
    if relative.extension().and_then(|value| value.to_str()) != Some("tether") {
        return Err(format!(
            "module import `{requested}` must name a .tether file"
        ));
    }
    if relative.components().any(forbidden_component) {
        return Err(format!("module import `{requested}` must be file-relative"));
    }
    let parent = importer
        .parent()
        .ok_or_else(|| format!("module {} has no parent directory", importer.display()))?;
    let candidate = std::fs::canonicalize(parent.join(relative))
        .map_err(|error| format!("can't resolve import `{requested}`: {error}"))?;
    if !candidate.starts_with(root) {
        return Err(format!("module import `{requested}` escapes package root"));
    }
    Ok(candidate)
}

fn forbidden_component(component: Component<'_>) -> bool {
    matches!(component, Component::RootDir | Component::Prefix(_))
}
