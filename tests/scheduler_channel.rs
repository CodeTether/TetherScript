//! Integration tests for the bounded scheduler channel.
//!
//! These exercise the public library surface (`tetherscript::scheduler::channel`)
//! rather than the CLI, because the `chan_*` built-ins are registered by the
//! integrator. Channel state is thread-local, so each test drains any state a
//! previously executed test left behind and asserts membership rather than exact
//! equality — that keeps the file correct under `--test-threads=1` too.

use tetherscript::scheduler::channel::{self, RecvOutcome, SelectOutcome, SendOutcome};
use tetherscript::value::Value;

/// Discard wakeups left by an earlier test on this thread.
fn reset() {
    let _stale = channel::take_wakeups();
}

/// Unwrap an int value from a receive outcome.
fn int(outcome: RecvOutcome) -> i64 {
    match outcome {
        RecvOutcome::Value(Value::Int(value)) => value,
        other => panic!("expected an int, got {other:?}"),
    }
}

/// Assert that `task` was queued for wakeup.
fn assert_woken(task: u64) {
    let woken = channel::take_wakeups();
    assert!(woken.contains(&task), "expected {task} in {woken:?}");
}

#[test]
fn send_then_receive_preserves_order() {
    reset();
    let (tx, rx) = channel::bounded(8, "order").expect("capacity is positive");

    for value in 1..=5 {
        assert_eq!(
            tx.send(&Value::Int(value), 1001).unwrap(),
            SendOutcome::Sent
        );
    }

    assert_eq!(
        (1..=5).map(|_| int(rx.recv(1002))).collect::<Vec<i64>>(),
        vec![1, 2, 3, 4, 5]
    );
}

#[test]
fn bounded_channel_parks_sender_at_capacity_and_resumes_after_receive() {
    reset();
    let (tx, rx) = channel::bounded(2, "backpressure").expect("capacity is positive");
    tx.send(&Value::Int(1), 1011).unwrap();
    tx.send(&Value::Int(2), 1011).unwrap();

    // Backpressure is the feature: the buffer refuses to grow.
    assert_eq!(tx.send(&Value::Int(3), 1011).unwrap(), SendOutcome::Parked);
    assert_eq!(tx.len(), tx.capacity());
    assert!(channel::parked_tasks().contains(&1011));

    assert_eq!(int(rx.recv(1012)), 1);

    assert_woken(1011);
    assert_eq!(tx.send(&Value::Int(3), 1011).unwrap(), SendOutcome::Sent);
}

#[test]
fn fifo_order_is_preserved_under_interleaving() {
    reset();
    let (tx, rx) = channel::bounded(2, "fifo").expect("capacity is positive");
    let mut seen = Vec::new();

    for value in 0..6 {
        if tx.send(&Value::Int(value), 1021).unwrap() == SendOutcome::Parked {
            seen.push(int(rx.recv(1022)));
            reset();
            tx.send(&Value::Int(value), 1021).unwrap();
        }
    }
    while !rx.is_empty() {
        seen.push(int(rx.recv(1022)));
    }

    assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn close_then_drain_then_end_of_stream() {
    reset();
    let (tx, rx) = channel::bounded(4, "drain").expect("capacity is positive");
    tx.send(&Value::Int(1), 1031).unwrap();
    tx.send(&Value::Int(2), 1031).unwrap();

    tx.close();

    assert_eq!(int(rx.recv(1032)), 1);
    assert_eq!(int(rx.recv(1032)), 2);
    assert!(matches!(rx.recv(1032), RecvOutcome::Ended));
}

#[test]
fn receive_with_all_senders_dropped_completes() {
    reset();
    let (tx, rx) = channel::bounded(2, "orphaned").expect("capacity is positive");
    tx.send(&Value::Int(1), 1041).unwrap();
    drop(tx);

    assert_eq!(int(rx.recv(1042)), 1);
    assert!(matches!(rx.recv(1042), RecvOutcome::Ended));
}

#[test]
fn send_with_no_receivers_fails_by_name() {
    reset();
    let (tx, rx) = channel::bounded(1, "abandoned").expect("capacity is positive");
    drop(rx);

    let error = tx
        .send(&Value::Int(1), 1051)
        .expect_err("a send with no receiver must fail, not park");

    assert!(error.contains("abandoned"), "{error}");
    assert!(error.contains("all receivers were dropped"), "{error}");
}

#[test]
fn select_over_two_channels_takes_whichever_is_ready() {
    reset();
    let (left_tx, left_rx) = channel::bounded(1, "left").expect("capacity is positive");
    let (right_tx, right_rx) = channel::bounded(1, "right").expect("capacity is positive");
    right_tx.send(&Value::Int(2), 1061).unwrap();

    let chosen = channel::select_recv(&[&left_rx, &right_rx], 1062).expect("two arms");
    assert!(matches!(chosen, SelectOutcome::Ready(1, Value::Int(2))));

    left_tx.send(&Value::Int(1), 1061).unwrap();
    let chosen = channel::select_recv(&[&left_rx, &right_rx], 1062).expect("two arms");
    assert!(matches!(chosen, SelectOutcome::Ready(0, Value::Int(1))));
}

#[test]
fn cancelling_a_parked_receiver_cleans_up() {
    reset();
    let (tx, rx) = channel::bounded(1, "cancelled").expect("capacity is positive");
    assert!(matches!(rx.recv(1071), RecvOutcome::Parked));

    assert!(channel::cancel_task(1071));

    assert!(!channel::parked_tasks().contains(&1071));
    tx.send(&Value::Int(1), 1072).unwrap();
    assert!(
        !channel::take_wakeups().contains(&1071),
        "a cancelled task must leave no dangling waiter"
    );
}

#[test]
fn all_parked_state_is_reported_rather_than_hanging() {
    reset();
    let (tx, rx) = channel::bounded(1, "stalled").expect("capacity is positive");
    assert!(matches!(rx.recv(1081), RecvOutcome::Parked));

    let report = channel::detect_deadlock(&[1081]).expect("all-parked is provable here");

    assert!(report.contains("channel deadlock"), "{report}");
    assert!(report.contains("stalled"), "{report}");
    channel::cancel_task(1081);
    drop(tx);
}

#[test]
fn many_values_through_a_small_buffer_are_neither_lost_nor_duplicated() {
    reset();
    let (tx, rx) = channel::bounded(3, "volume").expect("capacity is positive");
    let mut received = Vec::new();
    let mut next = 0_i64;

    while received.len() < 1_000 {
        while next < 1_000 && tx.send(&Value::Int(next), 1091).unwrap() == SendOutcome::Sent {
            next += 1;
        }
        received.push(int(rx.recv(1092)));
        reset();
    }

    assert_eq!(received, (0..1_000).collect::<Vec<i64>>());
    assert!(rx.is_empty());
}
