//! Flexbox layout engine.

pub mod types;
pub mod parse;
pub mod resolve;
pub mod size;
pub mod position;
pub mod align;
pub mod wrap_lines;
pub mod algorithm;

pub use self::algorithm::perform_flex_layout;
pub use self::types::*;

#[cfg(test)]
mod tests;
