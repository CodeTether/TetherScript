//! One closed browser tab must not take the server with it.
//!
//! The failure mode: the client goes away mid-stream, the write fails, and the
//! server either panics or keeps calling a generator forever against a dead
//! socket. Because the accept loop is single-threaded, either outcome is fatal —
//! the server can never serve anyone again. So the assertion is not "no panic"
//! but "it answers a later request".

use std::io::Write;
use std::net::TcpStream;
use std::time::{Duration, Instant};

use super::server::start;
use super::socket::{find, read_to_close, read_until, request};

/// Ask `/health` on a fresh connection and report whether it answered `ok`.
fn health_ok(port: u16) -> bool {
    let Ok(mut socket) = TcpStream::connect(("127.0.0.1", port)) else {
        return false;
    };
    if socket
        .set_read_timeout(Some(Duration::from_millis(1_000)))
        .is_err()
    {
        return false;
    }
    let get = "GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    if socket.write_all(get.as_bytes()).is_err() {
        return false;
    }
    let bytes = read_to_close(&mut socket, Duration::from_secs(2));
    find(&bytes, b"ok").is_some()
}

#[test]
fn closing_the_client_mid_stream_leaves_the_server_healthy() {
    let server = start();
    {
        let mut socket = request(server.port, "/endless");
        // Wait for real bytes so the stream is genuinely in flight, then hang up.
        let seen = read_until(&mut socket, "data: forever", Duration::from_secs(5));
        assert!(
            find(&seen, b"data: forever").is_some(),
            "stream never started, so nothing was interrupted: {:?}",
            String::from_utf8_lossy(&seen)
        );
    } // socket dropped here: the client is gone mid-stream.

    // The server must notice the dead peer and return to accepting. Poll rather
    // than sleep a fixed amount, so a slow machine does not fail a correct server.
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if health_ok(server.port) {
            return;
        }
    }
    panic!("server never recovered after a mid-stream client close");
}
