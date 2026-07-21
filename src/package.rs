//! Local package manifests and project scaffolding.
//!
//! A package is rooted by `tetherscript.json` and names one `.tether` entry file.
//! This foundation is intentionally local-only; remote registries and lockfiles
//! are separate concerns.

mod decode;
mod decode_value;
mod discover;
mod init;
mod manifest;
mod resolve;
mod template;
mod validate;

pub use init::init;
pub use manifest::{Manifest, MANIFEST_NAME};
pub use resolve::{resolve_target, ResolvedTarget};

/// Resolve the current package entry from `cwd`.
///
/// # Arguments
///
/// * `cwd` — Directory from which manifest discovery starts.
///
/// # Returns
///
/// The canonical package entry and discovered package root.
///
/// # Errors
///
/// Returns an error when no manifest exists or its entry is invalid.
///
/// # Examples
///
/// ```no_run
/// let target = tetherscript::package::current(std::path::Path::new("."))?;
/// println!("entry: {}", target.entry().display());
/// # Ok::<(), String>(())
/// ```
pub fn current(cwd: &std::path::Path) -> Result<ResolvedTarget, String> {
    resolve_target(None, cwd)
}
