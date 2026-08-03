//! Cross-module go-to-definition.
//!
//! This is the part users actually miss: everything else can be answered from the
//! buffer the client already sent, but jumping from `math.add` to the `add` in
//! `math.tether` requires resolving a module path and reading a second file.
//!
//! Resolution goes through [`crate::lsp_capabilities::module::resolve`], so the
//! server agrees with [`crate::modules`] about which file an import names, and it
//! points at the declaration *inside* that file rather than at its `export` line —
//! the export line is not where a reader wants to land. Jumping to the alias
//! itself opens the module at its first line, matching the client's
//! `module-navigation.js`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::module::exported_names;
//! assert_eq!(exported_names("export add\nfn add(a) { a }")[0].name, "add");
//! ```

use crate::lsp_capabilities::definition::Target;
use crate::lsp_capabilities::definition_target::nearest;
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::symbol::SymbolKind;
use crate::lsp_capabilities::{module, uri};

/// Jump to the start of the file an import alias names.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store.
/// * `name` — Identifier under the cursor, expected to be an import alias.
///
/// # Returns
///
/// `Some(Target)` at line 0 of the imported file, or `None` when `name` is not an
/// alias or the target cannot be resolved or read.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbol::SymbolKind;
/// assert_eq!(SymbolKind::Module.completion_kind(), 9);
/// ```
pub fn alias_file(cursor: &Cursor<'_>, docs: &Docs<'_>, name: &str) -> Option<Target> {
    let alias = nearest(cursor, name).filter(|s| s.kind == SymbolKind::Module)?;
    let path = module::resolve(&uri::to_path(&cursor.uri)?, &alias.detail)?;
    Some(Target {
        uri: uri::from_path(&path),
        text: docs.module_text(&path)?,
        start: 0,
        end: 0,
    })
}

/// Jump to an exported declaration reached through `qualifier.name`.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store.
/// * `qualifier` — Identifier before the `.`, expected to be an import alias.
/// * `name` — Member name after the `.`.
///
/// # Returns
///
/// `Some(Target)` at the declaration inside the imported file, or `None` when the
/// alias, the file, or the export cannot be resolved. Only `export`ed names
/// resolve, matching the language: a non-exported `fn` is unreachable through the
/// namespace, so jumping to it would land the user on code they cannot call.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::module::exported_names;
/// // `hidden` is declared but not exported, so it is not a jump target.
/// let names = exported_names("export shown\nfn shown() { }\nfn hidden() { }");
/// assert!(names.iter().all(|symbol| symbol.name != "hidden"));
/// ```
pub fn imported(
    cursor: &Cursor<'_>,
    docs: &Docs<'_>,
    qualifier: &str,
    name: &str,
) -> Option<Target> {
    let alias = nearest(cursor, qualifier).filter(|s| s.kind == SymbolKind::Module)?;
    let path = module::resolve(&uri::to_path(&cursor.uri)?, &alias.detail)?;
    let text = docs.module_text(&path)?;
    let found = module::exported_names(&text)
        .into_iter()
        .find(|symbol| symbol.name == name)?;
    Some(Target {
        uri: uri::from_path(&path),
        start: found.offset,
        end: found.offset + found.name.len(),
        text,
    })
}
