//! Length-field bounds: the 64-bit MSB, the payload cap, and minimality.
//!
//! The oversized cases are asserted from the *header alone*, with no payload
//! supplied. That is the point: a peer must be refused for *claiming* a huge
//! length, before any buffer is sized from that claim.

use super::reject_tests::reject;
use super::wire::{KEY, client_frame};
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::limits::{self, MAX_PAYLOAD_LEN};

/// Build a 64-bit-length client frame header declaring `declared` bytes.
fn declared_64bit_header(declared: u64) -> Vec<u8> {
    let mut bytes = vec![0x82, 0x80 | 127];
    bytes.extend_from_slice(&declared.to_be_bytes());
    bytes.extend_from_slice(&KEY);
    bytes
}

#[test]
fn a_control_payload_over_125_bytes_is_rejected() {
    let bytes = client_frame(true, 0x8, &[0u8; 126]);
    assert_eq!(
        reject(&bytes),
        ProtocolError::ControlPayloadTooLarge { len: 126 }
    );
}

#[test]
fn the_sixty_four_bit_length_msb_must_be_zero() {
    let raw = 0x8000_0000_0000_0001u64;
    assert_eq!(
        reject(&declared_64bit_header(raw)),
        ProtocolError::LengthMsbSet { raw }
    );
}

#[test]
fn a_payload_beyond_the_bound_is_rejected_from_the_header_alone() {
    let declared = MAX_PAYLOAD_LEN + 1;
    assert_eq!(
        reject(&declared_64bit_header(declared)),
        ProtocolError::PayloadTooLarge {
            declared,
            max: MAX_PAYLOAD_LEN,
        }
    );
    assert!(limits::check_payload(MAX_PAYLOAD_LEN).is_ok());
}

#[test]
fn a_non_minimal_length_encoding_is_rejected() {
    // 5 encoded in the 16-bit form, which the 7-bit form could have carried.
    let mut bytes = vec![0x81, 0x80 | 126, 0x00, 0x05];
    bytes.extend_from_slice(&KEY);
    bytes.extend_from_slice(&[0; 5]);
    assert_eq!(reject(&bytes), ProtocolError::NonMinimalLength { len: 5 });
}
