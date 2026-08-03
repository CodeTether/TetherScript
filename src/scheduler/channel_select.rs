//! Selecting across several channel receives.
//!
//! A task that can only wait on one channel cannot multiplex, which is the whole
//! point of `select`. This module makes a receive *selectable*: it scans the
//! given receivers in order, takes the first that is ready, and otherwise parks
//! the task on **every** one of them so any single sender wakes it. When a later
//! receiver wins, the parks recorded by earlier empty ones are scrubbed, so no
//! dangling waiter survives a completed select. Scan order is deterministic,
//! matching the existing `select` over tasks, which keeps output reproducible.

use crate::value::Value;

use super::endpoint::Receiver;
use super::recv::RecvOutcome;
use super::unpark;

/// Result of a [`select_recv`] scan.
#[derive(Clone, Debug)]
pub enum SelectOutcome {
    /// Receiver at this index yielded a value.
    Ready(usize, Value),
    /// Receiver at this index is drained and sealed.
    Ended(usize),
    /// No receiver was ready; the task is parked on all of them.
    Parked,
}

/// Take a value from the first ready receiver, or park on all of them.
///
/// # Arguments
///
/// * `receivers` — Receivers to scan, in priority order.
/// * `task` — Id of the selecting task, parked on every receiver if none is ready.
///
/// # Returns
///
/// [`SelectOutcome::Ready`] with the winning index and its value,
/// [`SelectOutcome::Ended`] when the first non-parking receiver is drained and
/// sealed, or [`SelectOutcome::Parked`] when every receiver is empty and live.
///
/// # Errors
///
/// Returns `Err` when `receivers` is empty, because a select with no arms could
/// never complete and would be an unconditional deadlock.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{self, SelectOutcome};
/// use tetherscript::value::Value;
///
/// let (left_tx, left_rx) = channel::bounded(1, "left")?;
/// let (right_tx, right_rx) = channel::bounded(1, "right")?;
/// right_tx.send(&Value::Int(5), 1)?;
/// // The left arm is empty, so the ready right arm wins and nothing stays parked.
/// let chosen = channel::select_recv(&[&left_rx, &right_rx], 2)?;
/// assert!(matches!(chosen, SelectOutcome::Ready(1, Value::Int(5))));
/// assert!(channel::parked_tasks().is_empty());
/// drop((left_tx, right_tx));
/// # Ok::<(), String>(())
/// ```
pub fn select_recv(receivers: &[&Receiver], task: u64) -> Result<SelectOutcome, String> {
    if receivers.is_empty() {
        return Err("channel select: expected at least one receiver".into());
    }
    for (index, receiver) in receivers.iter().enumerate() {
        let outcome = match receiver.recv(task) {
            RecvOutcome::Value(value) => SelectOutcome::Ready(index, value),
            RecvOutcome::Ended | RecvOutcome::Gone => SelectOutcome::Ended(index),
            RecvOutcome::Parked => continue,
        };
        unpark::clear(task);
        return Ok(outcome);
    }
    Ok(SelectOutcome::Parked)
}
