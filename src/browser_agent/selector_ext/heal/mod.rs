//! Self-healing selector support.

pub mod cache;
pub mod extract;
pub mod score;
pub mod search;

pub use cache::SelectorHealCache;
pub use extract::{extract_props, ElementLike};
pub use score::{element_similarity, ElementProps};
pub use search::heal_selector;

#[cfg(test)]
mod tests;
