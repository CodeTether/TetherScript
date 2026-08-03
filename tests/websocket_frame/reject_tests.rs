//! Reserved opcodes and reserved RSV bits are rejected, not ignored.
//!
//! Each frame here is header-only where possible: these violations must be caught
//! from the header alone, before the declared length is used to size anything.

use super::wire::client_frame;
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::frame::Frame;
use tetherscript::websocket::role::Role;

/// Decode `bytes` as a client frame and return the violation it triggers.
pub fn reject(bytes: &[u8]) -> ProtocolError {
    Frame::decode(bytes, Role::Client).expect_err("must be rejected")
}

#[test]
fn every_reserved_opcode_is_rejected() {
    // 0x3..=0x7 are reserved non-control, 0xB..=0xF reserved control.
    for opcode in [0x3u8, 0x4, 0x5, 0x6, 0x7, 0xb, 0xc, 0xd, 0xe, 0xf] {
        let bytes = client_frame(true, opcode, &[]);
        assert_eq!(
            reject(&bytes),
            ProtocolError::ReservedOpcode { bits: opcode }
        );
    }
}

#[test]
fn any_non_zero_rsv_bit_is_rejected() {
    for (bit, rsv) in [(0x40u8, 0b100u8), (0x20, 0b010), (0x10, 0b001)] {
        let mut bytes = client_frame(true, 0x1, b"hi");
        bytes[0] |= bit;
        assert_eq!(reject(&bytes), ProtocolError::ReservedBitSet { rsv });
    }
}

#[test]
fn all_three_rsv_bits_together_are_rejected() {
    let mut bytes = client_frame(true, 0x1, b"hi");
    bytes[0] |= 0x70;
    assert_eq!(reject(&bytes), ProtocolError::ReservedBitSet { rsv: 0b111 });
}
