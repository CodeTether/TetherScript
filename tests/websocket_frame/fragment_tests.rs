//! Fragmentation: a fragmented message is reassembled into one message.

use super::wire::{client_frame, decode_client};
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::message::{Assembler, Message};

/// Decode a wire frame and feed it to `assembler`.
pub fn feed(assembler: &mut Assembler, bytes: &[u8]) -> Result<Option<Message>, ProtocolError> {
    let (frame, _) = decode_client(bytes);
    assembler.accept(frame)
}

#[test]
fn a_fragmented_text_message_is_reassembled() {
    let mut assembler = Assembler::new();
    // "he" + "ll" + "o" across three frames.
    let first = feed(&mut assembler, &client_frame(false, 0x1, b"he")).unwrap();
    assert_eq!(first, None);
    let second = feed(&mut assembler, &client_frame(false, 0x0, b"ll")).unwrap();
    assert_eq!(second, None);
    let done = feed(&mut assembler, &client_frame(true, 0x0, b"o")).unwrap();
    assert_eq!(done, Some(Message::Text("hello".into())));
}

#[test]
fn a_fragmented_binary_message_is_reassembled() {
    let mut assembler = Assembler::new();
    assert_eq!(
        feed(&mut assembler, &client_frame(false, 0x2, &[0x00, 0xff])).unwrap(),
        None
    );
    let done = feed(&mut assembler, &client_frame(true, 0x0, &[0x80])).unwrap();
    assert_eq!(done, Some(Message::Binary(vec![0x00, 0xff, 0x80])));
}

#[test]
fn a_multibyte_character_may_straddle_a_fragment_boundary() {
    let mut assembler = Assembler::new();
    // U+00E9 is 0xC3 0xA9; split it across two fragments.
    let first = feed(&mut assembler, &client_frame(false, 0x1, &[0xc3])).unwrap();
    assert_eq!(first, None);
    let done = feed(&mut assembler, &client_frame(true, 0x0, &[0xa9])).unwrap();
    assert_eq!(done, Some(Message::Text("é".into())));
}

#[test]
fn an_unfragmented_message_completes_in_one_frame() {
    let mut assembler = Assembler::new();
    let done = feed(&mut assembler, &client_frame(true, 0x1, b"hi")).unwrap();
    assert_eq!(done, Some(Message::Text("hi".into())));
}
