//! Ordering, backpressure, and wakeup tests.

use crate::scheduler::channel::{self, RecvOutcome, SendOutcome};
use crate::value::Value;

use super::support::{assert_parked, assert_woken, drain};

/// Unwrap an int from a receive outcome.
fn int(outcome: RecvOutcome) -> i64 {
    match outcome {
        RecvOutcome::Value(Value::Int(value)) => value,
        other => panic!("expected an int value, got {other:?}"),
    }
}

#[test]
fn send_then_receive_preserves_order() {
    drain();
    let (tx, rx) = channel::bounded(4, "order").expect("capacity is positive");

    for value in 1..=3 {
        assert_eq!(tx.send(&Value::Int(value), 101).unwrap(), SendOutcome::Sent);
    }

    assert_eq!(int(rx.recv(102)), 1);
    assert_eq!(int(rx.recv(102)), 2);
    assert_eq!(int(rx.recv(102)), 3);
}

#[test]
fn full_channel_parks_sender_and_resumes_after_receive() {
    drain();
    let (tx, rx) = channel::bounded(1, "backpressure").expect("capacity is positive");

    assert_eq!(tx.send(&Value::Int(1), 111).unwrap(), SendOutcome::Sent);
    assert_eq!(tx.send(&Value::Int(2), 111).unwrap(), SendOutcome::Parked);
    assert_parked(111);
    assert_eq!(tx.len(), tx.capacity());

    assert_eq!(int(rx.recv(112)), 1);

    assert_woken(111);
    assert_eq!(tx.send(&Value::Int(2), 111).unwrap(), SendOutcome::Sent);
    assert_eq!(int(rx.recv(112)), 2);
}

#[test]
fn interleaved_sends_and_receives_stay_first_in_first_out() {
    drain();
    let (tx, rx) = channel::bounded(2, "fifo").expect("capacity is positive");

    tx.send(&Value::Int(10), 121).unwrap();
    tx.send(&Value::Int(20), 121).unwrap();
    assert_eq!(int(rx.recv(122)), 10);
    tx.send(&Value::Int(30), 121).unwrap();
    assert_eq!(int(rx.recv(122)), 20);
    assert_eq!(int(rx.recv(122)), 30);
}

#[test]
fn zero_capacity_is_rejected_by_name() {
    let error = channel::bounded(0, "rendezvous").expect_err("zero capacity is invalid");

    assert!(error.contains("rendezvous"), "{error}");
    assert!(error.contains("greater than zero"), "{error}");
}
