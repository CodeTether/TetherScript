//! Deadlock-reporting built-in for script channels.

use crate::value::Value;

use super::{deadlock, parked, result};

/// Report a proven channel deadlock among the currently parked tasks.
///
/// A hung script with no output is far worse to debug than a named error, and a
/// cooperative single-threaded scheduler is one of the few places the hang is
/// actually decidable: there is no other thread and no external event that could
/// still deliver a wakeup. Scripts call this at a quiescent point — typically
/// before exiting a drain loop — so a stalled pipeline names itself.
///
/// # Arguments
///
/// * `values` — Empty.
///
/// # Returns
///
/// `Ok(nil)` while progress is still possible.
///
/// # Errors
///
/// Returns `Err(message)` inside the `Result` value when every parked task is
/// stuck and no channel can release any of them. The message names each task and
/// the channel and side it is parked on.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_deadlock, channel_open};
/// use tetherscript::value::Value;
///
/// channel_open(&[Value::Int(1), Value::Str("jobs".to_string().into())])?;
/// // Nothing is parked, so no deadlock is reported.
/// assert!(matches!(channel_deadlock(&[])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_deadlock(_values: &[Value]) -> Result<Value, String> {
    let live = parked::parked_tasks();
    Ok(result::unit(match deadlock::detect_deadlock(&live) {
        Some(report) => Err(report),
        None => Ok(()),
    }))
}
