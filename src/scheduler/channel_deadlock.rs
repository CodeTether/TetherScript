//! Cheap deadlock detection for the cooperative scheduler.
//!
//! In a preemptive, multi-threaded runtime "everything is waiting" is not a
//! decidable state: another thread, a timer, or an OS event could still make
//! progress, so a runtime can only guess with timeouts. This scheduler is the
//! opposite case, and that is why detection is cheap and exact here: it is
//! single-threaded, every wake source is in-process, and both the parked tasks
//! and the channels they wait on are enumerable. If every live task is parked
//! on a channel, no wakeup is queued, and no channel is in a state that could
//! release a waiter, then nothing in the process can ever run again. Reporting
//! that by name beats hanging with no output, which is the worst possible
//! failure mode to debug.

use super::{deadlock_report, deadlock_rule};

/// Report a proven deadlock among the given live tasks.
///
/// # Arguments
///
/// * `live` — Ids of every task that is not yet done, from the scheduler.
///
/// # Returns
///
/// `Some(message)` naming the stuck tasks when every live task is parked on a
/// channel and no channel can release any of them; `None` whenever progress is
/// still possible, including when `live` is empty.
///
/// # Errors
///
/// This function does not fail; the returned message is the diagnostic the
/// caller should surface as a script error instead of hanging.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{self, RecvOutcome};
///
/// let (tx, rx) = channel::bounded(1, "stalled")?;
/// assert!(matches!(rx.recv(1), RecvOutcome::Parked));
/// // Task 1 waits for a value; only task 1 exists, so nobody can send one.
/// let report = channel::detect_deadlock(&[1]).expect("deadlock is provable");
/// assert!(report.contains("stalled"));
/// assert!(report.contains("deadlock"));
/// drop(tx);
/// # Ok::<(), String>(())
/// ```
pub fn detect_deadlock(live: &[u64]) -> Option<String> {
    if live.is_empty() || !deadlock_rule::all_parked(live) {
        return None;
    }
    if deadlock_rule::any_progress_possible(live) {
        return None;
    }
    Some(deadlock_report::report(live))
}
