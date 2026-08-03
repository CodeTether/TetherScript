//! Timed reads that distinguish "nothing yet" from a real failure.
//!
//! Split from [`super`] so connection setup and read strategy stay separate.
//!
//! A `WouldBlock` or `TimedOut` is not an error here: the socket carries a live
//! stream, so quiet periods are expected. Treating them as failures would make
//! every test a race.

use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use super::scan::find;

/// Read every byte until the peer closes, with a hard ceiling on waiting.
///
/// # Arguments
///
/// * `stream` — Socket to drain.
/// * `budget` — Maximum time to spend reading.
///
/// # Returns
///
/// Whatever arrived. A timeout ends the read normally, since a stream under test
/// may legitimately outlive the budget.
///
/// # Panics
///
/// Panics on an I/O error that is not a timeout, because that indicates a broken
/// harness rather than a slow server.
pub(crate) fn read_to_close(stream: &mut TcpStream, budget: Duration) -> Vec<u8> {
    let deadline = Instant::now() + budget;
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(error) if soft(error.kind()) => break,
            Err(error) => panic!("read failed: {error}"),
        }
    }
    out
}

/// Read until the accumulated bytes contain `needle`, or the budget expires.
///
/// # Arguments
///
/// * `stream` — Socket to read.
/// * `needle` — Byte sequence to wait for.
/// * `budget` — Maximum time to spend waiting.
///
/// # Returns
///
/// Everything read so far. This is the helper that proves *incremental* delivery:
/// it returns as soon as the marker appears, without waiting for the close.
///
/// # Panics
///
/// Panics on a non-timeout I/O error, as [`read_to_close`] does.
pub(crate) fn read_until(stream: &mut TcpStream, needle: &str, budget: Duration) -> Vec<u8> {
    let deadline = Instant::now() + budget;
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        if find(&out, needle.as_bytes()).is_some() {
            return out;
        }
        match stream.read(&mut buf) {
            Ok(0) => return out,
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(error) if soft(error.kind()) => {}
            Err(error) => panic!("read failed: {error}"),
        }
    }
    out
}

/// Whether an error kind merely means "nothing available right now".
fn soft(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
