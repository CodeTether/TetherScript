//! Global-position completion: scope symbols plus builtins and keywords.
//!
//! Ranking follows the tiers documented in [`crate::lsp_capabilities::rank`].
//! In-scope symbols come from
//! [`crate::lsp_capabilities::completion_scope`]; this file adds the language's
//! own vocabulary and concatenates the two in tier order.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::completion_global::items;
//! use tetherscript::lsp_capabilities::jsonval::{field, ValueText};
//!
//! let source = "let total = 1\n";
//! let labels: Vec<String> = items(source, source.len(), "")
//!     .iter()
//!     .filter_map(|item| field(item, "label").as_deref_str().map(str::to_string))
//!     .collect();
//! assert!(labels.contains(&"total".to_string()));
//! assert!(labels.contains(&"println".to_string()));
//! assert!(labels.contains(&"let".to_string()));
//! ```

use crate::lsp_capabilities::completion_item::{callable, plain};
use crate::lsp_capabilities::rank::{Tier, sort_text};
use crate::lsp_capabilities::{builtins, completion_scope, keywords};
use crate::value::Value;

/// Every completion item offered outside member position.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `offset` — Cursor byte offset, used for scope filtering and proximity.
/// * `typed` — Partial word already typed, used only to pick a tier; the client
///   performs the final filtering, so no candidate is dropped here.
///
/// # Returns
///
/// Scope symbols, builtins, keywords, and constants, each carrying a `sortText`
/// that encodes the server's ranking.
///
/// # Errors
///
/// Infallible; a document that does not lex still yields builtins and keywords.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::completion_global::items;
/// assert!(!items("\"unterminated", 3, "").is_empty());
/// ```
pub fn items(text: &str, offset: usize, typed: &str) -> Vec<Value> {
    let mut out = completion_scope::items(text, offset, typed);
    let builtin_sort = sort_text(Tier::Builtin, 0);
    for entry in builtins::iter() {
        let signature = builtins::signature(entry);
        out.push(callable(entry.0, &signature, entry.2, 3, &builtin_sort));
    }
    let keyword_sort = sort_text(Tier::Keyword, 0);
    for word in keywords::KEYWORDS {
        out.push(plain(word, 14, &keyword_sort));
    }
    let constant_sort = sort_text(Tier::Constant, 0);
    for word in keywords::CONSTANTS {
        out.push(plain(word, 21, &constant_sort));
    }
    out
}
