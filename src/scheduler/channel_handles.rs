//! Script-visible channel handles.
//!
//! The language reaches channels through a small integer handle rather than a
//! new [`Value`](crate::value::Value) variant, so this feature needs no change
//! to the value representation. The owning [`Sender`]/[`Receiver`] pair lives
//! here; dropping either half through the table is what makes "all senders
//! dropped" and "all receivers dropped" observable to scripts.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use super::endpoint::{Receiver, Sender};

pub(super) type Halves = (Option<Sender>, Option<Receiver>);

thread_local! {
    static HANDLES: RefCell<HashMap<i64, Halves>> = RefCell::new(HashMap::new());
    static NEXT: Cell<i64> = const { Cell::new(1) };
}

/// Store a fresh pair and return its newly allocated handle.
pub(super) fn insert(sender: Sender, receiver: Receiver) -> i64 {
    let handle = NEXT.with(|next| {
        let handle = next.get();
        next.set(handle.saturating_add(1));
        handle
    });
    HANDLES.with(|handles| {
        let _replaced = handles
            .borrow_mut()
            .insert(handle, (Some(sender), Some(receiver)));
    });
    handle
}

/// Run `action` against the halves of `handle`, or report an unknown handle.
pub(super) fn with<R>(handle: i64, action: impl FnOnce(&mut Halves) -> R) -> Result<R, String> {
    HANDLES
        .with(|handles| handles.borrow_mut().get_mut(&handle).map(action))
        .ok_or_else(|| format!("channel: unknown channel handle {handle}"))
}

/// Remove a handle whose halves are both retired, releasing the table slot.
pub(super) fn prune(handle: i64) {
    HANDLES.with(|handles| {
        let mut handles = handles.borrow_mut();
        if handles
            .get(&handle)
            .is_some_and(|(tx, rx)| tx.is_none() && rx.is_none())
        {
            handles.remove(&handle);
        }
    });
}
