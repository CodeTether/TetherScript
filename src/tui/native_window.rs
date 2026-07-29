//! Cross-platform native widget presentation for TUI views.

#[cfg(not(feature = "native-window"))]
#[path = "native_window/disabled.rs"]
mod backend;
#[cfg(feature = "native-window")]
#[path = "native_window/enabled.rs"]
mod backend;

pub(super) use backend::{agent_builtin, builtin};
