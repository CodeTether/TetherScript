//! Dependency-free software text rendering.

#[path = "browser_text/bitmap.rs"]
mod backend;

pub(crate) use backend::{draw, measure};
