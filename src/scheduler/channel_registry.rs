//! Per-thread table of live channels.
//!
//! The cooperative scheduler is single-threaded, so channel state lives in a
//! thread-local table keyed by a monotonic channel id. Handles store only that
//! id, which keeps [`Sender`](super::endpoint::Sender) and
//! [`Receiver`](super::endpoint::Receiver) cheap to clone and lets deadlock
//! detection inspect a channel without holding a reference to user data.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use super::state::ChannelState;

thread_local! {
    static CHANNELS: RefCell<HashMap<u64, ChannelState>> = RefCell::new(HashMap::new());
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

/// Register a new channel and return its stable id.
pub(super) fn create(name: &str, capacity: usize) -> u64 {
    let id = NEXT_ID.with(|next| {
        let id = next.get();
        next.set(id.saturating_add(1));
        id
    });
    CHANNELS.with(|channels| {
        let _replaced = channels
            .borrow_mut()
            .insert(id, ChannelState::new(name, capacity));
    });
    id
}

/// Run `action` against a live channel, or return `None` when it is gone.
pub(super) fn with<R>(id: u64, action: impl FnOnce(&mut ChannelState) -> R) -> Option<R> {
    CHANNELS.with(|channels| channels.borrow_mut().get_mut(&id).map(action))
}

/// Mutate a channel when it exists, ignoring an already-released channel.
pub(super) fn edit(id: u64, action: impl FnOnce(&mut ChannelState)) {
    let _released = with(id, action);
}

/// Forget a channel whose endpoints are all gone, releasing its slot.
pub(super) fn remove(id: u64) {
    CHANNELS.with(|channels| channels.borrow_mut().remove(&id));
}
