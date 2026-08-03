//! Per-event writing and end-of-body handling.
//!
//! Split from [`super`] so the loop holds no transfer-coding knowledge: the pump
//! decides *when* to write, this module decides *how* the bytes are framed.

use std::io::Write;

use super::super::chunk::{self, Coding};
use super::super::shape::StreamSpec;
use super::super::write::{flush_all, Flow};
use super::super::{Outcome, StopReason};

/// Write one payload with the response's transfer coding, then flush.
///
/// # Arguments
///
/// * `out` — Destination socket.
/// * `spec` — The parsed streaming response, for its coding.
/// * `bytes` — Payload exactly as the generator produced it.
///
/// # Returns
///
/// [`Flow::Open`] when the bytes reached the client, [`Flow::Closed`] when the
/// peer has gone. An empty payload under chunked coding is skipped and reported
/// as `Open`, because `0` is the chunked terminator and a zero-length data chunk
/// would end the body early.
///
/// # Errors
///
/// Returns `Err` for a non-disconnect I/O failure.
pub(crate) fn event<W: Write>(
    out: &mut W,
    spec: &StreamSpec,
    bytes: &[u8],
) -> Result<Flow, String> {
    match spec.coding {
        Coding::Close => flush_all(out, bytes),
        Coding::Chunked => match chunk::frame_bytes(bytes) {
            Some(framed) => flush_all(out, &framed),
            None => Ok(Flow::Open),
        },
    }
}

/// Close the body and build the outcome.
///
/// # Arguments
///
/// * `out` — Destination socket.
/// * `spec` — The parsed streaming response, for its coding.
/// * `events` — Payloads flushed so far.
/// * `stop` — Why the stream ended.
///
/// # Returns
///
/// The [`Outcome`]. Under chunked coding the terminating zero-length chunk is
/// written first, unless the peer already disconnected — writing a terminator to
/// a dead socket is pointless and its failure is not news.
pub(crate) fn finish<W: Write>(
    out: &mut W,
    spec: &StreamSpec,
    events: u32,
    stop: StopReason,
) -> Outcome {
    if spec.coding == Coding::Chunked && stop != StopReason::Disconnected {
        chunk::finish(out);
    }
    Outcome::new(events, stop)
}
