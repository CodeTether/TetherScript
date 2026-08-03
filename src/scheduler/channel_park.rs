//! Park bookkeeping for tasks blocked on channels.
//!
//! A parked task is one that asked a channel for progress the channel could not
//! give: a send against a full buffer, or a receive against an empty one. Rather
//! than blocking the single cooperative thread, the operation records the task
//! here and returns a `Parked` outcome. One task may hold several entries at once
//! — that is how a `select` over two receives waits on both — but never the same
//! `(channel, side)` twice, so a retried park cannot leak a duplicate waiter. The
//! table answers two later questions: which waiter slots to scrub when a task is
//! woken or cancelled, and whether every live task is parked (deadlock).

use std::cell::RefCell;
use std::collections::HashMap;

/// Which side of a channel a task parked on.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ParkKind {
    /// Parked in `send` because the bounded buffer was full.
    Send,
    /// Parked in `recv` because the buffer was empty and not sealed.
    Recv,
}

thread_local! {
    static PARKED: RefCell<HashMap<u64, Vec<(u64, ParkKind)>>> = RefCell::new(HashMap::new());
}

/// Record that `task` is parked on `channel`'s `kind` side, at most once.
///
/// # Returns
///
/// `true` when this is a new park entry, `false` when the task was already
/// parked on that exact channel and side.
pub(super) fn park(task: u64, channel: u64, kind: ParkKind) -> bool {
    PARKED.with(|parked| {
        let mut parked = parked.borrow_mut();
        let parks = parked.entry(task).or_default();
        if parks.contains(&(channel, kind)) {
            return false;
        }
        parks.push((channel, kind));
        true
    })
}

/// Forget every park entry for `task`, returning the channels it waited on.
pub(super) fn take(task: u64) -> Vec<(u64, ParkKind)> {
    PARKED.with(|parked| parked.borrow_mut().remove(&task).unwrap_or_default())
}

/// Return every park entry as `(task, channel, kind)`, ascending.
pub(super) fn entries() -> Vec<(u64, u64, ParkKind)> {
    PARKED.with(|parked| {
        let mut entries: Vec<(u64, u64, ParkKind)> = parked
            .borrow()
            .iter()
            .flat_map(|(task, parks)| parks.iter().map(|(id, kind)| (*task, *id, *kind)))
            .collect();
        entries.sort_unstable();
        entries
    })
}
