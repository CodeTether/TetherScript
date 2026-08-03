//! In-scope symbol completion items.
//!
//! Symbols are filtered by
//! [`crate::lsp_capabilities::symbol::Symbol::visible_at`] before anything else,
//! so a `let` declared inside a different function never appears — offering an
//! out-of-scope name is worse than offering nothing, since accepting it produces
//! code that fails the ownership pass.
//!
//! Within the local tiers, items are ordered by *proximity* to the cursor: the
//! nearest declaration sorts first, because in a long function the closest `let`
//! is nearly always the intended one.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::completion_scope::items;
//! use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
//!
//! let source = "fn f(arg) { let near = 1\n";
//! let all = items(source, source.len(), "");
//! let labels: Vec<String> = all
//!     .iter()
//!     .filter_map(|item| field(item, "label").as_deref_str().map(str::to_string))
//!     .collect();
//! assert!(labels.contains(&"near".to_string()));
//! assert!(labels.contains(&"arg".to_string()));
//! ```

use crate::lsp_capabilities::completion_item::{callable, described};
use crate::lsp_capabilities::rank::{sort_text, Tier};
use crate::lsp_capabilities::symbol::SymbolKind;
use crate::lsp_capabilities::symbols::collect;
use crate::value::Value;

/// Completion items for every symbol in scope at `offset`.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `offset` — Cursor byte offset.
/// * `typed` — Partial word already typed; a prefix match promotes a local to
///   [`Tier::LocalExact`].
///
/// # Returns
///
/// One item per visible symbol, each with a proximity-aware `sortText`.
///
/// # Errors
///
/// Infallible; a document that does not lex yields an empty vector.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_scope::items;
/// assert!(items("\"unterminated", 3, "").is_empty());
/// ```
pub fn items(text: &str, offset: usize, typed: &str) -> Vec<Value> {
    let mut visible: Vec<_> = collect(text)
        .into_iter()
        .filter(|symbol| symbol.visible_at(offset))
        .collect();
    visible.sort_by_key(|symbol| offset.abs_diff(symbol.offset));
    visible
        .into_iter()
        .enumerate()
        .map(|(distance, symbol)| {
            let sort = sort_text(tier_for(symbol.kind, &symbol.name, typed), distance);
            let doc = documentation(symbol.kind);
            if symbol.kind == SymbolKind::Function {
                callable(&symbol.name, &symbol.signature, doc, 3, &sort)
            } else {
                let kind = symbol.kind.completion_kind();
                described(&symbol.name, &symbol.signature, doc, kind, &sort)
            }
        })
        .collect()
}

fn tier_for(kind: SymbolKind, name: &str, typed: &str) -> Tier {
    match kind {
        SymbolKind::Function => Tier::Function,
        SymbolKind::Module => Tier::Module,
        _ if !typed.is_empty() && name.starts_with(typed) => Tier::LocalExact,
        _ => Tier::Local,
    }
}

fn documentation(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "Function declared in this file.",
        SymbolKind::Local => "Binding declared in this file.",
        SymbolKind::Param => "Parameter of the enclosing function.",
        SymbolKind::Module => "Module namespace imported by this file.",
    }
}
