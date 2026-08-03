//! Tests for the script-facing `chan_*` built-in surface.
//!
//! [`basics`] covers the happy path and close semantics; [`errors`] covers the
//! deadlock sources, select, and named failures. [`helpers`] and [`maps`] unwrap
//! the `Result` and status-map shapes the built-ins return.

#[path = "channel_tests_script_core.rs"]
mod basics;
#[path = "channel_tests_script_errors.rs"]
mod errors;
#[path = "channel_tests_script_helpers.rs"]
mod helpers;
#[path = "channel_tests_script_maps.rs"]
mod maps;
