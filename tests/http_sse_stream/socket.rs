//! Raw-socket helpers: send a bare GET, read bytes, split head from body.
//!
//! Deliberately no HTTP client. Every helper here returns bytes, because the
//! assertions are about bytes; anything that returned a parsed response would
//! normalise away the very defects under test.

use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

#[path = "socket_read.rs"]
mod read;
#[path = "socket_scan.rs"]
mod scan;

pub(crate) use read::{read_to_close, read_until};
pub(crate) use scan::{find, split};

/// Open a connection and send a bare GET for `path`.
///
/// # Arguments
///
/// * `port` — Server port.
/// * `path` — Request target.
///
/// # Returns
///
/// The connected socket with a 2 s read timeout, request already flushed. The
/// request omits `Connection: close`, so any close is the server's decision and
/// the test observes it rather than requesting it.
///
/// # Panics
///
/// Panics when the connection or the write fails: neither is a condition any test
/// here is designed to tolerate, so failing loudly is correct.
pub(crate) fn request(port: u16, path: &str) -> TcpStream {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_millis(2_000)))
        .expect("read timeout");
    let get =
        format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nAccept: text/event-stream\r\n\r\n");
    stream.write_all(get.as_bytes()).expect("send request");
    stream.flush().expect("flush request");
    stream
}
