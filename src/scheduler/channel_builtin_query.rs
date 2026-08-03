//! Inspection built-ins for script channels.

use crate::value::Value;

use super::{args, handles, result};

/// Return how many values are currently buffered.
///
/// # Arguments
///
/// * `values` — `[handle: int]`.
///
/// # Returns
///
/// `Ok(int)` buffer occupancy.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value for an unknown handle or a handle
/// whose halves are both retired.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_len, channel_open};
/// use tetherscript::value::Value;
///
/// channel_open(&[Value::Int(2), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(channel_len(&[Value::Int(1)])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_len(values: &[Value]) -> Result<Value, String> {
    Ok(result::result(
        args::handle(&values[0], "chan_len").and_then(|handle| {
            handles::with(handle, |(sender, receiver)| {
                sender
                    .as_ref()
                    .map(|sender| sender.len())
                    .or_else(|| receiver.as_ref().map(|receiver| receiver.len()))
                    .map(|length| Value::Int(length as i64))
                    .ok_or_else(|| format!("chan_len: channel {handle} was fully released"))
            })?
        }),
    ))
}

/// Return whether the channel is sealed and fully drained.
///
/// # Arguments
///
/// * `values` — `[handle: int]`.
///
/// # Returns
///
/// `Ok(bool)`; `true` only once no value remains *and* no value can arrive, so a
/// close never hides values a producer already handed over.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value when the receiver half is gone.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_ended, channel_open};
/// use tetherscript::value::Value;
///
/// channel_open(&[Value::Int(1), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(channel_ended(&[Value::Int(1)])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_ended(values: &[Value]) -> Result<Value, String> {
    Ok(result::result(
        args::handle(&values[0], "chan_ended").and_then(|handle| {
            handles::with(handle, |(_, receiver)| match receiver {
                Some(receiver) => Ok(Value::Bool(receiver.is_ended())),
                None => Err(format!("chan_ended: receiver {handle} was already dropped")),
            })?
        }),
    ))
}
