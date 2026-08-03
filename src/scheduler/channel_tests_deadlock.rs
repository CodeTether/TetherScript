//! Deadlock-detection tests: reported, not hung.

use crate::scheduler::channel::{self, RecvOutcome};
use crate::value::Value;

use super::support::drain;

#[test]
fn all_parked_receive_is_reported_as_deadlock() {
    drain();
    let (tx, rx) = channel::bounded(1, "stalled").expect("capacity is positive");
    assert!(matches!(rx.recv(501), RecvOutcome::Parked));

    let report = channel::detect_deadlock(&[501]).expect("an all-parked state is provable");

    assert!(report.contains("channel deadlock"), "{report}");
    assert!(report.contains("stalled"), "{report}");
    assert!(report.contains("task 501"), "{report}");
    assert!(report.contains("recv"), "{report}");
    channel::cancel_task(501);
    drop(tx);
}

#[test]
fn no_deadlock_after_a_send_unparks_the_receiver() {
    drain();
    let (tx, rx) = channel::bounded(2, "live").expect("capacity is positive");
    assert!(matches!(rx.recv(511), RecvOutcome::Parked));

    tx.send(&Value::Int(1), 512).unwrap();

    // The send unparked the receiver and queued its wakeup, so progress is possible.
    assert!(channel::detect_deadlock(&[511]).is_none());
    drain();
}

#[test]
fn no_deadlock_when_a_task_is_still_runnable() {
    drain();
    let (tx, rx) = channel::bounded(1, "partly_idle").expect("capacity is positive");
    assert!(matches!(rx.recv(521), RecvOutcome::Parked));

    // Task 522 is live but not parked, so it may still send.
    assert!(channel::detect_deadlock(&[521, 522]).is_none());
    channel::cancel_task(521);
    drop(tx);
}

#[test]
fn no_deadlock_without_live_tasks() {
    assert!(channel::detect_deadlock(&[]).is_none());
}

#[test]
fn cancelled_parked_task_clears_the_deadlock_report() {
    drain();
    let (tx, rx) = channel::bounded(1, "recovered").expect("capacity is positive");
    assert!(matches!(rx.recv(531), RecvOutcome::Parked));
    assert!(channel::detect_deadlock(&[531]).is_some());

    channel::cancel_task(531);

    assert!(channel::detect_deadlock(&[531]).is_none());
    drop(tx);
}
