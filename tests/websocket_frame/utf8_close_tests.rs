//! UTF-8 validity for text payloads; binary payloads stay unconstrained.

use super::wire::{client_frame, decode_client};
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::frame::Frame;
use tetherscript::websocket::message::Assembler;
use tetherscript::websocket::role::Role;

#[test]
fn invalid_utf8_in_a_text_frame_is_rejected() {
    // 0x80 is a bare continuation byte, 0xC3 0x28 a truncated two-byte sequence,
    // 0xFF 0xFE never appears in valid UTF-8 at all.
    for bad in [vec![0x80u8], vec![0xc3, 0x28], vec![0xff, 0xfe]] {
        let bytes = client_frame(true, 0x1, &bad);
        let error = Frame::decode(&bytes, Role::Client).expect_err("must reject");
        assert_eq!(
            error,
            ProtocolError::InvalidUtf8 {
                context: "text payload"
            }
        );
    }
}

#[test]
fn invalid_utf8_split_across_fragments_is_rejected_at_reassembly() {
    let mut assembler = Assembler::new();
    let (head, _) = decode_client(&client_frame(false, 0x1, &[0xc3]));
    assert_eq!(assembler.accept(head).unwrap(), None);
    let (tail, _) = decode_client(&client_frame(true, 0x0, &[0x28]));
    let error = assembler.accept(tail).expect_err("must reject");
    assert_eq!(
        error,
        ProtocolError::InvalidUtf8 {
            context: "text message"
        }
    );
}

#[test]
fn a_binary_frame_may_carry_arbitrary_bytes() {
    let bytes = client_frame(true, 0x2, &[0xff, 0x00, 0x80]);
    let (frame, _) = decode_client(&bytes);
    assert_eq!(frame.payload, vec![0xff, 0x00, 0x80]);
}
