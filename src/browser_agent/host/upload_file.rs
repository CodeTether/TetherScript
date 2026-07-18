//! Filesystem-backed upload payload construction.

use std::path::Path;

use crate::browser_agent::FilePayload;

pub(super) fn read(path: &str) -> Result<FilePayload, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("browser.upload: cannot read `{path}`: {error}"))?;
    let name = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("browser.upload: `{path}` has no UTF-8 file name"))?;
    Ok(FilePayload::new(name, mime(name), bytes))
}

fn mime(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("txt" | "tether") => "text/plain",
        Some("html" | "htm") => "text/html",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}
