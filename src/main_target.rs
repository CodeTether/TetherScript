//! Package-aware CLI target resolution.

use std::path::Path;

pub(crate) fn resolve(explicit: Option<&str>, command: &str) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|error| {
        eprintln!("tetherscript {command}: can't read current directory: {error}");
        std::process::exit(1);
    });
    let explicit = explicit.map(Path::new);
    match crate::package::resolve_target(explicit, &cwd) {
        Ok(target) => target.entry().to_string_lossy().into_owned(),
        Err(error) => {
            eprintln!("tetherscript {command}: {error}");
            std::process::exit(1);
        }
    }
}
