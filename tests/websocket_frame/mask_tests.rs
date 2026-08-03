//! Masking: the round-trip, and the direction rules that make it a real check.

use super::wire::{KEY, client_frame, decode_client};
use tetherscript::websocket::encode::encode_server;
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::frame::Frame;
use tetherscript::websocket::mask;
use tetherscript::websocket::opcode::Opcode;
use tetherscript::websocket::role::Role;

#[test]
fn masking_is_its_own_inverse_over_the_rfc_vector() {
    let mut bytes = b"Hello".to_vec();
    mask::apply(&mut bytes, KEY);
    assert_eq!(bytes, vec![0x7f, 0x9f, 0x4d, 0x51, 0x58]);
    mask::apply(&mut bytes, KEY);
    assert_eq!(&bytes, b"Hello");
}

#[test]
fn a_masked_client_frame_decodes_to_the_plain_payload() {
    // The exact bytes from RFC 6455 §5.7.
    let wire = [
        0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58,
    ];
    assert_eq!(client_frame(true, 0x1, b"Hello"), wire.to_vec());
    let (frame, consumed) = decode_client(&wire);
    assert_eq!(frame.payload, b"Hello".to_vec());
    assert_eq!(consumed, 11);
}

#[test]
fn a_server_frame_is_never_masked() {
    let wire = encode_server(true, Opcode::Text, b"Hello");
    assert_eq!(wire, vec![0x81, 0x05, b'H', b'e', b'l', b'l', b'o']);
    // Bit 7 of byte 1 is MASK.
    assert_eq!(wire[1] & 0x80, 0, "server frames must not set MASK");
    // And it round-trips when read back as a server frame.
    let frame = Frame {
        fin: true,
        opcode: Opcode::Text,
        payload: b"Hello".to_vec(),
    };
    assert_eq!(frame.encode_server(), wire);
}

#[test]
fn an_unmasked_client_frame_is_rejected() {
    let wire = [0x81, 0x05, b'H', b'e', b'l', b'l', b'o'];
    let error = Frame::decode(&wire, Role::Client).expect_err("must reject");
    assert_eq!(error, ProtocolError::UnmaskedClientFrame);
}

#[test]
fn a_masked_server_frame_is_rejected() {
    let wire = client_frame(true, 0x1, b"Hello");
    let error = Frame::decode(&wire, Role::Server).expect_err("must reject");
    assert_eq!(error, ProtocolError::MaskedServerFrame);
}
