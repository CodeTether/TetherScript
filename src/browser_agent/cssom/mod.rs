//! Agent-side CSSOM inspection for computed style snapshots.
//!
//! This module layers small Playwright-style style inspection APIs over the
//! deterministic browser style engine. It keeps viewport/media filtering local
//! to agent pages and returns stable property maps for agent assertions.

mod active_css;
mod conditions;
mod defaults;
mod lookup;
mod model;
mod page;

pub use model::ComputedStyle;

#[cfg(test)]
mod cascade_tests;
#[cfg(test)]
mod media_tests;
