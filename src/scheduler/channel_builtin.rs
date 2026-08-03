//! Channel creation and sending built-ins.

use crate::value::Value;

use super::send::SendOutcome;
use super::{args, bounded, current, handles, result};

/// Open a bounded channel and return its script handle.
///
/// # Arguments
///
/// * `values` — `[capacity: int, name: str]`.
///
/// # Returns
///
/// `Ok(handle)` as an int, usable with every other `chan_*` built-in.
///
/// # Errors
///
/// Returns the argument-coercion or zero-capacity error inside the `Result`
/// value; the outer `Result` is reserved for arity faults the caller prevents.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::channel_open;
/// use tetherscript::value::Value;
///
/// let handle = channel_open(&[Value::Int(2), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(handle, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_open(values: &[Value]) -> Result<Value, String> {
    Ok(result::result(
        args::capacity(&values[0], "chan_open").and_then(|capacity| {
            let name = args::name(&values[1], "chan_open")?;
            let (sender, receiver) = bounded::bounded(capacity, &name)?;
            Ok(Value::Int(handles::insert(sender, receiver)))
        }),
    ))
}

/// Send a value, reporting `"sent"` or `"parked"` under backpressure.
///
/// # Arguments
///
/// * `values` — `[handle: int, value: any]`.
///
/// # Returns
///
/// `Ok("sent")` once buffered, or `Ok("parked")` when the bounded buffer is
/// full and the current task has been recorded as a waiter.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value when the channel is closed, when
/// every receiver has been dropped, or when the handle is unknown.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_open, channel_send};
/// use tetherscript::value::Value;
///
/// let opened = channel_open(&[Value::Int(1), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(opened, Value::Result(_)));
/// assert!(matches!(
///     channel_send(&[Value::Int(1), Value::Int(9)])?,
///     Value::Result(_)
/// ));
/// # Ok::<(), String>(())
/// ```
pub fn channel_send(values: &[Value]) -> Result<Value, String> {
    Ok(result::result(
        args::handle(&values[0], "chan_send").and_then(|handle| {
            handles::with(handle, |(sender, _)| match sender {
                Some(sender) => sender.send(&values[1], current::current_task()),
                None => Err(format!("chan_send: sender {handle} was already dropped")),
            })?
            .map(|outcome| match outcome {
                SendOutcome::Sent => result::text("sent"),
                SendOutcome::Parked => result::text("parked"),
            })
        }),
    ))
}
