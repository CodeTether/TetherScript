//! Each of the three payload-length forms decodes to the right length.
//!
//! The boundary values matter: 125 is the last 7-bit length, 126 is the first
//! 16-bit length, 65535 is the last 16-bit length, and 65536 is the first 64-bit
//! length. An off-by-one in the form selector shows up exactly here.

use super::wire::{client_frame, decode_client};
use tetherscript::websocket::encode::encode_server;
use tetherscript::websocket::opcode::Opcode;

#[test]
fn the_seven_bit_length_form_decodes() {
    let bytes = client_frame(true, 0x2, &[0xab; 125]);
    // 2 header bytes + 4 mask bytes + 125 payload bytes.
    assert_eq!(bytes.len(), 131);
    assert_eq!(bytes[1] & 0x7f, 125);
    let (frame, consumed) = decode_client(&bytes);
    assert_eq!(frame.payload.len(), 125);
    assert_eq!(consumed, 131);
}

#[test]
fn the_sixteen_bit_length_form_decodes_at_both_ends() {
    for len in [126usize, 65_535] {
        let bytes = client_frame(true, 0x2, &vec![0xcd; len]);
        assert_eq!(bytes[1] & 0x7f, 126);
        let (frame, consumed) = decode_client(&bytes);
        assert_eq!(frame.payload.len(), len);
        assert_eq!(consumed, len + 8);
    }
}

#[test]
fn the_sixty_four_bit_length_form_decodes() {
    let bytes = client_frame(true, 0x2, &vec![0xef; 65_536]);
    assert_eq!(bytes[1] & 0x7f, 127);
    let (frame, consumed) = decode_client(&bytes);
    assert_eq!(frame.payload.len(), 65_536);
    assert_eq!(consumed, 65_536 + 14);
}

#[test]
fn the_encoder_uses_the_minimal_length_form() {
    assert_eq!(encode_server(true, Opcode::Binary, &[0; 125])[1], 125);
    assert_eq!(encode_server(true, Opcode::Binary, &[0; 126])[1], 126);
    assert_eq!(encode_server(true, Opcode::Binary, &[0; 65_535])[1], 126);
    assert_eq!(encode_server(true, Opcode::Binary, &[0; 65_536])[1], 127);
}
