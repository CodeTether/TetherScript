//! Validation rules for package metadata.

use std::path::{Component, Path};

pub(super) fn metadata(name: &str, version: &str, entry: &Path) -> Result<(), String> {
    if !name.chars().all(package_name_char) {
        return Err("package.name may contain only letters, digits, `_`, and `-`".into());
    }
    if version.trim().is_empty() {
        return Err("package.version must not be empty".into());
    }
    if entry.extension().and_then(|value| value.to_str()) != Some("tether") {
        return Err("package.entry must name a .tether file".into());
    }
    if entry.components().any(forbidden_component) {
        return Err("package.entry must stay inside the package root".into());
    }
    Ok(())
}

fn package_name_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-')
}

fn forbidden_component(component: Component<'_>) -> bool {
    matches!(
        component,
        Component::ParentDir | Component::RootDir | Component::Prefix(_)
    )
}
