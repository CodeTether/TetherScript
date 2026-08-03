//! Tests for the failure, select, and deadlock `chan_*` built-ins.

use std::cell::RefCell;
use std::rc::Rc;

use crate::scheduler::channel::{
    self, channel_deadlock, channel_drop_receiver, channel_len, channel_open, channel_recv,
    channel_select, channel_send,
};
use crate::value::Value;

use super::helpers::{err, handle, ok, reset, text};
use super::maps::{field, status};

#[test]
fn send_without_receivers_reports_the_channel_name() {
    reset();
    let id = handle(channel_open(&[Value::Int(1), text("abandoned")]).unwrap());
    channel_drop_receiver(&[Value::Int(id)]).unwrap();

    let message = err(channel_send(&[Value::Int(id), Value::Int(1)]).unwrap());

    assert!(message.contains("abandoned"), "{message}");
}

#[test]
fn select_reports_the_ready_channel() {
    reset();
    let first = handle(channel_open(&[Value::Int(1), text("a")]).unwrap());
    let second = handle(channel_open(&[Value::Int(1), text("b")]).unwrap());
    channel_send(&[Value::Int(second), Value::Int(5)]).unwrap();
    let list = Value::List(Rc::new(RefCell::new(vec![
        Value::Int(first),
        Value::Int(second),
    ])));

    let chosen = ok(channel_select(&[list]).unwrap());

    assert_eq!(status(&chosen), "value");
    assert!(matches!(field(&chosen, "index"), Some(Value::Int(1))));
    assert!(matches!(field(&chosen, "channel"), Some(Value::Int(_))));
    assert!(matches!(field(&chosen, "value"), Some(Value::Int(5))));
}

#[test]
fn deadlock_is_reported_rather_than_hanging() {
    reset();
    let id = handle(channel_open(&[Value::Int(1), text("stalled")]).unwrap());
    assert_eq!(
        status(&ok(channel_recv(&[Value::Int(id)]).unwrap())),
        "parked"
    );

    let message = err(channel_deadlock(&[]).unwrap());

    assert!(message.contains("stalled"), "{message}");
    channel::cancel_task(channel::current_task());
}

#[test]
fn unknown_handles_are_named_in_the_error() {
    let message = err(channel_len(&[Value::Int(999_999)]).unwrap());

    assert!(message.contains("999999"), "{message}");
}
