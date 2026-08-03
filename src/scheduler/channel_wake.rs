//! Wakeup handoff between channels and the scheduler ready queue.
//!
//! Channel operations never touch the scheduler directly; that would couple the
//! buffer to the run loop and make both harder to test. Instead a channel that
//! makes progress *queues* the task ids it unblocked here, and the scheduler
//! drains them with [`take_wakeups`] on each turn and pushes them back onto its
//! ready queue. The currency is the task id, which is exactly what the existing
//! `Scheduler::try_wake` already accepts.

use std::cell::RefCell;

use super::unpark;

thread_local! {
    static WAKEUPS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// Clear every park entry for `task` and queue it for the ready queue.
pub(super) fn wake(task: u64) {
    unpark::clear(task);
    WAKEUPS.with(|wakeups| {
        let mut wakeups = wakeups.borrow_mut();
        if !wakeups.contains(&task) {
            wakeups.push(task);
        }
    });
}

/// Return whether any wakeup is queued but not yet drained by the scheduler.
pub(super) fn pending() -> bool {
    WAKEUPS.with(|wakeups| !wakeups.borrow().is_empty())
}

/// Drain and return every task id woken by a channel since the last call.
///
/// # Returns
///
/// Woken task ids in wake order, oldest first, each appearing once. The queue is
/// left empty, so a scheduler turn pushes each id onto its ready queue exactly
/// once.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{self, RecvOutcome};
/// use tetherscript::value::Value;
///
/// let (tx, rx) = channel::bounded(1, "wakeups")?;
/// assert!(matches!(rx.recv(4), RecvOutcome::Parked));
/// tx.send(&Value::Int(1), 1)?;
/// assert_eq!(channel::take_wakeups(), vec![4]);
/// assert!(channel::take_wakeups().is_empty());
/// # Ok::<(), String>(())
/// ```
pub fn take_wakeups() -> Vec<u64> {
    WAKEUPS.with(|wakeups| std::mem::take(&mut *wakeups.borrow_mut()))
}
