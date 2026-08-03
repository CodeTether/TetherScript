//! Script-facing `chan_*` built-ins.
//!
//! This group is a thin translation layer: it coerces dynamic argument values,
//! calls the channel mechanism in the sibling `internals` group, and projects the
//! outcome into language `Result` and status-map values. No channel policy lives
//! here, so the mechanism stays testable without the language surface.

pub(super) use super::internals::{
    bounded, current, deadlock, endpoint, parked, recv, select, send,
};

#[path = "channel_args.rs"]
pub(super) mod args;
#[path = "channel_builtin.rs"]
pub(super) mod builtin;
#[path = "channel_builtin_close.rs"]
pub(super) mod builtin_close;
#[path = "channel_builtin_deadlock.rs"]
pub(super) mod builtin_deadlock;
#[path = "channel_builtin_drop.rs"]
pub(super) mod builtin_drop;
#[path = "channel_builtin_ops.rs"]
pub(super) mod builtin_ops;
#[path = "channel_builtin_query.rs"]
pub(super) mod builtin_query;
#[path = "channel_builtin_select.rs"]
pub(super) mod builtin_select;
#[path = "channel_handles.rs"]
pub(super) mod handles;
#[path = "channel_result.rs"]
pub(super) mod result;
#[path = "channel_select_args.rs"]
pub(super) mod select_args;
#[path = "channel_select_report.rs"]
pub(super) mod select_report;
