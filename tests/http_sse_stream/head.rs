//! Response-head assertions: no `Content-Length`, and the framing is named.
//!
//! `Content-Length` is the load-bearing absence. A streaming server cannot know
//! the length before the generator runs, and any guess is worse than silence: too
//! small and the client treats the surplus as another response, too large and it
//! waits forever for bytes that never come.

use std::time::Duration;

use super::response::stream_response;
use super::server::start;

#[test]
fn the_head_has_no_content_length_and_names_its_framing() {
    let server = start();
    let (head, _) = stream_response(server.port, "/events", Duration::from_secs(5));
    let lowered = head.to_ascii_lowercase();
    assert!(
        !lowered.contains("content-length"),
        "a stream cannot know its length: {head}"
    );
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert!(lowered.contains("connection: close"), "{head}");
    assert!(
        lowered.contains("content-type: text/event-stream"),
        "{head}"
    );
    assert!(lowered.contains("cache-control: no-cache"), "{head}");
}

#[test]
fn an_ordinary_response_on_the_same_server_is_unaffected() {
    let server = start();
    let (head, body) = stream_response(server.port, "/health", Duration::from_secs(5));
    assert!(
        head.to_ascii_lowercase().contains("content-length: 3"),
        "a fixed-length response must keep its length: {head}"
    );
    assert_eq!(body, b"ok\n".to_vec());
}
