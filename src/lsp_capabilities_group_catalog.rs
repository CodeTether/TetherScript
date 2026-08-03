//! Module group: static documentation catalogs.
//!
//! A grouping file (see `lsp_capabilities_group_core.rs` for why the groups
//! exist). Everything here is data ported from `editor/vscode/lib/*-data.js`:
//! builtin functions, value methods, and `resource.*` constructors. No LSP
//! request logic lives in this group.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins::lookup;
//! use tetherscript::lsp_capabilities::methods;
//! assert!(lookup("fs_read").is_some());
//! assert!(methods::lookup("push").is_some());
//! ```

#[path = "lsp_capabilities_builtins.rs"]
pub mod builtins;
#[path = "lsp_capabilities_builtins_browser.rs"]
pub mod builtins_browser;
#[path = "lsp_capabilities_builtins_core.rs"]
pub mod builtins_core;
#[path = "lsp_capabilities_builtins_data.rs"]
pub mod builtins_data;
#[path = "lsp_capabilities_builtins_files.rs"]
pub mod builtins_files;
#[path = "lsp_capabilities_builtins_net.rs"]
pub mod builtins_net;
#[path = "lsp_capabilities_builtins_system.rs"]
pub mod builtins_system;
#[path = "lsp_capabilities_builtins_terminal.rs"]
pub mod builtins_terminal;
#[path = "lsp_capabilities_methods.rs"]
pub mod methods;
#[path = "lsp_capabilities_methods_factory.rs"]
pub mod methods_factory;
#[path = "lsp_capabilities_methods_resource.rs"]
pub mod methods_resource;
#[path = "lsp_capabilities_methods_value.rs"]
pub mod methods_value;
