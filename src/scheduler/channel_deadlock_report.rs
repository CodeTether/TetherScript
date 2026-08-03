//! Human-readable naming of a proven channel deadlock.
//!
//! "Error" is not an error message. A stalled pipeline must say which task is
//! stuck, on which channel, and on which side, because that triple is exactly
//! what tells an author whether they forgot a `chan_close`, forgot to drain, or
//! wired a cycle.

use super::park::{self, ParkKind};
use super::registry;

/// Build the diagnostic naming every stuck task and the channel it waits on.
pub(super) fn report(live: &[u64]) -> String {
    let stuck = park::entries()
        .into_iter()
        .filter(|(task, _, _)| live.contains(task))
        .map(describe)
        .collect::<Vec<String>>()
        .join("; ");
    format!("channel deadlock: every live task is parked and no channel can make progress: {stuck}")
}

/// Name one park entry as `task N parked in channel `name`.side`.
fn describe(entry: (u64, u64, ParkKind)) -> String {
    let (task, channel, kind) = entry;
    let name = registry::with(channel, |state| state.name.clone())
        .unwrap_or_else(|| format!("#{channel}"));
    let side = match kind {
        ParkKind::Recv => "recv",
        ParkKind::Send => "send",
    };
    format!("task {task} parked in channel `{name}`.{side}")
}
