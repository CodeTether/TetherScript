//! Feature-selected native-window backend.

#[cfg(not(feature = "native-window"))]
#[path = "disabled.rs"]
mod implementation;
#[cfg(feature = "native-window")]
#[path = "enabled.rs"]
mod implementation;

pub(crate) use implementation::Slot;
