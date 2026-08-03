//! Module group: document analysis primitives.
//!
//! A grouping file, not a concern of its own. It exists because
//! [`crate::lsp_capabilities`] would otherwise need one `#[path]` attribute plus
//! one `mod` line for each of the ~39 files in this feature, which would push
//! that file past the repository's 50-line limit. The three group files
//! (`group_core`, `group_catalog`, `group_features`) are glob re-exported, so the
//! public paths stay flat: `crate::lsp_capabilities::position`, not
//! `crate::lsp_capabilities::group_core::position`.
//!
//! Everything declared here answers the question "what does this document say?",
//! independently of any LSP request.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::position::byte_offset;
//! assert_eq!(byte_offset("let x = 1", 0, 4), Some(4));
//! ```

#[path = "lsp_capabilities_context.rs"]
pub mod context;
#[path = "lsp_capabilities_docs.rs"]
pub mod docs;
#[path = "lsp_capabilities_jsonval.rs"]
pub mod jsonval;
#[path = "lsp_capabilities_keywords.rs"]
pub mod keywords;
#[path = "lsp_capabilities_module.rs"]
pub mod module;
#[path = "lsp_capabilities_position.rs"]
pub mod position;
#[path = "lsp_capabilities_rank.rs"]
pub mod rank;
#[path = "lsp_capabilities_request.rs"]
pub mod request;
#[path = "lsp_capabilities_scan.rs"]
pub mod scan;
#[path = "lsp_capabilities_scan_braces.rs"]
mod scan_braces;
#[path = "lsp_capabilities_symbol.rs"]
pub mod symbol;
#[path = "lsp_capabilities_symbols.rs"]
pub mod symbols;
#[path = "lsp_capabilities_symbols_fn.rs"]
pub mod symbols_fn;
#[path = "lsp_capabilities_symbols_local.rs"]
pub mod symbols_local;
#[path = "lsp_capabilities_uri.rs"]
pub mod uri;
