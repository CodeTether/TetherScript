//! Ad-blocking building blocks for agent and tool workloads.
//!
//! Provides reusable layers for building an ad blocker end to end:
//! filter parsing, network matching, cosmetic filters, and an engine.
//!
//! # Example
//!
//! `Engine` is internal to the `http_builtins` installation path, not part of
//! the crate's public surface, so this example is illustrative rather than
//! compiled. Scripts reach these layers through the `adblock_*` built-ins.
//!
//! ```ignore
//! use tetherscript::adblock::Engine;
//!
//! let mut engine = Engine::new();
//! engine.add_list("||ads.example.com^\n||tracker.net^$third-party");
//! assert!(engine.should_block("https://ads.example.com/banner.gif", "site.com"));
//! ```

mod classify;
pub(crate) mod cosmetic;
mod engine;
#[cfg(test)]
mod engine_default;
#[cfg(test)]
mod engine_tests;
pub(crate) mod network;
pub(crate) mod parse;
mod resource_type;
pub(crate) mod rule;

pub use resource_type::ResourceType;
