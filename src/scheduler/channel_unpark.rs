//! Removal of a task's waiter slots from every channel it parked on.
//!
//! Both waking and cancelling must scrub the same bookkeeping, so the logic
//! lives here once. Leaving a stale id in a channel's waiter queue would later
//! "wake" a task that is gone, and would also make that channel look busy to
//! deadlock detection, so the park table and the channel queues are always
//! retired together.

use super::park::{self, ParkKind};
use super::registry;

/// Drop `task`'s park entries and its slot in every channel waiter queue.
///
/// # Returns
///
/// `true` when the task held at least one park entry.
pub(super) fn clear(task: u64) -> bool {
    let parks = park::take(task);
    for (channel, kind) in &parks {
        registry::edit(*channel, |state| {
            let queue = match kind {
                ParkKind::Send => &mut state.send_waiters,
                ParkKind::Recv => &mut state.recv_waiters,
            };
            queue.retain(|waiting| *waiting != task);
        });
    }
    !parks.is_empty()
}
