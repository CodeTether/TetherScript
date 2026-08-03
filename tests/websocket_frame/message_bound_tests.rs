//! The reassembled-message cap, which bounds a fragment sequence as a whole.
//!
//! This is a distinct bound from the per-frame payload cap: without it, a peer
//! could stay inside the frame bound on every individual fragment and still grow
//! the reassembly buffer without limit.

use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::limits::{self, MAX_MESSAGE_LEN, MAX_PAYLOAD_LEN};

#[test]
fn the_message_bound_admits_exactly_the_limit() {
    assert!(limits::check_message(0).is_ok());
    assert!(limits::check_message(MAX_MESSAGE_LEN).is_ok());
}

#[test]
fn the_message_bound_rejects_one_byte_over() {
    let over = MAX_MESSAGE_LEN + 1;
    assert_eq!(
        limits::check_message(over),
        Err(ProtocolError::MessageTooLarge {
            total: over,
            max: MAX_MESSAGE_LEN,
        })
    );
}

#[test]
fn the_message_bound_is_larger_than_a_single_frame_bound() {
    // Reassembly must be able to span several maximum-size frames, otherwise the
    // message cap would be the effective frame cap.
    assert!(MAX_MESSAGE_LEN as u64 > MAX_PAYLOAD_LEN);
}

#[test]
fn the_documented_bound_values_are_what_the_module_docs_claim() {
    assert_eq!(MAX_PAYLOAD_LEN, 16 * 1024 * 1024);
    assert_eq!(MAX_MESSAGE_LEN, 64 * 1024 * 1024);
    assert_eq!(limits::MAX_CONTROL_PAYLOAD_LEN, 125);
}
