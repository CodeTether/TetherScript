//! Pre-send validation: which sends fail and which merely park.
//!
//! Two conditions look similar but must behave differently. A *full* buffer is
//! temporary, so the sender parks and retries. A channel with no receivers can
//! never drain, so parking would be a permanent hang — that case must fail by
//! name instead. Keeping the distinction in one place is what stops a future
//! edit from quietly turning a named error back into a deadlock.

use super::registry;

/// Decide whether a send may proceed, must park, or must fail.
///
/// # Arguments
///
/// * `id` — Channel id being sent to.
///
/// # Returns
///
/// `Ok(true)` when the caller should park because the buffer is full, and
/// `Ok(false)` when there is room to buffer immediately.
///
/// # Errors
///
/// Returns `Err` naming the channel when it has been closed, when every receiver
/// was dropped, or when the channel no longer exists.
pub(super) fn check(id: u64) -> Result<bool, String> {
    let Some((name, closed, receivers, full)) = registry::with(id, |state| {
        (
            state.name.clone(),
            state.closed,
            state.receivers,
            state.is_full(),
        )
    }) else {
        return Err("channel.send: channel no longer exists".into());
    };
    if closed {
        return Err(format!("channel `{name}`.send: channel is closed"));
    }
    if receivers == 0 {
        return Err(format!("channel `{name}`.send: all receivers were dropped"));
    }
    Ok(full)
}
