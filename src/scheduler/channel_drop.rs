//! Endpoint retirement: the other half must never park forever.
//!
//! Dropping a half is a wake source, not a silent bookkeeping change. A receiver
//! parked on an empty channel whose last sender disappeared would otherwise
//! sleep until the process exits, and a sender parked on a full channel with no
//! receiver could never be drained. Retiring an endpoint therefore wakes every
//! task parked on the opposite side so each re-observes the channel and
//! completes, and frees the registry slot once both halves are gone.

use super::registry;

/// Free the registry slot once both halves are gone, so nothing leaks.
pub(super) fn release(id: u64) {
    let empty = registry::with(id, |state| state.senders == 0 && state.receivers == 0);
    if empty == Some(true) {
        registry::remove(id);
    }
}
