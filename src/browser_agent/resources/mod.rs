//! External resource registry for deterministic browser-agent pages.
//!
//! This module stores host-provided script, stylesheet, and image resources.
//! Page runtime code can apply registered text resources without performing
//! ambient network I/O, while image bytes remain available for inspection.

mod apply;
mod discover;
mod images;
mod inline;
mod page;
mod registry;
mod script;
mod style;
mod types;
mod url;

pub use registry::ResourceRegistry;
pub use types::{BrowserResource, ImageResourceMetadata, ResourceKind, ResourcePayload};

#[cfg(test)]
mod tests;
