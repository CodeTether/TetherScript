//! `Default` for [`super::engine::Engine`].
//!
//! Split from `engine.rs` to keep that file within the 50-line limit; clippy
//! wants `Default` alongside an argument-free `new`, but the trait impl is a
//! separate concern from the engine's matching behavior.

use super::engine::Engine;

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
