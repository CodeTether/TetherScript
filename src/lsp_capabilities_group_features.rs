//! Module group: the three request handlers and the advertised capabilities.
//!
//! A grouping file (see `lsp_capabilities_group_core.rs` for why the groups
//! exist). Everything here turns an LSP request into an LSP reply; the analysis
//! it relies on lives in `group_core`, and the documentation text it quotes lives
//! in `group_catalog`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::capabilities::METHODS;
//! assert!(METHODS.contains(&"textDocument/definition"));
//! ```

#[path = "lsp_capabilities_capabilities.rs"]
pub mod capabilities;
#[path = "lsp_capabilities_completion.rs"]
pub mod completion;
#[path = "lsp_capabilities_completion_context.rs"]
pub mod completion_context;
#[path = "lsp_capabilities_completion_global.rs"]
pub mod completion_global;
#[path = "lsp_capabilities_completion_item.rs"]
pub mod completion_item;
#[path = "lsp_capabilities_completion_member.rs"]
pub mod completion_member;
#[path = "lsp_capabilities_completion_scope.rs"]
pub mod completion_scope;
#[path = "lsp_capabilities_definition.rs"]
pub mod definition;
#[path = "lsp_capabilities_definition_module.rs"]
pub mod definition_module;
#[path = "lsp_capabilities_definition_target.rs"]
pub mod definition_target;
#[path = "lsp_capabilities_hover.rs"]
pub mod hover;
#[path = "lsp_capabilities_hover_local.rs"]
pub mod hover_local;
#[path = "lsp_capabilities_hover_lookup.rs"]
pub mod hover_lookup;
#[path = "lsp_capabilities_hover_module.rs"]
pub mod hover_module;
