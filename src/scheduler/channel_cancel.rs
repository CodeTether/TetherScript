//! Cancellation cleanup for tasks parked on a channel.

use super::unpark;

/// Remove a cancelled task's park entries and its channel waiter slots.
///
/// Cancelling a task that is parked on a channel must unpark it and leave
/// neither a dangling waiter nor a leaked buffer slot; otherwise the channel
/// would later hand a wakeup to a task that no longer exists, and would look
/// busy to deadlock detection forever.
///
/// # Arguments
///
/// * `task` — Id of the cancelled task.
///
/// # Returns
///
/// `true` when the task was parked on at least one channel and has been cleaned
/// up, `false` when it was not parked at all.
///
/// # Errors
///
/// This function does not fail; a task that was never parked is not an error.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{self, RecvOutcome};
///
/// let (_tx, rx) = channel::bounded(1, "cancelled")?;
/// assert!(matches!(rx.recv(7), RecvOutcome::Parked));
/// assert!(channel::cancel_task(7));
/// assert!(channel::parked_tasks().is_empty());
/// assert!(!channel::cancel_task(7));
/// # Ok::<(), String>(())
/// ```
pub fn cancel_task(task: u64) -> bool {
    unpark::clear(task)
}
