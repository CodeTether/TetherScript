//! The channel receive built-in.

use crate::value::Value;

use super::recv::RecvOutcome;
use super::{args, current, handles, result};

/// Receive a value, reporting `"value"`, `"end"`, or `"parked"`.
///
/// # Arguments
///
/// * `values` — `[handle: int]`.
///
/// # Returns
///
/// `Ok(map)` with a `status` of `"value"` (plus a `value` field), `"end"` once
/// the channel is sealed *and* drained, or `"parked"` while more may arrive. A
/// status map is used rather than a bare value because a channel is allowed to
/// carry `nil`, so `nil` cannot double as end-of-stream.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value when the handle is unknown or its
/// receiver half was already dropped.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_open, channel_recv, channel_send};
/// use tetherscript::value::Value;
///
/// let opened = channel_open(&[Value::Int(1), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(opened, Value::Result(_)));
/// channel_send(&[Value::Int(1), Value::Int(4)])?;
/// assert!(matches!(channel_recv(&[Value::Int(1)])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_recv(values: &[Value]) -> Result<Value, String> {
    Ok(result::result(
        args::handle(&values[0], "chan_recv").and_then(|handle| {
            handles::with(handle, |(_, receiver)| match receiver {
                Some(receiver) => Ok(receiver.recv(current::current_task())),
                None => Err(format!("chan_recv: receiver {handle} was already dropped")),
            })?
            .map(describe)
        }),
    ))
}

/// Project a receive outcome into the script-visible status map.
fn describe(outcome: RecvOutcome) -> Value {
    match outcome {
        RecvOutcome::Value(value) => {
            result::map(vec![("status", result::text("value")), ("value", value)])
        }
        RecvOutcome::Ended | RecvOutcome::Gone => {
            result::map(vec![("status", result::text("end"))])
        }
        RecvOutcome::Parked => result::map(vec![("status", result::text("parked"))]),
    }
}
