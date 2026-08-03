//! Channel mechanism: buffer, endpoints, parking, wakeup, and deadlock.
//!
//! This group owns everything that does not depend on the language surface. Each
//! submodule is one concern: [`state`] and [`registry`] hold the buffer,
//! [`send`] and [`recv`] move values, [`park`], [`unpark`], and [`wake`] carry
//! task ids to and from the scheduler, and [`deadlock`] proves when nothing in
//! the process can run again.

#[path = "channel_bounded.rs"]
pub(super) mod bounded;
#[path = "channel_cancel.rs"]
pub(super) mod cancel;
#[path = "channel_close.rs"]
pub(super) mod close;
#[path = "channel_current.rs"]
pub(super) mod current;
#[path = "channel_deadlock.rs"]
pub(super) mod deadlock;
#[path = "channel_deadlock_report.rs"]
pub(super) mod deadlock_report;
#[path = "channel_deadlock_rule.rs"]
pub(super) mod deadlock_rule;
#[path = "channel_drop.rs"]
pub(super) mod drop_endpoint;
#[path = "channel_drop_receiver.rs"]
pub(super) mod drop_receiver;
#[path = "channel_drop_sender.rs"]
pub(super) mod drop_sender;
#[path = "channel_endpoint.rs"]
pub(super) mod endpoint;
#[path = "channel_park.rs"]
pub(super) mod park;
#[path = "channel_parked.rs"]
pub(super) mod parked;
#[path = "channel_query.rs"]
pub(super) mod query;
#[path = "channel_query_recv.rs"]
pub(super) mod query_recv;
#[path = "channel_recv.rs"]
pub(super) mod recv;
#[path = "channel_registry.rs"]
pub(super) mod registry;
#[path = "channel_select.rs"]
pub(super) mod select;
#[path = "channel_send.rs"]
pub(super) mod send;
#[path = "channel_send_guard.rs"]
pub(super) mod send_guard;
#[path = "channel_state.rs"]
pub(super) mod state;
#[path = "channel_unpark.rs"]
pub(super) mod unpark;
#[path = "channel_wake.rs"]
pub(super) mod wake;
