//! Select tests: a receive must be multiplexable alongside other arms.

use crate::scheduler::channel::{self, SelectOutcome};
use crate::value::Value;

use super::support::{assert_not_parked, assert_parked, assert_woken, drain};

#[test]
fn select_takes_whichever_channel_is_ready() {
    drain();
    let (left_tx, left_rx) = channel::bounded(1, "left").expect("capacity is positive");
    let (right_tx, right_rx) = channel::bounded(1, "right").expect("capacity is positive");
    right_tx.send(&Value::Int(9), 401).unwrap();

    let chosen = channel::select_recv(&[&left_rx, &right_rx], 402).expect("two arms");

    assert!(matches!(chosen, SelectOutcome::Ready(1, Value::Int(9))));
    assert_not_parked(402);
    drop((left_tx, right_tx));
}

#[test]
fn select_parks_on_every_arm_and_any_sender_wakes_it() {
    drain();
    let (left_tx, left_rx) = channel::bounded(1, "left").expect("capacity is positive");
    let (right_tx, right_rx) = channel::bounded(1, "right").expect("capacity is positive");

    assert!(matches!(
        channel::select_recv(&[&left_rx, &right_rx], 411).expect("two arms"),
        SelectOutcome::Parked
    ));
    assert_parked(411);

    left_tx.send(&Value::Int(1), 412).unwrap();

    assert_woken(411);
    assert_not_parked(411);
    drop(right_tx);
}

#[test]
fn select_without_arms_is_rejected() {
    let error = channel::select_recv(&[], 421).expect_err("a select with no arms cannot complete");

    assert!(error.contains("at least one receiver"), "{error}");
}
