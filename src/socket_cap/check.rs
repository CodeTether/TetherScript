//! The deny-by-default authorization decision and its error wording.

use super::grants;
use super::transport::Transport;

/// Authorize `host:port` for `transport`, naming the remedy on failure.
///
/// # Errors
///
/// Two distinct failures, deliberately worded differently so a user can tell
/// "you granted nothing" apart from "you granted something narrower":
///
/// * no grant installed for the transport
/// * a grant exists but does not cover this address
pub(super) fn run(
    transport: Transport,
    operation: &str,
    host: &str,
    port: u16,
) -> Result<(), String> {
    match grants::permits(transport, host, port) {
        Some(true) => Ok(()),
        Some(false) => Err(format!(
            "{operation}: {} access to {host}:{port} is outside the granted scope; widen `{}`",
            transport.label(),
            transport.flag()
        )),
        None => Err(format!(
            "{operation}: {} access requires `tetherscript run {} <host[:port]>` or `--access-mode full`",
            transport.label(),
            transport.flag()
        )),
    }
}
