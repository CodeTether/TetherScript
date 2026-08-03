//! Public view of which tasks are currently parked on a channel.

use super::park;

/// Return every currently parked task id, ascending and deduplicated.
///
/// # Returns
///
/// Task ids parked on at least one channel. Used by deadlock detection and by
/// supervisors that want to report where a script is stuck.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel;
///
/// let (_tx, rx) = channel::bounded(1, "idle")?;
/// let _ = rx.recv(9);
/// assert_eq!(channel::parked_tasks(), vec![9]);
/// # Ok::<(), String>(())
/// ```
pub fn parked_tasks() -> Vec<u64> {
    let mut ids: Vec<u64> = park::entries().into_iter().map(|entry| entry.0).collect();
    ids.dedup();
    ids
}
