//! Member-position completion: methods, resource factories, module exports.
//!
//! Three cases follow the `.`:
//!
//! - `resource.` — the owned-resource constructors.
//! - `alias.` where `alias` is an import — that module's `export`ed names,
//!   resolved and read through [`crate::lsp_capabilities::module`]. This is the
//!   case the VSCode client implemented in `module-symbol-completions.js`.
//! - anything else — value and resource methods, since a dynamically typed
//!   receiver gives no better information at the cursor.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::completion_member::factories;
//! use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
//!
//! let items = factories();
//! assert!(items.iter().any(|item| {
//!     field(item, "label").as_deref_str() == Some("timer")
//! }));
//! ```

use crate::lsp_capabilities::completion_item::{callable, described};
use crate::lsp_capabilities::docs::Docs;
use crate::lsp_capabilities::rank::{Tier, sort_text};
use crate::lsp_capabilities::request::Cursor;
use crate::lsp_capabilities::symbol::SymbolKind;
use crate::lsp_capabilities::{methods, methods_factory, module, symbols, uri};
use crate::value::Value;

/// Completion items offered directly after `resource.`.
///
/// # Returns
///
/// One callable item per owned-resource constructor.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_member::factories;
/// assert_eq!(factories().len(), 9);
/// ```
pub fn factories() -> Vec<Value> {
    let sort = sort_text(Tier::Builtin, 0);
    methods_factory::FACTORIES
        .iter()
        .map(|entry| callable(entry.0, entry.1, entry.2, 3, &sort))
        .collect()
}

/// Completion items offered after `owner.`.
///
/// # Arguments
///
/// * `cursor` — Resolved request cursor.
/// * `docs` — Open-document store, used to read the imported module.
/// * `owner` — Identifier before the `.`.
///
/// # Returns
///
/// Module exports when `owner` is an import alias that resolves, otherwise the
/// method catalog. Falling back to methods rather than to nothing matters
/// because a broken import should not also break method completion.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::methods;
/// // With no matching import alias, the method catalog is what is offered.
/// assert!(methods::lookup("push").is_some());
/// ```
pub fn members(cursor: &Cursor<'_>, docs: &Docs<'_>, owner: &str) -> Vec<Value> {
    exports(cursor, docs, owner).unwrap_or_else(method_items)
}

fn method_items() -> Vec<Value> {
    let sort = sort_text(Tier::Builtin, 0);
    methods::iter()
        .map(|entry| callable(entry.0, entry.1, entry.2, 2, &sort))
        .collect()
}

fn exports(cursor: &Cursor<'_>, docs: &Docs<'_>, owner: &str) -> Option<Vec<Value>> {
    let alias = symbols::collect(cursor.text)
        .into_iter()
        .find(|symbol| symbol.kind == SymbolKind::Module && symbol.name == owner)?;
    let path = module::resolve(&uri::to_path(&cursor.uri)?, &alias.detail)?;
    let text = docs.module_text(&path)?;
    let sort = sort_text(Tier::Module, 0);
    Some(
        module::exported_names(&text)
            .into_iter()
            .map(|symbol| export_item(&symbol, &alias.detail, &sort))
            .collect(),
    )
}

fn export_item(symbol: &crate::lsp_capabilities::symbol::Symbol, path: &str, sort: &str) -> Value {
    let doc = format!("Exported by `{path}`.");
    if symbol.kind == SymbolKind::Function {
        callable(&symbol.name, &symbol.signature, &doc, 3, sort)
    } else {
        let kind = symbol.kind.completion_kind();
        described(&symbol.name, &symbol.signature, &doc, kind, sort)
    }
}
