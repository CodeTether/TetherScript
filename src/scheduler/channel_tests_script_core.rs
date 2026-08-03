//! Tests for the core `chan_*` built-ins: open, send, receive, and close.

use crate::scheduler::channel::{
    channel_close, channel_ended, channel_len, channel_open, channel_recv, channel_send,
};
use crate::value::Value;

use super::helpers::{handle, ok, reset, text};
use super::maps::{assert_text, field, status};

#[test]
fn open_send_and_receive_round_trip() {
    reset();
    let id = handle(channel_open(&[Value::Int(2), text("jobs")]).unwrap());

    assert_text(&ok(channel_send(&[Value::Int(id), Value::Int(41)]).unwrap()), "sent");

    let received = ok(channel_recv(&[Value::Int(id)]).unwrap());
    assert_eq!(status(&received), "value");
    assert!(matches!(field(&received, "value"), Some(Value::Int(41))));
}

#[test]
fn full_channel_reports_parked_then_recovers() {
    reset();
    let id = handle(channel_open(&[Value::Int(1), text("jobs")]).unwrap());
    channel_send(&[Value::Int(id), Value::Int(1)]).unwrap();

    let parked = ok(channel_send(&[Value::Int(id), Value::Int(2)]).unwrap());
    assert_text(&parked, "parked");

    channel_recv(&[Value::Int(id)]).unwrap();
    assert_text(&ok(channel_send(&[Value::Int(id), Value::Int(2)]).unwrap()), "sent");
    assert!(matches!(
        ok(channel_len(&[Value::Int(id)]).unwrap()),
        Value::Int(1)
    ));
}

#[test]
fn close_lets_the_receiver_drain_then_see_end() {
    reset();
    let id = handle(channel_open(&[Value::Int(2), text("drain")]).unwrap());
    channel_send(&[Value::Int(id), Value::Int(7)]).unwrap();
    channel_close(&[Value::Int(id)]).unwrap();

    let first = ok(channel_recv(&[Value::Int(id)]).unwrap());
    assert_eq!(status(&first), "value");
    assert!(matches!(field(&first, "value"), Some(Value::Int(7))));
    assert_eq!(status(&ok(channel_recv(&[Value::Int(id)]).unwrap())), "end");
    assert!(matches!(
        ok(channel_ended(&[Value::Int(id)]).unwrap()),
        Value::Bool(true)
    ));
}
