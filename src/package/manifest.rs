//! Package manifest model and file loading.

use std::path::{Path, PathBuf};

/// Name of the local tetherscript package manifest.
pub const MANIFEST_NAME: &str = "tetherscript.json";

/// Validated local package metadata.
///
/// # Examples
///
/// ```no_run
/// use tetherscript::package::Manifest;
/// let manifest = Manifest::load(std::path::Path::new("tetherscript.json"))?;
/// assert!(!manifest.name().is_empty());
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone)]
pub struct Manifest {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) entry: PathBuf,
}

impl Manifest {
    /// Load and validate a JSON manifest.
    ///
    /// # Errors
    ///
    /// Returns a path-qualified I/O, JSON, or schema error.
    pub fn load(path: &Path) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let value = crate::json::parse_str(&source)
            .map_err(|error| format!("{}: invalid JSON: {error}", path.display()))?;
        super::decode::manifest(&value).map_err(|error| format!("{}: {error}", path.display()))
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn entry(&self) -> &Path {
        &self.entry
    }
}
