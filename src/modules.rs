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
/// # Errors
///
/// Returns a path-qualified error for unreadable, invalid, or cyclic modules.
pub fn load_program(path: &Path) -> Result<crate::ast::Program, String> {
    load::ModuleLoader::new().load_entry(path)
}
