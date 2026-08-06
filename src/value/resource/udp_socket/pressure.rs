//! Backpressure-aware error wording for nonblocking datagram I/O.

use std::io;

/// Describe an I/O failure, naming backpressure distinctly.
///
/// A nonblocking socket reports "no datagram ready" as `WouldBlock`, which is a
/// normal polling outcome rather than a fault; saying so keeps a caller from
/// treating it as a hard failure.
pub(super) fn pressure(operation: &str, error: io::Error) -> String {
    if error.kind() == io::ErrorKind::WouldBlock {
        format!("udp_socket.{operation}: backpressure: operation would block")
    } else {
        format!("udp_socket.{operation}: {error}")
    }
}
