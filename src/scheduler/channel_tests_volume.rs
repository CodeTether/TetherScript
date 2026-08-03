//! Volume test: many values through a small buffer lose and duplicate nothing.

use crate::scheduler::channel::{self, RecvOutcome, SendOutcome};
use crate::value::Value;

use super::support::drain;

#[test]
fn many_values_through_a_small_buffer_arrive_exactly_once() {
    drain();
    let (tx, rx) = channel::bounded(3, "volume").expect("capacity is positive");
    let mut received = Vec::new();
    let mut next = 0_i64;

    // Drive producer and consumer by hand, the way the scheduler alternates them.
    while received.len() < 500 {
        while next < 500 {
            match tx.send(&Value::Int(next), 601).expect("receiver is live") {
                SendOutcome::Sent => next += 1,
                SendOutcome::Parked => break,
            }
        }
        match rx.recv(602) {
            RecvOutcome::Value(Value::Int(value)) => received.push(value),
            other => panic!("unexpected receive outcome {other:?}"),
        }
        drain();
    }

    assert_eq!(received, (0..500).collect::<Vec<i64>>());
}

#[test]
fn buffer_never_exceeds_capacity_under_pressure() {
    drain();
    let (tx, rx) = channel::bounded(2, "bounded").expect("capacity is positive");

    for value in 0..10 {
        if tx.send(&Value::Int(value), 611).unwrap() == SendOutcome::Parked {
            assert_eq!(tx.len(), 2);
            assert!(matches!(rx.recv(612), RecvOutcome::Value(_)));
            drain();
        }
        assert!(tx.len() <= tx.capacity());
    }
}
