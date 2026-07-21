//! File-based tetherscript module loading.
//!
//! Imports are explicit, file-relative, and lowered to isolated namespace maps.
//! Use `import "./math.tether" as math` and declare public bindings with
//! `export add` in the imported file.

mod load;
mod load_body;
mod lower;
mod names;
mod parse;
mod path;

use std::path::Path;

/// Load an entry file and all of its file-relative imports.
///
/// Imports are resolved relative to their declaring file and are constrained to
/// the nearest package root. When no manifest exists, the entry's directory is
/// used as the root.
///
/// # Arguments
///
/// * `path` — Existing `.tether` entry file to parse and link.
///
/// # Returns
///
/// A single linked program suitable for ownership analysis or execution.
///
/// # Errors
///
/// Returns a path-qualified error for unreadable, invalid, or cyclic modules.
///
/// # Examples
///
/// ```no_run
/// let program = tetherscript::modules::load_program(
///     std::path::Path::new("examples/modules.tether"),
/// )?;
/// assert!(!program.stmts.is_empty());
/// # Ok::<(), String>(())
/// ```
pub fn load_program(path: &Path) -> Result<crate::ast::Program, String> {
    load::ModuleLoader::new().load_entry(path)
}
