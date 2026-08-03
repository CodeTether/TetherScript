//! Receiver-drop built-in: a send with no receivers must fail, not park.

use crate::value::Value;

use super::{args, handles, result};

/// Drop the receiver half so later sends fail instead of parking forever.
///
/// A send with no receivers is a deadlock source: nothing can ever drain the
/// buffer, so parking would sleep until the process exits. Retiring the receiver
/// therefore wakes every parked sender so each observes a named failure.
///
/// # Arguments
///
/// * `values` — `[handle: int]`.
///
/// # Returns
///
/// `Ok(nil)`.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value for an unknown handle.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_drop_receiver, channel_open, channel_send};
/// use tetherscript::value::Value;
///
/// channel_open(&[Value::Int(1), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(
///     channel_drop_receiver(&[Value::Int(1)])?,
///     Value::Result(_)
/// ));
/// // The send now fails by name rather than parking forever.
/// assert!(matches!(channel_send(&[Value::Int(1), Value::Int(1)])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_drop_receiver(values: &[Value]) -> Result<Value, String> {
    Ok(result::unit(
        args::handle(&values[0], "chan_drop_receiver").and_then(|handle| {
            let retired = handles::with(handle, |(_, receiver)| receiver.take())?;
            drop(retired);
            handles::prune(handle);
            Ok(())
        }),
    ))
}
