//! Feature-selected software text rendering.

#[cfg(not(feature = "native-window"))]
#[path = "browser_text/bitmap.rs"]
mod backend;
#[cfg(feature = "native-window")]
#[path = "browser_text/truetype.rs"]
mod backend;

pub(crate) use backend::{draw, measure};
