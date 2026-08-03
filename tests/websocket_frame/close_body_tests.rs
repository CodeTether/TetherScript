//! Close-frame bodies: the 2-byte big-endian code, the reason, and the
//! one-byte body that cannot hold a code.

use super::wire::{client_frame, decode_client};
use tetherscript::websocket::close::{self, CloseFrame};
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::message::{Assembler, Message};

#[test]
fn a_close_body_round_trips_a_big_endian_code_and_utf8_reason() {
    // 1000 = normal closure, big-endian 0x03E8, then the reason bytes.
    let body = [0x03, 0xe8, b'b', b'y', b'e'];
    let parsed = close::validate(&body).unwrap().expect("body is present");
    assert_eq!(parsed.code, 1000);
    assert_eq!(parsed.reason, "bye");
    assert_eq!(parsed.to_payload(), body.to_vec());
}

#[test]
fn a_close_frame_decodes_and_assembles_into_a_close_message() {
    let body = [0x03, 0xe8, b'b', b'y', b'e'];
    let (frame, _) = decode_client(&client_frame(true, 0x8, &body));
    let mut assembler = Assembler::new();
    let expected = CloseFrame {
        code: 1000,
        reason: "bye".into(),
    };
    assert_eq!(
        assembler.accept(frame).unwrap(),
        Some(Message::Close(Some(expected)))
    );
}

#[test]
fn an_empty_close_body_means_no_status_given() {
    assert_eq!(close::validate(&[]).unwrap(), None);
    let (frame, _) = decode_client(&client_frame(true, 0x8, &[]));
    let mut assembler = Assembler::new();
    assert_eq!(assembler.accept(frame).unwrap(), Some(Message::Close(None)));
}

#[test]
fn a_one_byte_close_body_cannot_hold_a_code() {
    assert_eq!(
        close::validate(&[0x03]),
        Err(ProtocolError::TruncatedCloseCode)
    );
}

#[test]
fn a_non_utf8_close_reason_is_rejected() {
    assert_eq!(
        close::validate(&[0x03, 0xe8, 0xff]),
        Err(ProtocolError::InvalidUtf8 {
            context: "close reason"
        })
    );
}
