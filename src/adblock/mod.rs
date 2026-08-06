//! Ad-blocking building blocks for agent and tool workloads.
//!
//! Provides reusable layers for building an ad blocker end to end:
//! filter parsing, network matching, cosmetic filters, and an engine.
//!
//! # Example
//!
//! ```no_run
//! use tetherscript::adblock::Engine;
//!
//! let mut engine = Engine::new();
//! engine.add_list("||ads.example.com^\n||tracker.net^$third-party");
//! assert!(engine.should_block("https://ads.example.com/banner.gif", "site.com"));
//! ```

mod classify;
pub(crate) mod cosmetic;
mod engine;
pub(crate) mod network;
pub(crate) mod parse;
mod resource_type;
pub(crate) mod rule;

pub use engine::Engine;
pub use resource_type::ResourceType;
pub use rule::{FilterType, Rule};
