//! Construction of a bounded channel pair.

use super::endpoint::{Receiver, Sender};
use super::registry;

/// Create a bounded channel and return its sender and receiver halves.
///
/// # Arguments
///
/// * `capacity` — Maximum buffered values. Must be greater than zero; a
///   zero-capacity buffer could never accept a value without a rendezvous
///   partner, which this cooperative scheduler does not provide.
/// * `name` — Diagnostic name repeated in every error this channel raises, so
///   a failure names the channel that failed rather than saying "error".
///
/// # Returns
///
/// `Ok((sender, receiver))` for a channel that buffers at most `capacity`
/// values before a send applies backpressure by parking instead of growing.
///
/// # Errors
///
/// Returns `Err` naming the channel when `capacity` is zero.
///
/// # Examples
///
/// ```
/// use tetherscript::scheduler::channel;
///
/// let (tx, rx) = channel::bounded(4, "pipeline")?;
/// assert_eq!(tx.capacity(), 4);
/// assert_eq!(rx.len(), 0);
/// assert!(channel::bounded(0, "pipeline").is_err());
/// # Ok::<(), String>(())
/// ```
pub fn bounded(capacity: usize, name: &str) -> Result<(Sender, Receiver), String> {
    if capacity == 0 {
        return Err(format!(
            "channel `{name}`: capacity must be greater than zero"
        ));
    }
    let id = registry::create(name, capacity);
    Ok((Sender { id }, Receiver { id }))
}
