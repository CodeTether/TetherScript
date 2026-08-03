//! The predicates behind channel deadlock detection.
//!
//! Split out from [`detect_deadlock`](super::deadlock::detect_deadlock) so the
//! rule reads as three separate, individually testable questions: is every live
//! task parked, could any channel still release one of them, and how do we name
//! the result. Each predicate is a pure read of the park table and the channel
//! registry, which is why detection costs nothing while nothing is stuck.

use super::park::{self, ParkKind};
use super::{parked, registry, wake};

/// True when every live task id appears in the channel park table.
pub(super) fn all_parked(live: &[u64]) -> bool {
    let waiting = parked::parked_tasks();
    live.iter().all(|task| waiting.contains(task))
}

/// True when a wakeup is queued or some channel could release a parked task.
pub(super) fn any_progress_possible(live: &[u64]) -> bool {
    if wake::pending() {
        return true;
    }
    park::entries()
        .into_iter()
        .filter(|(task, _, _)| live.contains(task))
        .any(|(_, channel, kind)| releasable(channel, kind))
}

/// True when this channel's current state would let a parked task continue.
fn releasable(channel: u64, kind: ParkKind) -> bool {
    registry::with(channel, |state| match kind {
        ParkKind::Recv => !state.queue.is_empty() || state.sealed(),
        ParkKind::Send => !state.is_full() || state.receivers == 0 || state.closed,
    })
    .unwrap_or(true)
}
