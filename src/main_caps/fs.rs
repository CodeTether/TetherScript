//! Filesystem capability selection for CLI grants.

pub(super) fn root(explicit: &Option<String>, full: bool) -> Result<Option<String>, String> {
    if let Some(root) = explicit {
        return Ok(Some(root.clone()));
    }
    if !full {
        return Ok(None);
    }
    std::env::current_dir()
        .map(|path| Some(path.display().to_string()))
        .map_err(|error| format!("access-mode full: cannot read cwd: {error}"))
}
