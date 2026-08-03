//! Endpoint-loss and cancellation tests: neither side may park forever.

use crate::scheduler::channel::{self, RecvOutcome, SendOutcome};
use crate::value::Value;

use super::support::{assert_no_wakeups, assert_not_parked, assert_woken, drain};

#[test]
fn send_with_no_receivers_fails_by_channel_name() {
    drain();
    let (tx, rx) = channel::bounded(1, "abandoned").expect("capacity is positive");
    drop(rx);

    let error = tx
        .send(&Value::Int(1), 301)
        .expect_err("send without receivers must fail");

    assert!(error.contains("abandoned"), "{error}");
    assert!(error.contains("all receivers were dropped"), "{error}");
}

#[test]
fn parked_sender_wakes_when_last_receiver_drops() {
    drain();
    let (tx, rx) = channel::bounded(1, "abandon_wake").expect("capacity is positive");
    tx.send(&Value::Int(1), 311).unwrap();
    assert_eq!(tx.send(&Value::Int(2), 311).unwrap(), SendOutcome::Parked);

    drop(rx);

    assert_woken(311);
    assert!(tx.send(&Value::Int(2), 311).is_err());
}

#[test]
fn cancelling_a_parked_receiver_cleans_up() {
    drain();
    let (tx, rx) = channel::bounded(1, "cancelled").expect("capacity is positive");
    assert!(matches!(rx.recv(321), RecvOutcome::Parked));

    assert!(channel::cancel_task(321));

    assert_not_parked(321);
    // No dangling waiter survives: a later send wakes nobody.
    tx.send(&Value::Int(1), 322).unwrap();
    assert_no_wakeups();
    assert!(!channel::cancel_task(321));
}
