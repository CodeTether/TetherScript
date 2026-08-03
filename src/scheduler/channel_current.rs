//! Current-task identity for channel operations.
//!
//! Channel parking is expressed in scheduler task ids, but the language-facing
//! built-ins (`chan_send`, `chan_recv`) are called from inside a running task
//! body and have no id argument. The scheduler therefore publishes the id of
//! the task it is currently driving here, and the built-ins read it. Task id
//! `0` means "the top-level script", which is still a legitimate parker: it is
//! the task the deadlock rule will name if nothing can make progress.

use std::cell::Cell;

thread_local! {
    static CURRENT: Cell<u64> = const { Cell::new(0) };
}

/// Publish the id of the task the scheduler is about to run.
///
/// # Arguments
///
/// * `task` — Scheduler task id, or `0` for the top-level script.
///
/// # Returns
///
/// The previously current id, so a caller can restore it after a nested run.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel;
///
/// let previous = channel::set_current_task(11);
/// assert_eq!(channel::current_task(), 11);
/// channel::set_current_task(previous);
/// ```
pub fn set_current_task(task: u64) -> u64 {
    CURRENT.with(|current| current.replace(task))
}

/// Return the id of the task currently being driven.
///
/// # Returns
///
/// The published task id, defaulting to `0` for the top-level script.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel;
///
/// channel::set_current_task(0);
/// assert_eq!(channel::current_task(), 0);
/// ```
pub fn current_task() -> u64 {
    CURRENT.with(Cell::get)
}
