//! Resolving an import alias to another file's exported symbols.
//!
//! This is the part of the task that requires real work: everything else can be
//! answered from the buffer the client already sent, but `math.add` → the `add`
//! in `math.tether` requires resolving a path and reading a second file.
//!
//! Resolution follows the same rules as [`crate::modules`] so the server cannot
//! disagree with the compiler about which file an import names:
//!
//! - the request must be relative and must end in `.tether`;
//! - it resolves against the *importing file's* directory;
//! - it must canonicalize inside the nearest package root (the closest ancestor
//!   containing `tetherscript.json`, else the entry's directory).
//!
//! Only `export`ed names are offered, matching the language: a `fn` that is not
//! exported is not reachable through the namespace, so jumping to it would take
//! the user somewhere they cannot actually call.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::module::exported_names;
//!
//! let names = exported_names("export add\nfn add(a, b) { a }\nfn hidden() { }");
//! assert_eq!(names.len(), 1);
//! assert_eq!(names[0].name, "add");
//! ```

use std::path::{Component, Path, PathBuf};

use crate::lsp_capabilities::symbol::Symbol;
use crate::lsp_capabilities::symbols::collect;

/// Resolve an import request against the importing file.
///
/// # Arguments
///
/// * `importer` — Path of the file containing the `import` declaration.
/// * `request` — Path exactly as written in the import, e.g. `"./math.tether"`.
///
/// # Returns
///
/// `Some(canonical_path)` when the target exists and is inside the package root,
/// `None` when the request is absolute, is not a `.tether` file, does not exist,
/// or escapes the root.
///
/// # Errors
///
/// Infallible; every rejection is reported as `None` so a bad import degrades to
/// "no definition" instead of an editor-visible error.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::module::resolve;
/// use std::path::Path;
/// assert!(resolve(Path::new("/a/main.tether"), "/etc/passwd").is_none());
/// assert!(resolve(Path::new("/a/main.tether"), "./m.txt").is_none());
/// ```
pub fn resolve(importer: &Path, request: &str) -> Option<PathBuf> {
    let relative = Path::new(request);
    if relative.extension().and_then(|value| value.to_str()) != Some("tether") {
        return None;
    }
    if relative.components().any(forbidden) {
        return None;
    }
    let parent = importer.parent()?;
    let root = package_root(importer)?;
    let candidate = std::fs::canonicalize(parent.join(relative)).ok()?;
    if candidate.starts_with(&root) {
        Some(candidate)
    } else {
        None
    }
}

fn forbidden(component: Component<'_>) -> bool {
    matches!(component, Component::RootDir | Component::Prefix(_))
}

/// Nearest package root above `entry`, mirroring `crate::modules::path`.
///
/// # Arguments
///
/// * `entry` — Path of the importing file.
///
/// # Returns
///
/// The closest ancestor directory containing `tetherscript.json`, or the file's
/// own directory when no manifest exists.
///
/// # Errors
///
/// Infallible; a path with no parent yields `None`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::module::package_root;
/// use std::path::Path;
/// assert!(package_root(Path::new("/a/b/main.tether")).is_some());
/// assert!(package_root(Path::new("/")).is_none());
/// ```
pub fn package_root(entry: &Path) -> Option<PathBuf> {
    let parent = entry.parent()?;
    for directory in parent.ancestors() {
        if directory.join(crate::package::MANIFEST_NAME).is_file() {
            return Some(directory.to_owned());
        }
    }
    Some(parent.to_owned())
}

/// Symbols a module source text exposes via `export`.
///
/// # Arguments
///
/// * `text` — Full source of the imported module.
///
/// # Returns
///
/// The declarations named by `export`, each with its own byte offset so a
/// definition reply can point at the declaration rather than the export line.
///
/// # Errors
///
/// Infallible; a module that does not lex yields an empty vector.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::module::exported_names;
/// assert!(exported_names("fn add(a) { a }").is_empty());
/// assert_eq!(exported_names("export k\nlet k = 1")[0].signature, "let k");
/// ```
pub fn exported_names(text: &str) -> Vec<Symbol> {
    let exports = export_list(text);
    collect(text)
        .into_iter()
        // `exports` holds owned Strings, so this compares by str rather than allocating a
        // String per symbol just to satisfy `contains`.
        .filter(|symbol| exports.iter().any(|name| name == &symbol.name))
        .collect()
}

fn export_list(text: &str) -> Vec<String> {
    match crate::lexer::Lexer::new(text).tokenize() {
        Ok(tokens) => crate::parser::Parser::new(tokens)
            .parse_program()
            .map(|program| program.exports)
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}
