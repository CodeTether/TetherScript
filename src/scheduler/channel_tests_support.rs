//! Isolation helpers for the channel unit tests.
//!
//! Channel state is thread-local. The test harness normally gives each `#[test]`
//! its own thread, but `--test-threads=1` runs them all on one thread, so tests
//! must not assume a globally empty park table or wakeup queue. Every test
//! therefore drains the wakeup queue first and uses task ids unique to itself,
//! then asserts membership rather than exact equality.

use crate::scheduler::channel;

/// Discard any wakeups queued by a previously executed test.
pub(super) fn drain() {
    let _stale = channel::take_wakeups();
}

/// Assert that `task` was queued for wakeup.
pub(super) fn assert_woken(task: u64) {
    let woken = channel::take_wakeups();
    assert!(woken.contains(&task), "expected {task} in {woken:?}");
}

/// Assert that no task was queued for wakeup.
pub(super) fn assert_no_wakeups() {
    let woken = channel::take_wakeups();
    assert!(woken.is_empty(), "expected no wakeups, got {woken:?}");
}

/// Assert that `task` is currently parked on some channel.
pub(super) fn assert_parked(task: u64) {
    let parked = channel::parked_tasks();
    assert!(parked.contains(&task), "expected {task} in {parked:?}");
}

/// Assert that `task` is not parked on any channel.
pub(super) fn assert_not_parked(task: u64) {
    let parked = channel::parked_tasks();
    assert!(!parked.contains(&task), "expected {task} absent from {parked:?}");
}
