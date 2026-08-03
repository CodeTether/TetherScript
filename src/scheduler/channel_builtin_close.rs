//! Close, drop, and lifecycle built-ins for script channels.

use crate::value::Value;

use super::{args, handles, result};

/// Close the sender half, preserving buffered values for the receiver to drain.
///
/// Closing is not a discard. The classic bug is to treat close as "stop now",
/// which loses every value the producer already handed over; here close only
/// forbids new sends.
///
/// # Arguments
///
/// * `values` — `[handle: int]`.
///
/// # Returns
///
/// `Ok(nil)`. Closing twice is harmless, and receivers parked on the empty
/// buffer are woken so they can observe end-of-stream.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value for an unknown handle.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel::{channel_close, channel_open};
/// use tetherscript::value::Value;
///
/// channel_open(&[Value::Int(1), Value::Str("jobs".to_string().into())])?;
/// assert!(matches!(channel_close(&[Value::Int(1)])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_close(values: &[Value]) -> Result<Value, String> {
    Ok(result::unit(
        args::handle(&values[0], "chan_close").and_then(|handle| {
            let retired = handles::with(handle, |(sender, _)| {
                if let Some(sender) = sender.as_ref() {
                    sender.close();
                }
                sender.take()
            })?;
            drop(retired);
            handles::prune(handle);
            Ok(())
        }),
    ))
}
