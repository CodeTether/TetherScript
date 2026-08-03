//! The property the whole feature exists for: incremental delivery.
//!
//! An SSE client that receives nothing until the connection closes cannot tell a
//! working server from a hung one. So it is not enough that the right bytes
//! eventually arrive — the first event must be readable while the server is still
//! producing later ones.

use std::time::{Duration, Instant};

use super::server::start;
use super::socket::{find, read_to_close, read_until, request};

#[test]
fn events_arrive_incrementally_rather_than_all_at_the_end() {
    let server = start();
    let mut socket = request(server.port, "/slow");
    let started = Instant::now();
    // The route sleeps 150 ms per event and sends three, so a buffering server
    // could not surface the first event before roughly 450 ms.
    let early = read_until(&mut socket, "data: tick 1", Duration::from_secs(3));
    let first_seen = started.elapsed();
    assert!(
        find(&early, b"data: tick 1").is_some(),
        "first event never arrived: {:?}",
        String::from_utf8_lossy(&early)
    );
    assert!(
        find(&early, b"data: tick 3").is_none(),
        "the whole stream arrived at once, so nothing was streamed: {:?}",
        String::from_utf8_lossy(&early)
    );
    assert!(
        first_seen < Duration::from_millis(400),
        "first event took {first_seen:?}; buffering defeats the feature"
    );
    let rest = read_to_close(&mut socket, Duration::from_secs(3));
    assert!(
        find(&rest, b"data: tick 3").is_some(),
        "stream truncated after the first events: {:?}",
        String::from_utf8_lossy(&rest)
    );
}
