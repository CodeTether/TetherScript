//! Close, drain, and end-of-stream tests.

use crate::scheduler::channel::{self, RecvOutcome};
use crate::value::Value;

use super::support::{assert_woken, drain};

#[test]
fn close_then_drain_then_end_of_stream() {
    drain();
    let (tx, rx) = channel::bounded(4, "drain").expect("capacity is positive");
    tx.send(&Value::Int(1), 201).unwrap();
    tx.send(&Value::Int(2), 201).unwrap();

    tx.close();

    // Closing must not discard buffered values; that is the classic bug.
    assert!(matches!(rx.recv(202), RecvOutcome::Value(Value::Int(1))));
    assert!(matches!(rx.recv(202), RecvOutcome::Value(Value::Int(2))));
    assert!(matches!(rx.recv(202), RecvOutcome::Ended));
    assert!(rx.is_ended());
}

#[test]
fn send_after_close_fails_by_channel_name() {
    drain();
    let (tx, _rx) = channel::bounded(2, "sealed").expect("capacity is positive");
    tx.close();

    let error = tx
        .send(&Value::Int(1), 211)
        .expect_err("closed send must fail");

    assert!(error.contains("sealed"), "{error}");
    assert!(error.contains("closed"), "{error}");
}

#[test]
fn receive_with_all_senders_dropped_completes() {
    drain();
    let (tx, rx) = channel::bounded(2, "orphaned").expect("capacity is positive");
    tx.send(&Value::Int(5), 221).unwrap();
    drop(tx);

    assert!(matches!(rx.recv(222), RecvOutcome::Value(Value::Int(5))));
    assert!(matches!(rx.recv(222), RecvOutcome::Ended));
}

#[test]
fn parked_receiver_wakes_when_last_sender_drops() {
    drain();
    let (tx, rx) = channel::bounded(1, "orphan_wake").expect("capacity is positive");
    assert!(matches!(rx.recv(231), RecvOutcome::Parked));

    drop(tx);

    assert_woken(231);
    assert!(matches!(rx.recv(231), RecvOutcome::Ended));
}
