//! The bound that keeps a runaway generator from owning the server.
//!
//! `http_serve` is a single-threaded accept loop, so an unbounded stream is not a
//! slow route — it is an outage for every other client. `/runaway` returns a
//! generator that never yields `nil`; only `max_events` can end it, so the fact
//! that the response *ends at all* is the assertion.

use std::time::Duration;

use super::response::stream_response;
use super::server::start;

#[test]
fn the_stream_bound_terminates_a_runaway_generator() {
    let server = start();
    let (_, body) = stream_response(server.port, "/runaway", Duration::from_secs(10));
    let text = String::from_utf8_lossy(&body).to_string();
    assert_eq!(
        text.matches("data: forever\n\n").count(),
        4,
        "bound must be exact, got {text:?}"
    );
    let expected = "data: forever\n\n".repeat(4);
    assert_eq!(text, expected);
}

#[test]
fn the_server_still_answers_after_a_bounded_stream_ends() {
    let server = start();
    let _ = stream_response(server.port, "/runaway", Duration::from_secs(10));
    // Reaching the bound must return the accept loop, not wedge it.
    let (_, body) = stream_response(server.port, "/health", Duration::from_secs(5));
    assert_eq!(body, b"ok\n".to_vec());
}
