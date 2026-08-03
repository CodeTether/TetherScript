//! Unit tests for bounded channel behaviour.
//!
//! Split by concern: [`flow`] covers ordering and backpressure, [`close`] covers
//! sealing and draining, [`endpoints`] covers endpoint loss and cancellation,
//! [`select`] covers multiplexing, [`deadlock`] covers the detection rule,
//! [`volume`] proves no loss or duplication through a small buffer, and
//! [`script`] covers the `chan_*` built-ins. [`support`] holds the isolation
//! helpers every group uses.

#[path = "channel_tests_close.rs"]
mod close;
#[path = "channel_tests_deadlock.rs"]
mod deadlock;
#[path = "channel_tests_endpoints.rs"]
mod endpoints;
#[path = "channel_tests_flow.rs"]
mod flow;
#[path = "channel_tests_script.rs"]
mod script;
#[path = "channel_tests_select.rs"]
mod select;
#[path = "channel_tests_support.rs"]
mod support;
#[path = "channel_tests_volume.rs"]
mod volume;
