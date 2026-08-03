//! Reading one complete streaming response.
//!
//! Separated from [`super::socket`] so the low-level byte helpers stay free of
//! any notion of "a whole response", which only makes sense for a stream that
//! ends on its own.

use std::time::Duration;

use super::socket::{read_to_close, request, split};

/// Read one complete streaming response as `(head, body)`.
///
/// # Arguments
///
/// * `port` — Server port.
/// * `path` — Request target.
/// * `budget` — Maximum time to wait for the stream to finish.
///
/// # Returns
///
/// The head as text and the body as bytes.
///
/// # Panics
///
/// Panics when the server sent nothing, which means the route did not respond at
/// all — a far more useful failure than an empty-body assertion downstream.
pub(crate) fn stream_response(port: u16, path: &str, budget: Duration) -> (String, Vec<u8>) {
    let mut socket = request(port, path);
    let bytes = read_to_close(&mut socket, budget);
    assert!(!bytes.is_empty(), "server sent nothing for {path}");
    split(&bytes)
}
