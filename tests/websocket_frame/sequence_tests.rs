//! Sequencing rules: fragmented control frames and illegal frame ordering.

use super::fragment_tests::feed;
use super::wire::client_frame;
use tetherscript::websocket::error::ProtocolError;
use tetherscript::websocket::frame::Frame;
use tetherscript::websocket::message::{Assembler, Message};
use tetherscript::websocket::opcode::Opcode;
use tetherscript::websocket::role::Role;

#[test]
fn a_fragmented_control_frame_is_rejected() {
    // Close, ping, and pong all carry FIN clear here, which §5.5 forbids.
    for opcode in [0x8u8, 0x9, 0xa] {
        let bytes = client_frame(false, opcode, &[]);
        let error = Frame::decode(&bytes, Role::Client).expect_err("must reject");
        assert_eq!(error, ProtocolError::FragmentedControlFrame);
    }
}

#[test]
fn a_ping_may_be_interleaved_into_a_fragmented_message() {
    let mut assembler = Assembler::new();
    feed(&mut assembler, &client_frame(false, 0x1, b"he")).unwrap();
    let ping = feed(&mut assembler, &client_frame(true, 0x9, b"p")).unwrap();
    assert_eq!(ping, Some(Message::Ping(b"p".to_vec())));
    let done = feed(&mut assembler, &client_frame(true, 0x0, b"llo")).unwrap();
    assert_eq!(done, Some(Message::Text("hello".into())));
}

#[test]
fn a_continuation_with_no_message_in_progress_is_rejected() {
    let mut assembler = Assembler::new();
    let error = feed(&mut assembler, &client_frame(true, 0x0, b"x")).expect_err("must reject");
    assert_eq!(error, ProtocolError::UnexpectedContinuation);
}

#[test]
fn a_data_frame_interleaved_into_a_fragmented_message_is_rejected() {
    let mut assembler = Assembler::new();
    feed(&mut assembler, &client_frame(false, 0x1, b"he")).unwrap();
    let frame = Frame {
        fin: true,
        opcode: Opcode::Text,
        payload: b"x".to_vec(),
    };
    let error = assembler.accept(frame).expect_err("must reject");
    assert_eq!(error, ProtocolError::InterleavedDataFrame);
}
