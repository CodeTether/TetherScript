//! Platform-specific launcher permission handling.

use std::path::Path;

#[cfg(unix)]
pub(crate) fn make_executable(path: &Path) -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
pub(crate) fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}
