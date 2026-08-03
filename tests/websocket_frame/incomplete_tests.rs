//! Every strict prefix of a valid frame must be `Incomplete`, not `Malformed`.
//!
//! This is the property that keeps a streaming reader correct across TCP segment
//! boundaries. It is asserted over all three length forms *and* over a frame that
//! is followed by trailing bytes, so a decoder that greedily consumed its whole
//! buffer would fail here.

use super::wire::{client_frame, decode_client};
use tetherscript::websocket::frame::{DecodeOutcome, Frame};
use tetherscript::websocket::role::Role;

/// Assert every strict prefix of `bytes` decodes to `Incomplete`.
fn every_prefix_is_incomplete(bytes: &[u8]) {
    for cut in 0..bytes.len() {
        let outcome = Frame::decode(&bytes[..cut], Role::Client);
        assert_eq!(
            outcome,
            Ok(DecodeOutcome::Incomplete),
            "prefix of length {cut} must be Incomplete, not an error"
        );
    }
}

#[test]
fn every_prefix_of_a_seven_bit_frame_is_incomplete() {
    every_prefix_is_incomplete(&client_frame(true, 0x1, b"Hello"));
}

#[test]
fn every_prefix_of_a_sixteen_bit_frame_is_incomplete() {
    every_prefix_is_incomplete(&client_frame(true, 0x2, &[0xab; 200]));
}

#[test]
fn every_prefix_of_a_sixty_four_bit_frame_is_incomplete() {
    every_prefix_is_incomplete(&client_frame(true, 0x2, &vec![0xcd; 65_536]));
}

#[test]
fn every_prefix_of_an_empty_close_frame_is_incomplete() {
    every_prefix_is_incomplete(&client_frame(true, 0x8, &[]));
}

#[test]
fn a_decode_consumes_only_its_own_frame() {
    let mut bytes = client_frame(true, 0x1, b"one");
    let first_len = bytes.len();
    bytes.extend_from_slice(&client_frame(true, 0x1, b"two"));
    let (frame, consumed) = decode_client(&bytes);
    assert_eq!(frame.payload, b"one".to_vec());
    assert_eq!(consumed, first_len);
    let (second, _) = decode_client(&bytes[consumed..]);
    assert_eq!(second.payload, b"two".to_vec());
}
